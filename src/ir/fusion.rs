use crate::ir;
use crate::rust_wgpu_backend::ffi;
use crate::shadergen::{FuseDescriptor, FuseSource, FusedResource, ShaderModule};

use std::collections::hash_map::{Entry, HashMap};
use std::ops::Range;

use super::ffi::GpuKernelResourceBinding;

/// Criteria for automatic fusing:
///  1. Dispatches are sequential.
///     Currently, this means that they can only be separated by None nodes.
///  2. Dispatches form a dependency chain.
///     Otherwise, fusing may be a performance loss since the GPU can execute them in parallel.
///  3. Dispatches share workgroup dimensions and local sizes.
#[derive(Debug)]
pub struct Opportunity {
    /// The scheduling nodes which should be replaced by this kernel dispatch.
    pub bounds: Range<ir::NodeId>,
    /// The fused kernel, ready for insertion into the FFI system.
    pub kernel: ffi::GpuKernel,
    /// The concrete workgroup dimensions of the fused kernel.
    pub dimensions: [Option<ir::NodeId>; 3],
    /// The concrete input arguments to the fused kernel.
    pub inputs: Vec<ir::NodeId>,
    /// The concrete outputs of the fused kernel.
    pub outputs: Vec<ir::NodeId>,
}

#[derive(Debug)]
struct SlotState {
    /// The type of this slot.
    ty: ffi::TypeId,
    /// The binding of this slot in the fused shader (within group 0)
    binding: u32,
    /// Is this type an input?
    input: bool,
    /// Is this type an output? (note: at least one of `input`, `output` must be set)
    output: bool,
}

#[derive(Debug)]
struct FuseState {
    start: ir::NodeId,
    workgroup_dimensions: [Option<ir::NodeId>; 3],
    local_size: [u32; 3],
    next_binding: u32,
    /// (shader source, entry point)
    kernels: Vec<(ShaderModule, String)>,
    resources: HashMap<(usize, u32, u32), FusedResource>,
    slots: HashMap<ir::NodeId, SlotState>,
}

impl FuseState {
    pub fn new(prog: &ir::Program, start: ir::NodeId, dispatch: DispatchInfo) -> FuseState {
        let kernel = prog.native_interface.external_functions[dispatch.id.0]
            .get_gpu_kernel()
            .expect("kernel fusion: not a GPU kernel!");

        let local_size = kernel.shader_module.local_size(&kernel.entry_point);

        let mut state = FuseState {
            start,
            workgroup_dimensions: dispatch.workgroup_dimensions,
            local_size,
            next_binding: 0,
            kernels: Vec::new(),
            resources: HashMap::new(),
            slots: HashMap::new(),
        };

        // TODO: Shader setup is duplicated here
        let result = state.fuse(prog, dispatch);
        assert!(result);
        return state;
    }

    fn register(
        &mut self,
        module_index: usize,
        node: ir::NodeId,
        ty: ffi::TypeId,
        is_input: bool,
        resource: &ffi::GpuKernelResourceBinding,
    ) {
        let binding = match self.slots.entry(node) {
            Entry::Occupied(mut entry) => {
                let state = entry.get_mut();
                assert_eq!(ty, state.ty, "mismatched kernel types");
                state.input |= is_input;
                state.output |= !is_input;
                state.binding
            }
            Entry::Vacant(entry) => {
                let binding = self.next_binding;
                self.next_binding += 1;
                entry.insert(SlotState {
                    ty,
                    binding,
                    input: is_input,
                    output: !is_input,
                });
                binding
            }
        };
        self.resources.insert(
            (module_index, resource.group as u32, resource.binding as u32),
            FusedResource::Binding { group: 0, binding },
        );
    }

    pub fn fuse(&mut self, prog: &ir::Program, dispatch: DispatchInfo) -> bool {
        if self.workgroup_dimensions != dispatch.workgroup_dimensions {
            return false;
        }

        let kernel = prog.native_interface.external_functions[dispatch.id.0]
            .get_gpu_kernel()
            .expect("kernel fusion: not a GPU kernel!");

        if self.local_size != kernel.shader_module.local_size(&kernel.entry_point) {
            return false;
        }

        let module_index = self.kernels.len();
        let mut dependency_chain = false;

        // Optimistically update our resource map and slot to binding map, as if fusion would
        // succeed. It's not illegal to have *extra* elements in those maps!
        for (in_index, input) in dispatch.inputs.iter().enumerate() {
            let ty = kernel.input_types[in_index];
            let resource = kernel
                .resource_bindings
                .iter()
                .find(|r| r.input == Some(in_index))
                .expect("unknown input");
            dependency_chain |= self.slots.contains_key(input);
            self.register(module_index, *input, ty, true, resource);
        }

        for (out_index, output) in dispatch.outputs.iter().enumerate() {
            let ty = kernel.output_types[out_index];
            let resource = kernel
                .resource_bindings
                .iter()
                .find(|r| r.output == Some(out_index))
                .expect("unknown output");
            self.register(module_index, *output, ty, false, resource);
        }

        // Update the module *if* any dependency chains were involved, or if this is the
        // first module, since otherwise we'd never make any progress.
        if (self.kernels.is_empty() || dependency_chain) {
            // TODO: Should really store a reference here
            let module = (kernel.shader_module.clone(), kernel.entry_point.clone());
            self.kernels.push(module);
            return true;
        } else {
            return false;
        }
    }

