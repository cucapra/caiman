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
pub struct Opportunity {
    /// The scheduling nodes which should be replaced by this kernel dispatch.
    bounds: Range<ir::NodeId>,
    /// The fused kernel.
    shader_module: ShaderModule,
    /// TODO: Write this explanation
    resources: HashMap<(usize, u32, u32), FusedResource>,
}

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
        let info = &prog.native_interface.external_gpu_functions[dispatch.id];
        let shader_module = match &info.shader_module_content {
            ffi::ShaderModuleContent::Wgsl(wgsl) => ShaderModule::from_wgsl(wgsl).unwrap(),
            ffi::ShaderModuleContent::Glsl(glsl) => ShaderModule::from_glsl(glsl).unwrap(),
        };
        let local_size = shader_module.local_size(&info.entry_point);

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
        resource: &ffi::ExternalGpuFunctionResourceBinding,
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

        let config = &prog.native_interface.external_gpu_functions[dispatch.id];
        let shader_module = match &config.shader_module_content {
            ffi::ShaderModuleContent::Wgsl(wgsl) => ShaderModule::from_wgsl(wgsl).unwrap(),
            ffi::ShaderModuleContent::Glsl(glsl) => ShaderModule::from_glsl(glsl).unwrap(),
        };

        if self.local_size != shader_module.local_size(&config.entry_point) {
            return false;
        }

        let module_index = self.kernels.len();
        let mut dependency_chain = false;

        // Optimistically update our resource map and slot to binding map, as if fusion would
        // succeed. It's not illegal to have *extra* elements in those maps!
        for (in_index, input) in dispatch.inputs.iter().enumerate() {
            let resource = config
                .resource_bindings
                .iter()
                .find(|r| r.input == Some(in_index))
                .expect("unknown input");
            dependency_chain |= self.slot_to_binding.contains_key(input);
            self.register(module_index, *input, resource);
        }

        for (output_index, output) in dispatch.outputs.iter().enumerate() {
            let resource = config
                .resource_bindings
                .iter()
                .find(|r| r.output == Some(output_index))
                .expect("unknown output");
            self.register(module_index, *output, resource);
        }

        // Update the module *if* any dependency chains were involved, or if this is the
        // first module, since otherwise we'd never make any progress.
        if (self.kernels.is_empty() || dependency_chain) {
            let module = (shader_module, config.entry_point.clone());
            self.kernels.push(module);
            return true;
        } else {
            return false;
        }
    }

    pub fn finish(self, end: ir::OperationId) -> Option<Opportunity> {
        if (end <= self.start + 1) {
            // We're not actually fusing anything...
            return None;
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
        return Some(Opportunity {
            bounds: self.start..end,
            shader_module: fused,
            resources: self.resources,
        });
    }
}

struct DispatchInfo {
    /// The ID of the kernel we're dispatching.
    id: ir::ExternalGpuFunctionId,
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

pub fn identify_opportunities(prog: &ir::Program, funclet: &ir::Funclet) -> Vec<Opportunity> {
    let mut ops = Vec::new();
    let mut state: Option<FuseState> = None;
    for (id, node) in funclet.nodes.iter().enumerate() {
        if let ir::Node::None = node {
            continue;
        }

        if let Some(dispatch) = DispatchInfo::from_node(prog, node) {
            if let Some(fs) = state.as_mut() {
                if fs.fuse(prog, dispatch) {
                    continue;
                }
                // Intentional fallthrough here...
            } else {
                state = Some(FuseState::new(prog, id, dispatch));
                continue;
            }
        }

        if let Some(op) = state.take().and_then(|fs| fs.finish(id)) {
            ops.push(op)
        }
    }
    return ops;
}
