use crate::ir;
use crate::rust_wgpu_backend::ffi;
use crate::shadergen::{FuseDescriptor, FuseSource, FusedResource, ShaderModule};

use std::collections::HashMap;
use std::ops::Range;

/// Criteria for automatic fusing:
///  1. Dispatches are sequential.
///     Currently, this means that they can only be separated by None nodes.
///  2. Dispatches form a dependency chain.
///     Otherwise, fusing may be a performance loss since the GPU can execute them in parallel.
///  3. Dispatches share workgroup dimensions and local sizes.
#[derive(Debug)]
pub struct Opportunity {
    /// The scheduling nodes which should be replaced by this kernel dispatch.
    bounds: Range<ir::NodeId>,
    /// The fused kernel.
    shader_module: ShaderModule,
    /// TODO: Write this explanation
    resources: HashMap<(usize, u32, u32), FusedResource>,
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
    slot_to_binding: HashMap<ir::OperationId, u32>,
}
impl FuseState {
    pub fn new(prog: &ir::Program, start: ir::NodeId, dispatch: DispatchInfo) -> FuseState {
        let kernel = match &prog.native_interface.external_functions[dispatch.id.0] {
            ffi::ExternalFunction::GpuKernel(gk) => gk,
            _ => panic!("kernel fusion: not a GPU kernel!")
        };

        let shader_module = match &kernel.shader_module_content {
            ffi::ShaderModuleContent::Wgsl(wgsl) => ShaderModule::from_wgsl(wgsl).unwrap(),
            ffi::ShaderModuleContent::Glsl(glsl) => ShaderModule::from_glsl(glsl).unwrap(),
        };
        let local_size = shader_module.local_size(&kernel.entry_point);

        let mut state = FuseState {
            start,
            workgroup_dimensions: dispatch.workgroup_dimensions,
            local_size,
            next_binding: 0,
            kernels: Vec::new(),
            resources: HashMap::new(),
            slot_to_binding: HashMap::new(),
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
        resource: &ffi::GpuKernelResourceBinding,
    ) {
        let binding = self.slot_to_binding.get(&node).copied().unwrap_or_else(|| {
            let old = self.next_binding;
            self.slot_to_binding.insert(node, old);
            self.next_binding += 1;
            old
        });
        self.resources.insert(
            (module_index, resource.group as u32, resource.binding as u32),
            FusedResource::Binding { group: 0, binding },
        );
    }

    pub fn fuse(&mut self, prog: &ir::Program, dispatch: DispatchInfo) -> bool {
        if self.workgroup_dimensions != dispatch.workgroup_dimensions {
            return false;
        }

        let kernel = match &prog.native_interface.external_functions[dispatch.id.0] {
            ffi::ExternalFunction::GpuKernel(gk) => gk,
            _ => panic!("kernel fusion: not a GPU kernel!")
        };

        let shader_module = match &kernel.shader_module_content {
            ffi::ShaderModuleContent::Wgsl(wgsl) => ShaderModule::from_wgsl(wgsl).unwrap(),
            ffi::ShaderModuleContent::Glsl(glsl) => ShaderModule::from_glsl(glsl).unwrap(),
        };

        if self.local_size != shader_module.local_size(&kernel.entry_point) {
            return false;
        }

        let module_index = self.kernels.len();
        let mut dependency_chain = false;

        // Optimistically update our resource map and slot to binding map, as if fusion would
        // succeed. It's not illegal to have *extra* elements in those maps!
        for (in_index, input) in dispatch.inputs.iter().enumerate() {
            let resource = kernel
                .resource_bindings
                .iter()
                .find(|r| r.input == Some(in_index))
                .expect("unknown input");
            dependency_chain |= self.slot_to_binding.contains_key(input);
            self.register(module_index, *input, resource);
        }

        for (output_index, output) in dispatch.outputs.iter().enumerate() {
            let resource = kernel
                .resource_bindings
                .iter()
                .find(|r| r.output == Some(output_index))
                .expect("unknown output");
            self.register(module_index, *output, resource);
        }

        // Update the module *if* any dependency chains were involved, or if this is the
        // first module, since otherwise we'd never make any progress.
        if (self.kernels.is_empty() || dependency_chain) {
            let module = (shader_module, kernel.entry_point.clone());
            self.kernels.push(module);
            return true;
        } else {
            return false;
        }
    }

    pub fn finish(self, end: ir::OperationId, ops: &mut Vec<Opportunity>) {
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
        let fused = ShaderModule::fuse(desc);
        ops.push(Opportunity {
            bounds: self.start..end,
            shader_module: fused,
            resources: self.resources,
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
pub fn identify_opportunities(prog: &ir::Program, funclet: &ir::Funclet) -> Vec<Opportunity> {
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
                    state.take().unwrap().finish(id, &mut ops);
                    state = Some(FuseState::new(prog, id, dispatch));
                }
            } else {
                // Ok, we're not already fusing. Let's start!
                state = Some(FuseState::new(prog, id, dispatch));
            }
        } else {
            // Alright, it's not a kernel. If we were in the middle of fusing, finish the job.
            if let Some(fs) = state.take() {
                fs.finish(id, &mut ops);
            }
        }
    }

    // Handle a fusion sequence which runs right off the end of the funclet
    if let Some(fs) = state.take() {
        fs.finish(funclet.nodes.len(), &mut ops);
    }

    return ops;
}