    pub fn finish(
        self,
        end: ir::OperationId,
        funclet_id: ir::FuncletId,
        ops: &mut Vec<Opportunity>,
    ) {
        if (self.kernels.len() <= 1) {
            // We're not actually fusing anything...
            return;
        }

        let sources: Vec<_> = self
            .kernels
            .iter()
            .map(|(shader_module, entry_point)| FuseSource {
                shader_module,
                entry_point,
            })
            .collect();
        let desc = FuseDescriptor {
            sources: sources.as_slice(),
            resources: &self.resources,
        };
        let shader_module = ShaderModule::fuse(desc);

        let mut inputs = Vec::new();
        let mut input_types = Vec::new();
        let mut outputs = Vec::new();
        let mut output_types = Vec::new();
        let mut resource_bindings: Vec<GpuKernelResourceBinding> = Vec::new();

        // Each GPU binding should have at most one input slot and at most one output slot
        // assigned to it. Otherwise we would be merging equivalent inputs or equivalent outputs,
        // which is out-of-scope
        for (&slot, state) in self.slots.iter() {
            let existing = resource_bindings
                .iter_mut()
                .find(|b| b.binding == state.binding as usize);
            let res = match existing {
                Some(inner) => {
                    assert_eq!(0, inner.group, "foreign binding?");
                    inner
                }
                None => {
                    let i = resource_bindings.len();
                    resource_bindings.push(GpuKernelResourceBinding {
                        group: 0,
                        binding: state.binding as usize,
                        input: None,
                        output: None,
                    });
                    resource_bindings.get_mut(i).unwrap()
                }
            };

            if state.input {
                inputs.push(slot);
                let input_id = input_types.len();
                input_types.push(state.ty);
                assert!(res.input.is_none());
                res.input = Some(input_id);
            }
            if state.output {
                outputs.push(slot);
                let output_id = output_types.len();
                output_types.push(state.ty);
                assert!(res.output.is_none());
                res.output = Some(output_id);
            }
        }

        let kernel = ffi::GpuKernel {
            // The name doesn't *really* matter, but we can uniquely identify each fused shader
            // by it's scheduling funclet ID and the starting node (fuse sequences are disjoint)
            name: format!("fused_funclet{}_node{}", funclet_id, self.start),
            input_types: input_types.into_boxed_slice(),
            output_types: output_types.into_boxed_slice(),
            entry_point: "main".to_owned(),
            resource_bindings: resource_bindings.into_boxed_slice(),
            shader_module,
        };
        ops.push(Opportunity {
            bounds: self.start..end,
            kernel,
            dimensions: self.workgroup_dimensions,
            inputs,
            outputs,
        });
    }
}

#[derive(Debug, Clone)]
struct DispatchInfo {
    /// The ID of the kernel we're dispatching.
    id: ir::ExternalFunctionId,
    /// IDs of the scheduling nodes providing the kernel dimensions, if they're assigned.
    workgroup_dimensions: [Option<ir::NodeId>; 3],
    /// IDs of the scheduling nodes used for the inputs.
    inputs: Vec<ir::NodeId>,
    /// IDs of the scheduling nodes used for the outputs.
    outputs: Vec<ir::NodeId>,
}

impl DispatchInfo {
    fn from_node(prog: &ir::Program, node: &ir::Node) -> Option<Self> {
        use ir::Node::{CallExternalGpuCompute, EncodeDo};
        let EncodeDo { operation, inputs, outputs, .. } = node else {
            return None;
        };
        let remote = &prog.funclets[operation.funclet_id].nodes[operation.node_id];
        let CallExternalGpuCompute { external_function_id, dimensions, .. } = remote else {
            return None;
        };
        let mut true_dims = [None; 3];
        for i in 0..dimensions.len() {
            true_dims[i] = Some(inputs[i]);
        }
        let true_inputs = Vec::from(&inputs[dimensions.len()..]);
        let true_outputs = Vec::from(&outputs[..]);
        return Some(Self {
            id: *external_function_id,
            workgroup_dimensions: true_dims,
            inputs: true_inputs,
            outputs: true_outputs,
        });
    }
}

/// # Panics
/// Panics if `funclet` is not a scheduling funclet.
pub fn identify_opportunities(
    prog: &ir::Program,
    funclet_id: ir::FuncletId,
    funclet: &ir::Funclet,
) -> Vec<Opportunity> {
    // Nothing goes wrong if we run this on a non-scheduling funclet.
    // But there is literally zero reason to run this on a non-scheduling funclet, so it's
    // probably a bug if it ever gets called on one...
    assert_eq!(ir::FuncletKind::ScheduleExplicit, funclet.kind);

    let mut ops = Vec::new();
    let mut state: Option<FuseState> = None;

    for (id, node) in funclet.nodes.iter().enumerate() {
        if let ir::Node::None = node {
            // Always ignore None nodes.
            continue;
        }

        // First: is this even a kernel?
        if let Some(dispatch) = DispatchInfo::from_node(prog, node) {
            // Alright, it's a kernel. Are we already fusing?
            if let Some(fs) = state.as_mut() {
                // We're already fusing. Can we add this?
                if fs.fuse(prog, dispatch.clone()) {
                    // Yes, we can fuse the current node into our existing dispatch!
                    continue;
                } else {
                    // Nope, the current node is incompatible for one reason or another.
                    // Finish our existing fusion sequence and restart
                    state.take().unwrap().finish(id, funclet_id, &mut ops);
                    state = Some(FuseState::new(prog, id, dispatch));
                }
            } else {
                // Ok, we're not already fusing. Let's start!
                state = Some(FuseState::new(prog, id, dispatch));
            }
        } else {
            // Alright, it's not a kernel. If we were in the middle of fusing, finish the job.
            if let Some(fs) = state.take() {
                fs.finish(id, funclet_id, &mut ops);
            }
        }
    }

    // Handle a fusion sequence which runs right off the end of the funclet
    if let Some(fs) = state.take() {
        fs.finish(funclet.nodes.len(), funclet_id, &mut ops);
    }

    return ops;
}
