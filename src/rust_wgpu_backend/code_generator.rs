use itertools::Itertools;

use super::ffi;
use crate::id_generator::IdGenerator;
use crate::ir;
use crate::rust_wgpu_backend::code_writer::CodeWriter;
use crate::shadergen;
use crate::shadergen::ShaderModule;
use crate::stable_vec::StableVec;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::default::Default;
use std::env::var;
use std::fmt::Write;

// The dependency on crate::ir is not good
// code_generator should be independent of the ir definition, but fixing it will take time

// Submissions represent groups of tasks that are executing in a logical sequence
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Default)]
pub struct SubmissionId(usize);

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Default)]
pub struct CommandBufferId(usize);

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Default, Hash)]
pub struct VarId(usize);

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Default)]
pub struct TypeId(usize);

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Default)]
pub struct FenceId(usize);

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Default, Hash)]
pub struct ClosureId(usize);

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Default, Hash)]
pub struct DispatcherId(usize);

#[derive(Debug, Default)]
pub struct YieldPoint {
    pub name: String,
    pub yielded_types: Box<[ffi::TypeId]>,
    pub resuming_types: Box<[ffi::TypeId]>,
}

#[derive(Debug, Default)]
struct SubmissionQueue {
    last_submission_id_opt: Option<SubmissionId>,
    next_submission_id: SubmissionId,
    next_fence_id: FenceId,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum VariableKind {
    Dead,
    Buffer,
    Fence,
    LocalData,
}

#[derive(Default)]
struct VariableTracker {
    id_generator: IdGenerator,
    variable_kinds: HashMap<VarId, VariableKind>,
    variable_types: HashMap<VarId, ffi::TypeId>,
    /// variables allocated via alloc_temporary
    allocated_temps: HashSet<VarId>,
    arguments: HashSet<VarId>,
}

impl VariableTracker {
    fn new() -> Self {
        Self {
            id_generator: IdGenerator::new(),
            variable_kinds: HashMap::<VarId, VariableKind>::new(),
            variable_types: HashMap::<VarId, ffi::TypeId>::new(),
            allocated_temps: HashSet::<VarId>::new(),
            arguments: HashSet::<VarId>::new(),
        }
    }

    fn generate(&mut self) -> VarId {
        VarId(self.id_generator.generate())
    }

    fn create(&mut self, kind: VariableKind, type_id: Option<ffi::TypeId>) -> VarId {
        let id = self.generate();
        self.variable_kinds.insert(id, kind);
        if let Some(type_id) = type_id {
            self.variable_types.insert(id, type_id);
        }
        id
    }

    fn create_local_data(&mut self, type_id: Option<ffi::TypeId>) -> VarId {
        let id = self.create(VariableKind::LocalData, type_id);
        id
    }

    /// Creates a local temporary variable on the CPU. Adds the temporary
    /// to the set of temporary variables the variable tracker is aware of.
    fn create_local_alloc(&mut self, type_id: Option<ffi::TypeId>) -> VarId {
        let id = self.create_local_data(type_id);
        assert!(!self.allocated_temps.contains(&id));
        self.allocated_temps.insert(id);
        id
    }

    /// Creates a local temporary variable on the GPU. Adds the temporary
    /// to the set of temporary variables the variable tracker is aware of.
    fn create_local_gpu(&mut self, type_id: Option<ffi::TypeId>) -> VarId {
        let id = self.create_buffer(type_id);
        assert!(!self.allocated_temps.contains(&id));
        self.allocated_temps.insert(id);
        id
    }

    fn create_buffer(&mut self, type_id: Option<ffi::TypeId>) -> VarId {
        let id = self.create(VariableKind::Buffer, type_id);
        id
    }

    fn create_fence(&mut self, type_id: Option<ffi::TypeId>) -> VarId {
        let id = self.create(VariableKind::Fence, type_id);
        id
    }

    /*fn get_type_id(&self, variable_id: VarId) -> ffi::TypeId {
        self.variable_types[&variable_id]
    }*/

    fn get_kind(&self, variable_id: VarId) -> VariableKind {
        self.variable_kinds[&variable_id]
    }

    // Returns if the variable was created by alloc temporary
    fn is_temp(&self, variable_id: VarId) -> bool {
        self.allocated_temps.contains(&variable_id)
    }

    fn is_arg(&self, variable_id: VarId) -> bool {
        self.arguments.contains(&variable_id)
    }

    fn new_arg(&mut self, variable_id: VarId, variable_type: ffi::TypeId) {
        self.arguments.insert(variable_id);
        self.variable_types.insert(variable_id, variable_type);
    }
}

enum Binding {
    Buffer(usize),
}

#[derive(Default)]
struct SubmissionEncodingState {
    command_buffer_ids: Vec<CommandBufferId>,
}

struct ActiveFuncletState {
    funclet_id: ir::FuncletId,
    result_type_ids: Box<[ffi::TypeId]>,
    next_funclet_ids: Option<Box<[ir::FuncletId]>>,
    capture_count: usize,
    output_count: usize,
    output_type_ids: Box<[ffi::TypeId]>,
    next_funclet_input_types: Option<Box<[Box<[ffi::TypeId]>]>>,
}

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Debug, Default)]
struct ShaderModuleKey {
    external_gpu_function_name: String,
}

impl ShaderModuleKey {
    fn instance_field_name(&self) -> String {
        format!(
            "external_gpu_function_{}_module",
            self.external_gpu_function_name
        )
    }
}

struct GpuFunctionInvocation {
    external_gpu_function_name: String,
    bindings: BTreeMap<usize, (Option<usize>, Option<usize>, bool)>,
    shader_module_key: ShaderModuleKey,
}

#[derive(Debug)]
struct Closure {
    capture_types: Box<[ir::ffi::TypeId]>,
    argument_types: Box<[ir::ffi::TypeId]>,
    closure_id: ClosureId,
    dispatcher_id: DispatcherId,
}

struct Dispatcher {
    dispatcher_id: DispatcherId,
}

pub struct CodeGenerator<'program> {
    type_code_writer: CodeWriter,
    state_code_writer: CodeWriter,
    code_writer: CodeWriter, // the "everything else" for now
    has_been_generated: HashSet<ffi::TypeId>,
    variable_tracker: VariableTracker,
    active_pipeline_name: Option<String>,
    active_funclet_result_type_ids: Option<Box<[ffi::TypeId]>>,
    active_funclet_state: Option<ActiveFuncletState>,
    active_submission_encoding_state: Option<SubmissionEncodingState>,
    active_external_gpu_function_name: Option<String>,
    active_shader_module_key: Option<ShaderModuleKey>,
    shader_modules: BTreeMap<ShaderModuleKey, shadergen::ShaderModule>,
    submission_queue: SubmissionQueue,
    next_command_buffer_id: CommandBufferId,
    gpu_function_invocations: Vec<GpuFunctionInvocation>,
    original_native_interface: &'program ffi::NativeInterface,
    native_interface: ffi::NativeInterface,
    active_closures: HashMap<(ir::FuncletId, usize), Closure>,
    closure_id_generator: IdGenerator,
    active_yield_point_ids: HashSet<ffi::ExternalFunctionId>,
    dispatcher_id_generator: IdGenerator,
    active_dispatchers: HashMap<Box<[ffi::TypeId]>, Dispatcher>,
    gpu_fence_type: Option<ffi::TypeId>,
}

impl<'program> CodeGenerator<'program> {
    pub fn new(native_interface: &'program ffi::NativeInterface) -> Self {
        let variable_tracker = VariableTracker::new();
        let type_code_writer = CodeWriter::new();
        let state_code_writer = CodeWriter::new();
        let code_writer = CodeWriter::new();
        let has_been_generated = HashSet::new();
        let mut code_generator = Self {
            original_native_interface: native_interface,
            native_interface: native_interface.clone(),
            type_code_writer,
            state_code_writer,
            code_writer,
            has_been_generated,
            variable_tracker,
            active_pipeline_name: None,
            active_funclet_result_type_ids: None,
            active_funclet_state: None,
            active_submission_encoding_state: None,
            active_external_gpu_function_name: None,
            active_shader_module_key: None,
            shader_modules: BTreeMap::new(),
            submission_queue: Default::default(),
            next_command_buffer_id: CommandBufferId(0),
            gpu_function_invocations: Vec::new(),
            active_closures: HashMap::new(),
            closure_id_generator: IdGenerator::new(),
            active_yield_point_ids: HashSet::new(),
            dispatcher_id_generator: IdGenerator::new(),
            active_dispatchers: HashMap::new(),
            gpu_fence_type: None,
        };

        code_generator.gpu_fence_type = Some(code_generator.create_ffi_type(ffi::Type::GpuFence));

        let type_ids = code_generator
            .native_interface
            .types
            .iter()
            .map(|(type_id, _)| ffi::TypeId(type_id))
            .collect::<Box<[ffi::TypeId]>>();
        for &type_id in type_ids.iter() {
            code_generator.generate_type_definition(type_id);
        }

        code_generator
    }

    pub fn finish(&mut self) -> String {
        self.write_states();
        self.type_code_writer.finish()
            + self.state_code_writer.finish().as_str()
            + self.code_writer.finish().as_str()
    }

    pub fn get_native_interface(&self) -> &ffi::NativeInterface {
        &self.native_interface
    }

    fn get_var_name(&self, variable_id: VarId) -> String {
        format!("var_{}", variable_id.0)
    }

    fn get_tuple_definition_string(&self, type_ids: &[ffi::TypeId]) -> String {
        let mut output_string = String::new();
        write!(output_string, "(");
        for (index, type_id) in type_ids.iter().enumerate() {
            let type_name = self.get_type_name_with_ref(*type_id, Some("callee"));
            write!(output_string, "{}, ", type_name);
        }
        write!(output_string, ")");
        output_string
    }

    fn generate_compute_dispatch_outputs(
        &mut self,
        external_function_id: ir::ExternalFunctionId,
    ) -> Box<[VarId]> {
        let mut output_vars = Vec::<VarId>::new();

        let external_gpu_function = self.native_interface.external_functions
            [external_function_id.0]
            .get_gpu_kernel()
            .unwrap();
        for (output_index, output_type_id) in external_gpu_function.output_types.iter().enumerate()
        {
            let variable_id = self.variable_tracker.create_buffer(Some(*output_type_id));
            output_vars.push(variable_id);
        }

        return output_vars.into_boxed_slice();
    }

    fn set_active_external_gpu_function(&mut self, kernel: &ffi::GpuKernel) {
        // Will need to be more careful with this check once modules no longer correspond to external gpu functions one to one
        // FIXME: Assumes every kernel has a distinct name
        if let Some(previous_name) = self.active_external_gpu_function_name.as_ref() {
            if previous_name == &kernel.name {
                return;
            }
        }

        self.active_external_gpu_function_name = None;

        let shader_module_key = ShaderModuleKey {
            external_gpu_function_name: kernel.name.clone(),
        };

        write!(
            self.code_writer,
            "let module = & instance.{};\n",
            shader_module_key.instance_field_name()
        );

        if !self.shader_modules.contains_key(&shader_module_key) {
            // TODO: Is this clone necessary?
            self.shader_modules
                .insert(shader_module_key.clone(), kernel.shader_module.clone());
        }

        self.active_external_gpu_function_name = Some(kernel.name.clone());
        self.active_shader_module_key = Some(shader_module_key);
    }

    fn set_active_bindings(
        &mut self,
        kernel: &ffi::GpuKernel,
        argument_vars: &[VarId],
        output_vars: &[VarId],
    ) {
        let active_kernel_name = self.active_external_gpu_function_name.as_ref().unwrap();
        assert_eq!(active_kernel_name, &kernel.name);

        let mut bindings =
            std::collections::BTreeMap::<usize, (Option<usize>, Option<usize>, bool)>::new();
        let mut output_binding_map = std::collections::BTreeMap::<usize, usize>::new();
        let mut input_binding_map = std::collections::BTreeMap::<usize, usize>::new();

        for resource_binding in kernel.resource_bindings.iter() {
            assert_eq!(resource_binding.group, 0);

            let mut rw_override = false;
            if let Some(input) = resource_binding.input {
                input_binding_map.insert(input, resource_binding.binding);
                let in_var = argument_vars[input];
            }

            if let Some(output) = resource_binding.output {
                output_binding_map.insert(output, resource_binding.binding);
                let out_var = output_vars[output];
            }

            bindings.insert(
                resource_binding.binding,
                (resource_binding.input, resource_binding.output, rw_override),
            );
        }

        let mut input_staging_variables = Vec::<VarId>::new();
        assert_eq!(argument_vars.len(), kernel.input_types.len());
        for input_index in 0..kernel.input_types.len() {
            let type_id = kernel.input_types[input_index];
            let input_variable_id = argument_vars[input_index];

            let binding = input_binding_map[&input_index];
            if let (_, Some(_output), _) = bindings[&binding] {
                input_staging_variables.push(input_variable_id);
            } else {
                input_staging_variables.push(input_variable_id);
            }
        }

        let mut output_staging_variables = Vec::<VarId>::new();
        for output_index in 0..kernel.output_types.len() {
            let binding = output_binding_map[&output_index];
            if let (Some(input), _, _) = bindings[&binding] {
                let variable_id = input_staging_variables[input];
                assert_eq!(variable_id, output_vars[output_index]);
                output_staging_variables.push(variable_id);
            } else {
                let type_id = kernel.output_types[output_index];
                let variable_id = output_vars[output_index];
                output_staging_variables.push(variable_id);
            }
        }

        let invocation_id = self.gpu_function_invocations.len();
        self.gpu_function_invocations.push(GpuFunctionInvocation {
            external_gpu_function_name: kernel.name.clone(),
            bindings,
            shader_module_key: self.active_shader_module_key.clone().unwrap(),
        });
        let gpu_function_invocation = &self.gpu_function_invocations[invocation_id];

        self.code_writer.write("let entries = [".to_string());
        for (binding, (input_opt, output_opt, rw_override)) in
            gpu_function_invocation.bindings.iter()
        {
            let mut variable_id: Option<VarId> = None;

            if let Some(input) = input_opt {
                variable_id = Some(input_staging_variables[*input]);
            }

            if let Some(output) = output_opt {
                variable_id = Some(output_staging_variables[*output]);
            }

            assert_eq!(
                variable_id.is_some(),
                true,
                "Binding must be input or output"
            );
            let variable_id = variable_id.unwrap();
            self.code_writer.write(format!(
                "wgpu::BindGroupEntry {{binding : {}, resource : {}.as_binding_resource() }}, ",
                binding,
                self.build_get_gpu_ref(variable_id, None)
            ));
        }
        self.code_writer.write("];\n".to_string());
        write!(self.code_writer, "let bind_group = instance.state.get_device_mut().create_bind_group(& wgpu::BindGroupDescriptor {{label : None, layout : & instance.static_bind_group_layout_{}, entries : & entries}});\n", invocation_id);
        write!(
            self.code_writer,
            "let pipeline = & instance.static_pipeline_{};\n",
            invocation_id
        );
    }

    fn begin_command_encoding(&mut self) {
        self.code_writer.write("let mut command_encoder = instance.state.get_device_mut().create_command_encoder(& wgpu::CommandEncoderDescriptor {label : None});\n".to_string());
    }

    fn end_command_encoding(&mut self) -> CommandBufferId {
        let command_buffer_id = self.next_command_buffer_id;
        self.next_command_buffer_id.0 += 1;
        self.code_writer.write(format!(
            "let command_buffer_{} = command_encoder.finish();\n",
            command_buffer_id.0
        ));
        return command_buffer_id;
    }

    fn enqueue_command_buffer(&mut self, command_buffer_id: CommandBufferId) {
        if self.active_submission_encoding_state.is_none() {
            self.active_submission_encoding_state = Some(Default::default());
        }

        if let Some(submission_encoding_state) = self.active_submission_encoding_state.as_mut() {
            submission_encoding_state
                .command_buffer_ids
                .push(command_buffer_id)
        }
    }

    fn reset_pipeline(&mut self) {
        self.active_external_gpu_function_name = None;
        self.active_shader_module_key = None;
    }

    fn generate_compute_dispatch(
        &mut self,
        kernel: &ffi::GpuKernel,
        dimension_vars: &[VarId; 3],
        argument_vars: &[VarId],
        output_vars: &[VarId],
    ) {
        let mut rw_bindings = HashSet::new();
        /*for rb in kernel.resource_bindings.iter() {
            if let Some(input) = rb.input {
                let in_var = argument_vars[input];
                if let Some(in_buf) = placement_state.get_var_buffer_id(in_var) {
                    if any_writes.contains(&in_buf) {
                        rw_bindings.insert((0u32, rb.binding as u32));
                    }
                }
            }
            if let Some(output) = rb.output {
                rw_bindings.insert((0u32, rb.binding as u32));
            }
        }*/

        // HACK: We need to fix up the readwrite specifiers on shader bindings to account for the
        // actual buffer usage pattern
        let mut shader_module = kernel.shader_module.clone();
        shader_module.force_writable_bindings(&rw_bindings);
        let kernel = ffi::GpuKernel {
            name: kernel.name.clone(),
            dimensionality: kernel.dimensionality,
            input_types: kernel.input_types.clone(),
            output_types: kernel.output_types.clone(),
            entry_point: kernel.entry_point.clone(),
            resource_bindings: kernel.resource_bindings.clone(),
            shader_module,
        };

        self.set_active_external_gpu_function(&kernel);
        self.set_active_bindings(&kernel, argument_vars, output_vars);

        self.begin_command_encoding();

        assert_eq!(kernel.input_types.len(), argument_vars.len());
        // self.code_writer.write(format!("let ("));
        // for output_index in 0..kernel.output_types.len() {
        //     let var_id = output_vars[output_index];
        //     self.code_writer
        //         .write(format!("{}, ", self.get_var_name(var_id),));
        // }
        // self.code_writer.write(format!(") = "));

        // self.code_writer.write("{\n".to_string());

        self.code_writer.write_str("{\n");
        self.code_writer.write("let mut compute_pass = command_encoder.begin_compute_pass(& wgpu::ComputePassDescriptor {label : None});\n".to_string());
        self.code_writer
            .write("compute_pass.set_pipeline(& pipeline);\n".to_string());
        self.code_writer
            .write("compute_pass.set_bind_group(0, & bind_group, & []);\n".to_string());
        self.code_writer.write(format!(
            "compute_pass.dispatch_workgroups(({}).try_into().unwrap(), \
        ({}).try_into().unwrap(), \
        ({}).try_into().unwrap());\n",
            self.access_val_str(dimension_vars[0]),
            self.access_val_str(dimension_vars[1]),
            self.access_val_str(dimension_vars[2])
        ));
        // self.code_writer.write_str("}\n");

        // let mut output_temp_variables = Vec::<VarId>::new();
        // for output_index in 0..kernel.output_types.len() {
        //     let staging_var_id = output_vars[output_index];
        //     let type_id = kernel.output_types[output_index];
        //     let range_var_id = self.variable_tracker.generate();
        //     let output_temp_var_id = self.variable_tracker.generate();
        //     let slice_var_id = self.variable_tracker.generate();
        //     let future_var_id = self.variable_tracker.generate();
        //     let type_binding_info = self.get_type_binding_info(type_id);
        //     let type_name = self.get_type_name(type_id);

        //     output_temp_variables.push(staging_var_id);
        // }

        // self.code_writer.write(format!("("));
        // for output_temp_var_id in output_temp_variables.iter() {
        //     self.code_writer
        //         .write(format!("{}, ", self.get_var_name(*output_temp_var_id)));
        // }
        // self.code_writer.write(format!(")"));

        self.code_writer.write("}\n".to_string());

        let command_buffer_id = self.end_command_encoding();
        self.enqueue_command_buffer(command_buffer_id);
    }

    pub fn flush_submission(&mut self) -> SubmissionId {
        let mut active_submission_encoding_state = None;
        std::mem::swap(
            &mut self.active_submission_encoding_state,
            &mut active_submission_encoding_state,
        );

        let submission_id = self.submission_queue.next_submission_id;

        if let Some(submission_encoding_state) = active_submission_encoding_state {
            if submission_encoding_state.command_buffer_ids.len() > 0 {
                write!(
                    self.code_writer,
                    "let submission_index_{} = instance.state.get_queue_mut().submit([",
                    submission_id.0
                );
                for &command_buffer_id in submission_encoding_state.command_buffer_ids.iter() {
                    self.code_writer
                        .write(format!("command_buffer_{}, ", command_buffer_id.0));
                }
                self.code_writer.write("]);\n".to_string());
            }
        }

        self.submission_queue.next_submission_id.0 += 1;
        self.submission_queue.last_submission_id_opt = Some(submission_id);

        submission_id
    }

    pub fn sync_submission(&mut self, submission_id: SubmissionId) {
        //self.code_writer.write("futures::executor::block_on(queue.on_submitted_work_done());\n".to_string());
        //self.code_writer.write(format!("instance.state.get_device_mut().poll(wgpu::Maintain::Wait);\n"));
        //self.code_writer.write(format!("futures::executor::block_on(future_var_{});\n", submission_id.0));
        //self.code_writer.write("futures::executor::block_on(queue.on_submitted_work_done());\n".to_string());
    }

    pub fn encode_gpu_fence(&mut self) -> VarId {
        let fence_id = self.submission_queue.next_fence_id;
        self.submission_queue.next_fence_id.0 += 1;

        let recv_var_id = self
            .variable_tracker
            .create_fence(Some(self.gpu_fence_type.unwrap()));
        write!(
            self.code_writer,
            "let {} = Some(submission_index_{});\n",
            self.get_var_name(recv_var_id),
            self.submission_queue.last_submission_id_opt.unwrap().0
        );

        recv_var_id
    }

    pub fn sync_gpu_fence(&mut self, recv_var_id: VarId) {
        write!(self.code_writer, "instance.state.get_device_mut().poll(if let Some(id) = {} {{ wgpu::Maintain::WaitForSubmissionIndex(id) }} else {{ wgpu::Maintain::Wait }});\n", self.get_var_name(recv_var_id));
    }

    pub fn insert_comment(&mut self, comment_string: &str) {
        self.code_writer.write(format!("// {}\n", comment_string));
    }

    fn write_states(&mut self) {
        let code_string = "
		/*pub struct CpuFunctionInvocationState<'parent>
		{
			parent_state : & 'parent mut dyn caiman_rt::State
		}*/
";

        write!(self.state_code_writer, "{}", code_string);
    }

    pub fn begin_pipeline(&mut self, pipeline_name: &str) {
        self.reset_pipeline();
        self.active_closures.clear();
        self.active_yield_point_ids.clear();
        self.active_dispatchers.clear();

        self.active_pipeline_name = Some(String::from(pipeline_name));
        self.code_writer.begin_module(pipeline_name);
        write!(
            self.code_writer,
            "use caiman_rt::{{LocalVars, GpuLocals, wgpu, bytemuck}};\n"
        );
        write!(self.code_writer, "use std::marker::PhantomData;\n");

        self.code_writer.begin_module("outputs");
        {
            for (_, external_cpu_function) in self.native_interface.external_functions.iter() {
                let mut tuple_fields = Vec::<ffi::TypeId>::new();
                if let Some(cpu_operation) = external_cpu_function.get_cpu_pure_operation() {
                    for (output_index, output_type) in cpu_operation.output_types.iter().enumerate()
                    {
                        tuple_fields.push(*output_type);
                    }

                    write!(
                        self.code_writer,
                        "pub type {} = {};\n",
                        cpu_operation.name,
                        self.get_tuple_definition_string(tuple_fields.as_slice())
                    );
                }
            }
        }
        self.code_writer.end_module();

        self.code_writer
            .write(format!("pub trait CpuFunctions\n{{\n"));
        for (_, external_cpu_function) in self.native_interface.external_functions.iter() {
            if let Some(cpu_operation) = external_cpu_function.get_cpu_pure_operation() {
                self.code_writer.write(format!(
                    "\tfn {}(&self, state : &mut caiman_rt::State",
                    cpu_operation.name
                ));
                for (input_index, input_type) in cpu_operation.input_types.iter().enumerate() {
                    //self.generate_type_definition(* input_type);
                    self.code_writer
                        .write(format!(", _ : {}", self.get_type_name(*input_type)));
                }
                self.code_writer
                    .write(format!(") -> outputs::{};\n", cpu_operation.name));
            }
        }
        self.code_writer.write(format!("}}\n"));
    }

    pub fn begin_funclet(
        &mut self,
        funclet_id: ir::FuncletId,
        input_types: &[ffi::TypeId],
        output_types: &[ffi::TypeId],
    ) -> Box<[VarId]> {
        // Temporarily need to do this until pipelines are constructed correctly
        self.reset_pipeline();

        let funclet_result_type_ids = {
            let mut tuple_fields = Vec::<ffi::TypeId>::new();
            for output_index in 0..output_types.len() {
                let output_type = output_types[output_index];
                tuple_fields.push(output_type);
            }

            tuple_fields.into_boxed_slice()
        };

        self.active_funclet_result_type_ids = Some(funclet_result_type_ids.clone());

        let mut next_trait_index = 0usize;

        let mut argument_variable_ids = Vec::<VarId>::new();
        write!(self.code_writer, "fn funclet{}_func<'state,  'cpu_functions, 'callee, Callbacks : CpuFunctions>(mut instance : Instance<'state, 'cpu_functions, Callbacks>, join_stack : &mut caiman_rt::JoinStack<'callee>", funclet_id);

        let mut inputs = Vec::new();
        for (input_index, input_type) in input_types.iter().enumerate() {
            write!(self.code_writer, ", ");

            let variable_id = self.variable_tracker.create_local_data(Some(*input_type));
            self.variable_tracker
                .new_arg(variable_id, input_type.clone());
            argument_variable_ids.push(variable_id);
            let type_name = self.get_type_name_with_ref(*input_type, Some("callee"));
            let is_mutable = match &self.native_interface.types[input_type.0] {
                ffi::Type::GpuBufferAllocator => true,
                _ => false,
            };
            inputs.push((variable_id, input_type));
            if is_mutable {
                self.code_writer.write(format!(
                    "mut {} : {}",
                    self.get_var_name(variable_id),
                    type_name
                ));
            } else {
                self.code_writer.write(format!(
                    "{} : {}",
                    self.get_var_name(variable_id),
                    type_name
                ));
            }
        }

        write!(
            self.code_writer,
            " ) -> FuncletResult<'state, 'cpu_functions, 'callee, Callbacks, PipelineOutputTuple<'callee>>",
        );
        self.code_writer
            .write("\n{\n\tuse std::convert::TryInto;\n".to_string());

        self.active_funclet_state = Some(ActiveFuncletState {
            funclet_id,
            result_type_ids: funclet_result_type_ids,
            next_funclet_ids: None,
            capture_count: 0,
            output_count: 0,
            output_type_ids: output_types.to_vec().into_boxed_slice(),
            next_funclet_input_types: None,
        });

        argument_variable_ids.into_boxed_slice()
    }

    fn emit_pipeline_entry_point(
        &mut self,
        funclet_id: ir::FuncletId,
        input_types: &[ffi::TypeId],
        output_types: &[ffi::TypeId],
        yield_points_opt: Option<&[(ffi::ExternalFunctionId, YieldPoint)]>,
    ) {
        let pipeline_name = self.active_pipeline_name.as_ref().unwrap();

        let funclet_result_definition_string = "
		pub struct FuncletResult<'state, 'cpu_functions, 'callee, Callbacks : CpuFunctions, Intermediates>
		{
			instance : Instance<'state, 'cpu_functions, Callbacks>,
			phantom : std::marker::PhantomData<& 'callee ()>,
			intermediates : FuncletResultIntermediates<Intermediates>
		}

		impl<'state, 'cpu_functions, 'callee, Callbacks : CpuFunctions, Intermediates> FuncletResult<'state, 'cpu_functions, 'callee, Callbacks, Intermediates>
		{
			pub fn returned(&self) -> Option<& Intermediates>
			{
				if let FuncletResultIntermediates::Return(intermediates) = & self.intermediates
				{
					return Some(& intermediates);
				}

				None
			}

			pub fn prepare_next(self) -> Instance<'state, 'cpu_functions, Callbacks>
			{
				self.instance
			}
		}

		";

        write!(self.code_writer, "{}", funclet_result_definition_string);

        let pipeline_output_tuple_string = self.get_tuple_definition_string(output_types);
        write!(
            self.code_writer,
            "type PipelineOutputTuple<'callee> = {};\n",
            pipeline_output_tuple_string
        );

        write!(
            self.code_writer,
            "enum FuncletResultIntermediates<Intermediates>\n{{ Return(Intermediates), "
        );
        let mut yield_point_ref_map = HashMap::<ffi::ExternalFunctionId, &YieldPoint>::new();
        if let Some(yield_points) = yield_points_opt {
            for (yield_point_id, yield_point) in yield_points.iter() {
                yield_point_ref_map.insert(*yield_point_id, yield_point);
                write!(
                    self.code_writer,
                    "Yield{}{{ yielded : {} }}, ",
                    yield_point_id.0,
                    self.get_tuple_definition_string(&yield_point.yielded_types)
                );
            }
        }
        write!(self.code_writer, "}}");

        write!(self.code_writer, "impl<'state, 'cpu_functions, 'callee, Callbacks : CpuFunctions, Intermediates> FuncletResult<'state, 'cpu_functions, 'callee, Callbacks, Intermediates>\n{{\n");
        if let Some(yield_points) = yield_points_opt {
            for (yield_point_id, yield_point) in yield_points.iter() {
                write!(self.code_writer, "pub fn yielded_at_{}(&self) -> Option<& {}> {{ if let FuncletResultIntermediates::Yield{}{{yielded}} = & self.intermediates {{ Some(yielded) }} else {{ None }} }}\n", yield_point.name, self.get_tuple_definition_string(& yield_point.yielded_types), yield_point_id.0);
            }
        }
        write!(self.code_writer, "}}");

        // Write the instance state
        write!(
            self.code_writer,
            "pub struct Instance<'state, 'cpu_functions, F : CpuFunctions>{{\
            state : & 'state mut dyn caiman_rt::State, cpu_functions : & 'cpu_functions F, \
            locals: LocalVars, glocals: GpuLocals"
        );

        for (shader_module_key, shader_module) in self.shader_modules.iter() {
            write!(
                self.code_writer,
                ", {} : wgpu::ShaderModule",
                shader_module_key.instance_field_name()
            );
        }

        for (gpu_function_invocation_id, gpu_function_invocation) in
            self.gpu_function_invocations.iter().enumerate()
        {
            write!(self.code_writer, ", static_bind_group_layout_{} : wgpu::BindGroupLayout, static_pipeline_layout_{} : wgpu::PipelineLayout, static_pipeline_{} : wgpu::ComputePipeline", gpu_function_invocation_id, gpu_function_invocation_id, gpu_function_invocation_id);
        }

        write!(self.code_writer, "}}\n");

        write!(
            self.code_writer,
            "{}",
            "
		impl<'state, 'cpu_functions, F : CpuFunctions> Instance<'state, 'cpu_functions, F>
		{
			pub fn new(state : & 'state mut dyn caiman_rt::State, cpu_functions : & 'cpu_functions F) -> Self
			{
				"
        );

        for (shader_module_key, shader_module) in self.shader_modules.iter_mut() {
            write!(self.code_writer, "let {} = state.get_device_mut().create_shader_module(wgpu::ShaderModuleDescriptor {{ label : None, source : wgpu::ShaderSource::Wgsl(std::borrow::Cow::from(\"{}\"))}});\n", shader_module_key.instance_field_name(), shader_module.emit_wgsl().as_str());
        }

        for (gpu_function_invocation_id, gpu_function_invocation) in
            self.gpu_function_invocations.iter().enumerate()
        {
            self.code_writer
                .write("let bind_group_layout_entries = [".to_string());
            for (binding, (_input_opt, output_opt, rw_override)) in
                gpu_function_invocation.bindings.iter()
            {
                let is_read_only: bool = output_opt.is_none() && !rw_override;
                self.code_writer
                    .write("wgpu::BindGroupLayoutEntry { ".to_string());
                self.code_writer.write(format!("binding : {}, visibility : wgpu::ShaderStages::COMPUTE, ty : wgpu::BindingType::Buffer{{ ty : wgpu::BufferBindingType::Storage {{ read_only : {} }}, has_dynamic_offset : false, min_binding_size : None}}, count : None", binding, is_read_only));
                self.code_writer.write(" }, ".to_string());
            }
            self.code_writer.write("];\n".to_string());

            write!(self.code_writer, "let static_bind_group_layout_{} = state.get_device_mut().create_bind_group_layout(& wgpu::BindGroupLayoutDescriptor {{ label : None, entries : & bind_group_layout_entries}});\n", gpu_function_invocation_id);

            write!(self.code_writer, "let static_pipeline_layout_{} = state.get_device_mut().create_pipeline_layout(& wgpu::PipelineLayoutDescriptor {{ label : None, bind_group_layouts : & [& static_bind_group_layout_{}], push_constant_ranges : & []}});\n", gpu_function_invocation_id, gpu_function_invocation_id);
            write!(self.code_writer, "let static_pipeline_{} = state.get_device_mut().create_compute_pipeline(& wgpu::ComputePipelineDescriptor {{label : None, layout : Some(& static_pipeline_layout_{}), module : & {}, entry_point : & \"main\"}});\n", gpu_function_invocation_id, gpu_function_invocation_id, gpu_function_invocation.shader_module_key.instance_field_name());
        }

        write!(
            self.code_writer,
            "{}",
            "
				Self{locals: LocalVars::new(), glocals: GpuLocals::new(state), state, cpu_functions"
        );

        for (shader_module_key, shader_module) in self.shader_modules.iter() {
            write!(
                self.code_writer,
                ", {}",
                shader_module_key.instance_field_name()
            );
        }

        for (gpu_function_invocation_id, gpu_function_invocation) in
            self.gpu_function_invocations.iter().enumerate()
        {
            write!(
                self.code_writer,
                ", static_bind_group_layout_{}, static_pipeline_layout_{}, static_pipeline_{}",
                gpu_function_invocation_id, gpu_function_invocation_id, gpu_function_invocation_id
            );
        }

        write!(
            self.code_writer,
            "{}",
            "}
			}

		"
        );

        write!(
            self.code_writer,
            "{}",
            "
		}
		"
        );

        write!(
            self.code_writer,
            "impl<'state, 'cpu_functions, F : CpuFunctions> Instance<'state, 'cpu_functions, F>\n"
        );
        write!(self.code_writer, "{{\n");
        {
            write!(
                self.code_writer,
                "pub fn start<'callee>(mut self, join_stack : &mut caiman_rt::JoinStack<'callee>"
            );
            for (input_index, input_type) in input_types.iter().enumerate() {
                write!(
                    self.code_writer,
                    ", arg_{} : {}",
                    input_index,
                    self.get_type_name(*input_type)
                );
            }
            write!(self.code_writer, ") -> FuncletResult<'state, 'cpu_functions, 'callee, F, PipelineOutputTuple<'callee>> {{\n");
            for (input_index, input_type) in input_types.iter().enumerate() {
                if self.is_cpu_ref(*input_type) {
                    write!(
                        self.code_writer,
                        "self.locals.calloc({}, *arg_{});\n",
                        input_index, input_index
                    );
                }
            }
            write!(
                self.code_writer,
                "let r = funclet{}_func(self, join_stack",
                funclet_id
            );
            for (input_index, input_type) in input_types.iter().enumerate() {
                if self.is_cpu_ref(*input_type) {
                    write!(self.code_writer, ", StackRef::local({input_index})");
                } else {
                    write!(self.code_writer, ", arg_{input_index}");
                }
            }
            write!(self.code_writer, ");\n",);
            for (input_index, input_type) in input_types.iter().enumerate() {
                if self.is_cpu_mut_ref(*input_type) {
                    write!(
                        self.code_writer,
                        "*arg_{input_index} = *r.instance.locals.get({input_index});\n",
                    );
                }
            }
            write!(self.code_writer, "r }}");
        }
        if let Some(yield_points) = yield_points_opt {
            for (yield_point_id, yield_point) in yield_points.iter() {
                let dispatcher_id = self.lookup_dispatcher_id(&yield_point.resuming_types);
                write!(self.code_writer, "pub fn resume_at_{}<'callee>(self, join_stack : &mut caiman_rt::JoinStack<'callee>", yield_point.name);
                for (resuming_argument_index, resuming_type) in
                    yield_point.resuming_types.iter().enumerate()
                {
                    write!(
                        self.code_writer,
                        ", arg_{} : {}",
                        resuming_argument_index,
                        self.get_type_name(*resuming_type)
                    );
                }
                write!(self.code_writer, ") -> FuncletResult<'state, 'cpu_functions, 'callee, F, PipelineOutputTuple<'callee>> {{ pop_join_and_dispatch_at_{}::<F, PipelineOutputTuple<'callee>>(join_stack", dispatcher_id.0);
                for (resuming_argument_index, resuming_type) in
                    yield_point.resuming_types.iter().enumerate()
                {
                    write!(self.code_writer, ", arg_{}", resuming_argument_index);
                }
                write!(self.code_writer, ", self) }}\n");
            }
        }
        write!(self.code_writer, "}}\n");

        // Generate closures all the way at the end

        write!(
            self.code_writer,
            "#[derive(Debug)] enum ClosureHeader {{ Root, "
        );
        for ((funclet_id, capture_count), closure) in self.active_closures.iter() {
            write!(
                self.code_writer,
                "Funclet{}Capturing{}, ",
                funclet_id, capture_count
            );
        }
        write!(self.code_writer, "}}\n");

        for ((funclet_id, capture_count), closure) in self.active_closures.iter() {
            write!(
                self.code_writer,
                "type Funclet{}Capturing{}CapturedTuple<'callee> = {};\n",
                funclet_id,
                capture_count,
                self.get_tuple_definition_string(&closure.capture_types)
            );
        }

        for (argument_types, dispatcher) in self.active_dispatchers.iter() {
            write!(self.code_writer, "fn pop_join_and_dispatch_at_{}<'state, 'cpu_functions, 'callee, Callbacks : CpuFunctions, Intermediates>(join_stack : &mut caiman_rt::JoinStack<'callee>", dispatcher.dispatcher_id.0);

            for (resuming_argument_index, resuming_type) in argument_types.iter().enumerate() {
                write!(
                    self.code_writer,
                    ", arg_{} : {}",
                    resuming_argument_index,
                    self.get_type_name_with_ref(*resuming_type, Some("callee"))
                );
            }
            write!(
                self.code_writer,
                ", mut instance : Instance<'state, 'cpu_functions, Callbacks>"
            );
            write!(
                self.code_writer,
                " ) -> FuncletResult<'state, 'cpu_functions, 'callee, Callbacks, {}>\n",
                pipeline_output_tuple_string
            );
            write!(self.code_writer, "{{\n",);

            write!(self.code_writer, "let closure_header = unsafe {{ join_stack.pop_unsafe_unaligned::<ClosureHeader>().unwrap() }}; match closure_header {{\n",);

            for ((funclet_id, capture_count), closure) in self.active_closures.iter() {
                if closure.dispatcher_id != dispatcher.dispatcher_id {
                    continue;
                }

                write!(self.code_writer, "ClosureHeader::Funclet{}Capturing{} => {{ let join_captures = unsafe {{ join_stack.pop_unsafe_unaligned::<Funclet{}Capturing{}CapturedTuple<'callee>>().unwrap() }}; funclet{}_func(instance, join_stack", funclet_id, capture_count, funclet_id, capture_count, funclet_id);
                for capture_index in 0..*capture_count {
                    write!(self.code_writer, ", join_captures.{}", capture_index);
                }
                for (argument_index, _argument_type) in argument_types.iter().enumerate() {
                    write!(self.code_writer, ", arg_{}", argument_index);
                }
                write!(self.code_writer, ") }}\n");
            }

            write!(self.code_writer, "_ => panic!(\"Dispatcher cannot dispatch given closure {{:?}}\", closure_header), }} }}", );
        }
        let s = r#"
        #[derive(Debug, Copy, Clone)]
        enum StackRef<T> {
            Local(usize, PhantomData<T>),
        }
        impl<T: 'static> StackRef<T> {
            fn get<'state, 'cpu_functions, 'a, F: CpuFunctions>(
                &self,
                instance: &'a Instance<'state, 'cpu_functions, F>,
            ) -> &'a T {
                match self {
                    StackRef::Local(index, ..) => instance.locals.get::<T>(*index),
                }
            }
    
            fn get_mut<'state, 'cpu_functions, 'a, F: CpuFunctions>(
                &self,
                instance: &'a mut Instance<'state, 'cpu_functions, F>,
            ) -> &'a mut T {
                match self {
                    StackRef::Local(index, ..) => instance.locals.get_mut::<T>(*index),
                }
            }
    
            fn local(index: usize) -> Self {
                StackRef::Local(index, PhantomData)
            }
        }"#;
        write!(self.code_writer, "{s}");
    }

    pub fn emit_oneshot_pipeline_entry_point(
        &mut self,
        funclet_id: ir::FuncletId,
        input_types: &[ffi::TypeId],
        output_types: &[ffi::TypeId],
    ) {
        self.emit_pipeline_entry_point(funclet_id, input_types, output_types, None)
    }

    pub fn emit_yieldable_pipeline_entry_point(
        &mut self,
        funclet_id: ir::FuncletId,
        input_types: &[ffi::TypeId],
        output_types: &[ffi::TypeId],
        yield_points: &[(ffi::ExternalFunctionId, YieldPoint)],
    ) {
        self.emit_pipeline_entry_point(funclet_id, input_types, output_types, Some(yield_points))
    }

    pub fn build_indirect_stack_jump_to_popped_serialized_join<'a>(
        &'a mut self,
        argument_var_ids: &[VarId],
        mut argument_types: &'a [ffi::TypeId],
    ) {
        let dispatcher_id = self.lookup_dispatcher_id(argument_types);
        write!(
            self.code_writer,
            "return pop_join_and_dispatch_at_{}::<Callbacks, PipelineOutputTuple<'callee>>",
            dispatcher_id.0
        );
        write!(self.code_writer, "(join_stack");
        for ((argument_index, var_id), var_type) in argument_var_ids
            .iter()
            .enumerate()
            .zip(argument_types.iter())
        {
            if self.is_cpu_ref(*var_type) {
                write!(self.code_writer, ", {}", self.make_stack_ref(*var_id));
            } else if self.is_gpu_buf(*var_type) {
                write!(self.code_writer, ", {}", self.access_gpu_str(*var_id));
            } else {
                write!(self.code_writer, ", {}", self.access_val_str(*var_id));
            }
        }
        write!(self.code_writer, ", instance);\n");
    }

    /// Builds a return statement for the current funclet
    /// # Arguments
    /// * `output_var_ids` - The variable ids of the variables being returned
    /// * `output_var_types` - The types of the variables being returned
    /// * `may_return` - Whether or not the funclet may return.
    ///    If false, the generated code will panic if there is nothing to pop from
    ///    the join stack.
    pub fn build_return(
        &mut self,
        output_var_ids: &[VarId],
        output_var_types: &[ffi::TypeId],
        may_return: bool,
    ) {
        if let Some(result_type_ids) = &self.active_funclet_result_type_ids {
            let result_type_ids = result_type_ids.clone(); // Make a copy for now to satisfy the borrowchecking gods...
            let dispatcher_id = self.lookup_dispatcher_id(&output_var_types);
            write!(self.code_writer, "if join_stack.used_bytes().len() > 0 {{ ");
            write!(
                self.code_writer,
                "return pop_join_and_dispatch_at_{}::<Callbacks, PipelineOutputTuple<'callee>>",
                dispatcher_id.0
            );
            write!(self.code_writer, "(join_stack");
            for ((return_index, var_id), var_type) in output_var_ids
                .iter()
                .enumerate()
                .zip(output_var_types.iter())
            {
                if self.is_cpu_ref(*var_type) {
                    write!(self.code_writer, ", {}", self.make_stack_ref(*var_id));
                } else if self.is_gpu_buf(*var_type) {
                    write!(self.code_writer, ", {}", self.access_gpu_str(*var_id));
                } else {
                    write!(self.code_writer, ", {}", self.access_val_str(*var_id));
                }
            }
            write!(self.code_writer, ", instance) }}");
            if may_return {
                write!(self.code_writer, "return FuncletResult::<'state, 'cpu_functions, 'callee, Callbacks, _> {{phantom : std::marker::PhantomData::<& 'callee ()>, intermediates : FuncletResultIntermediates::<_>::Return((");
                for ((return_index, var_id), var_type) in output_var_ids
                    .iter()
                    .enumerate()
                    .zip(result_type_ids.iter())
                {
                    if self.is_cpu_ref(*var_type) {
                        if self.variable_tracker.is_arg(*var_id) {
                            write!(self.code_writer, "{}, ", self.get_var_name(*var_id));
                        } else {
                            write!(self.code_writer, "StackRef::local({}) ", var_id.0);
                        }
                    } else {
                        assert!(!self.is_gpu_buf(*var_type));
                        write!(self.code_writer, "{}, ", self.access_val_str(*var_id));
                    }
                }
                write!(self.code_writer, ")), instance}};");
            } else {
                write!(
                    self.code_writer,
                    "else {{ panic!(\"Corrupted join stack\"); }}"
                );
            }
        } else {
            panic!("Can this happen?");
        }
    }

    pub fn build_yield(
        &mut self,
        yield_point_id: ffi::ExternalFunctionId,
        yielded_var_ids: &[VarId],
    ) {
        write!(
            self.code_writer,
            "instance.locals.reset();\ninstance.glocals.reset();\n"
        );
        write!(self.code_writer, "return FuncletResult::<'state, 'cpu_functions, 'callee, Callbacks, _> {{phantom : std::marker::PhantomData::<& 'callee ()>, intermediates : FuncletResultIntermediates::<_>::Yield{}{{ yielded : (", yield_point_id.0);
        for (return_index, var_id) in yielded_var_ids.iter().enumerate() {
            write!(self.code_writer, "{}, ", self.get_var_name(*var_id));
        }
        write!(self.code_writer, ") }}, instance }};");
    }

    pub fn end_funclet(&mut self) {
        self.code_writer.write("}\n".to_string());

        self.active_funclet_result_type_ids = None;
        self.active_funclet_state = None;
    }

    pub fn end_pipeline(&mut self) {
        self.code_writer.end_module();
        self.active_pipeline_name = None;
        self.reset_pipeline();
    }

    fn generate_type_definition(&mut self, type_id: ffi::TypeId) {
        if self.has_been_generated.contains(&type_id) {
            return;
        }

        self.has_been_generated.insert(type_id);

        let typ = &self.native_interface.types[type_id.0];
        write!(self.type_code_writer, "// Type #{}: {:?}\n", type_id.0, typ);
        match typ {
            ffi::Type::F32 => (),
            ffi::Type::F64 => (),
            ffi::Type::U8 => (),
            ffi::Type::U16 => (),
            ffi::Type::U32 => (),
            ffi::Type::U64 => (),
            ffi::Type::USize => (),
            ffi::Type::I8 => (),
            ffi::Type::I16 => (),
            ffi::Type::I32 => (),
            ffi::Type::I64 => (),
            ffi::Type::ConstRef { element_type } => (),
            ffi::Type::MutRef { element_type } => (),
            ffi::Type::ConstSlice { element_type } => (),
            ffi::Type::MutSlice { element_type } => (),
            ffi::Type::Array {
                element_type,
                length,
            } => (),
            ffi::Type::Tuple { fields } => {
                write!(self.type_code_writer, "pub type type_{} = (", type_id.0);
                for (index, field_type_id) in fields.iter().enumerate() {
                    let type_name = self.get_type_name(*field_type_id);
                    write!(self.type_code_writer, "{}, ", type_name);
                }
                self.type_code_writer.write_str(");\n");
            }
            ffi::Type::Struct {
                fields,
                byte_alignment,
                byte_size,
            } => {
                write!(self.type_code_writer, "pub struct type_{}", type_id.0);
                self.type_code_writer.write_str("{\n");
                for field in fields.iter() {
                    let type_name = self.get_type_name(field.type_id);
                    write!(
                        self.type_code_writer,
                        "\tpub {} : {},\n",
                        field.name, type_name
                    );
                }
                self.type_code_writer.write_str("}\n\n");
            }
            ffi::Type::GpuBufferRef { element_type } => (),
            ffi::Type::GpuBufferSlice { element_type } => (),
            ffi::Type::GpuBufferAllocator => (),
            ffi::Type::CpuBufferAllocator => (),
            ffi::Type::GpuFence => (),
            _ => panic!("Unimplemented type #{}: {:?}", type_id.0, typ),
            //_ => panic!("Unimplemented")
        }
    }

    fn get_type_binding_info(&self, type_id: ffi::TypeId) -> ffi::TypeBindingInfo {
        self.native_interface.calculate_type_binding_info(type_id)
    }

    /// Returns true if the type is a CPU reference type
    fn is_cpu_ref(&self, type_id: ffi::TypeId) -> bool {
        match &self.native_interface.types[type_id.0] {
            ffi::Type::ConstRef { .. }
            | ffi::Type::MutRef { .. }
            | ffi::Type::ConstSlice { .. }
            | ffi::Type::MutSlice { .. } => true,
            _ => false,
        }
    }

    /// Returns true if the type is a GPU buffer type
    fn is_gpu_buf(&self, type_id: ffi::TypeId) -> bool {
        match &self.native_interface.types[type_id.0] {
            ffi::Type::GpuBufferRef { .. } | ffi::Type::GpuBufferSlice { .. } => true,
            _ => false,
        }
    }

    fn is_cpu_mut_ref(&self, type_id: ffi::TypeId) -> bool {
        match &self.native_interface.types[type_id.0] {
            ffi::Type::MutRef { .. } | ffi::Type::MutSlice { .. } => true,
            _ => false,
        }
    }

    fn get_type_name(&self, type_id: ffi::TypeId) -> String {
        self.get_type_name_with_ref(type_id, None)
    }

    fn get_type_name_with_ref(&self, type_id: ffi::TypeId, lifetime: Option<&str>) -> String {
        match &self.native_interface.types[type_id.0] {
            ffi::Type::F32 => "f32".to_string(),
            ffi::Type::F64 => "f64".to_string(),
            ffi::Type::U8 => "u8".to_string(),
            ffi::Type::U16 => "u16".to_string(),
            ffi::Type::U32 => "u32".to_string(),
            ffi::Type::U64 => "u64".to_string(),
            ffi::Type::USize => "usize".to_string(),
            ffi::Type::I8 => "i8".to_string(),
            ffi::Type::I16 => "i16".to_string(),
            ffi::Type::I32 => "i32".to_string(),
            ffi::Type::I64 => "i64".to_string(),
            ffi::Type::ConstRef { element_type } => {
                if let Some(_lifetime) = lifetime {
                    format!(
                        "StackRef<{}>",
                        self.get_type_name_with_ref(*element_type, lifetime)
                    )
                } else {
                    format!("&{}", self.get_type_name_with_ref(*element_type, lifetime))
                }
            }
            ffi::Type::MutRef { element_type } => {
                if let Some(_lifetime) = lifetime {
                    format!(
                        "StackRef<{}>",
                        self.get_type_name_with_ref(*element_type, lifetime)
                    )
                } else {
                    format!(
                        "&mut {}",
                        self.get_type_name_with_ref(*element_type, lifetime)
                    )
                }
            }
            ffi::Type::ConstSlice { element_type } => {
                if let Some(_lifetime) = lifetime {
                    format!(
                        "StackRef<[{}]>",
                        self.get_type_name_with_ref(*element_type, lifetime)
                    )
                } else {
                    format!(
                        "&[{}]",
                        self.get_type_name_with_ref(*element_type, lifetime)
                    )
                }
            }
            ffi::Type::MutSlice { element_type } => {
                if let Some(_lifetime) = lifetime {
                    format!(
                        "StackRef<[{}]>",
                        self.get_type_name_with_ref(*element_type, lifetime)
                    )
                } else {
                    format!(
                        "&mut [{}]",
                        self.get_type_name_with_ref(*element_type, lifetime)
                    )
                }
            }
            ffi::Type::Array {
                element_type,
                length,
            } => format!(
                "[{}; {}]",
                self.get_type_name_with_ref(*element_type, lifetime),
                length
            ),
            ffi::Type::GpuBufferRef { element_type } => format!(
                "caiman_rt::GpuBufferRef<{}>",
                self.get_type_name_with_ref(*element_type, lifetime)
            ),
            ffi::Type::GpuBufferSlice { element_type } => format!(
                "caiman_rt::GpuBufferSlice<'callee, {}>",
                self.get_type_name_with_ref(*element_type, lifetime)
            ),
            ffi::Type::GpuBufferAllocator => format!("caiman_rt::GpuBufferAllocator<'callee>"),
            ffi::Type::GpuFence => format!("caiman_rt::GpuFence"),
            _ => format!("type_{}", type_id.0),
        }
    }

    fn get_stripped_type_name(&self, type_id: ffi::TypeId) -> String {
        match &self.native_interface.types[type_id.0] {
            ffi::Type::ConstRef { element_type } => self.get_type_name(*element_type),
            ffi::Type::MutRef { element_type } => self.get_type_name(*element_type),
            ffi::Type::ConstSlice { element_type } => self.get_type_name(*element_type),
            ffi::Type::MutSlice { element_type } => self.get_type_name(*element_type),
            ffi::Type::GpuBufferRef { element_type } => self.get_type_name(*element_type),
            ffi::Type::GpuBufferSlice { element_type } => self.get_type_name(*element_type),
            _ => self.get_type_name(type_id),
        }
    }

    fn get_stripped_var_type_name(&self, var_id: VarId) -> String {
        let type_id = self.variable_tracker.variable_types[&var_id];
        self.get_stripped_type_name(type_id)
    }

    pub fn create_ffi_type(&mut self, typ: ffi::Type) -> ffi::TypeId {
        let type_id = ffi::TypeId(self.native_interface.types.add(typ));
        self.generate_type_definition(type_id);
        type_id
    }

    pub fn lookup_closure_id(
        &mut self,
        funclet_id: ir::FuncletId,
        capture_types: &[ffi::TypeId],
        argument_types: &[ffi::TypeId],
    ) -> ClosureId {
        if let Some(closure) = self.active_closures.get(&(funclet_id, capture_types.len())) {
            for (capture_index, capture_type) in capture_types.iter().enumerate() {
                assert_eq!(closure.capture_types[capture_index], *capture_type);
            }
            closure.closure_id
        } else {
            let closure_id = ClosureId(self.closure_id_generator.generate());
            let dispatcher_id = self.lookup_dispatcher_id(argument_types);
            let old = self.active_closures.insert(
                (funclet_id, capture_types.len()),
                Closure {
                    capture_types: capture_types.to_vec().into_boxed_slice(),
                    argument_types: argument_types.to_vec().into_boxed_slice(),
                    closure_id,
                    dispatcher_id,
                },
            );
            assert!(old.is_none());
            closure_id
        }
    }

    pub fn lookup_dispatcher_id(&mut self, argument_types: &[ffi::TypeId]) -> DispatcherId {
        if let Some(dispatcher) = self.active_dispatchers.get(argument_types) {
            dispatcher.dispatcher_id
        } else {
            let dispatcher_id = DispatcherId(self.dispatcher_id_generator.generate());
            let old = self.active_dispatchers.insert(
                argument_types.to_vec().into_boxed_slice(),
                Dispatcher { dispatcher_id },
            );
            assert!(old.is_none());
            dispatcher_id
        }
    }

    fn build_const_int(&mut self, value: String, typ: &str, type_id: ffi::TypeId) -> VarId {
        let variable_id = self.variable_tracker.create_local_alloc(Some(type_id));
        write!(
            self.code_writer,
            "instance.locals.calloc::<{typ}>({}, {value});\n",
            variable_id.0
        );
        variable_id
    }

    pub fn build_constant_int<T: ToString>(&mut self, value: T, type_id: ffi::TypeId) -> VarId {
        self.build_const_int(
            value.to_string(),
            &self.get_stripped_type_name(type_id),
            type_id,
        )
    }

    pub fn build_select_hack(
        &mut self,
        condition_var_id: VarId,
        true_case_var_id: VarId,
        false_case_var_id: VarId,
    ) -> VarId {
        //let true_type_id = self.variable_tracker.variable_types[&true_case_var_id];
        //let false_type_id = self.variable_tracker.variable_types[&false_case_var_id];
        //assert_eq!(true_type_id, false_type_id);
        //let type_id = true_type_id;
        let variable_kind = self.variable_tracker.get_kind(true_case_var_id);
        assert_eq!(
            variable_kind,
            self.variable_tracker.get_kind(false_case_var_id)
        );
        let variable_id = self.variable_tracker.create(variable_kind, None);
        // Too lazy to implement booleans for now
        write!(
            self.code_writer,
            "let {} = if {} != 0 {{ {} }} else {{ {} }};\n",
            self.get_var_name(variable_id),
            self.get_var_name(condition_var_id),
            self.get_var_name(true_case_var_id),
            self.get_var_name(false_case_var_id)
        );
        variable_id
    }

    pub fn begin_if_else(
        &mut self,
        condition_var_id: VarId,
        output_type_ids: &[ffi::TypeId],
    ) -> Box<[VarId]> {
        // Temporary fix
        self.reset_pipeline();

        let mut var_ids = Vec::<VarId>::new();
        let mut var_names = Vec::<String>::new();
        let mut var_types = Vec::<String>::new();
        for (i, type_id) in output_type_ids.iter().enumerate() {
            let var_id = self.variable_tracker.create_local_alloc(Some(*type_id));
            var_types.push(self.get_stripped_type_name(*type_id));
            var_ids.push(var_id);
        }

        write!(
            self.code_writer,
            "if {} !=0 {{ ",
            self.get_var_name(condition_var_id)
        );

        var_ids.into_boxed_slice()
    }

    pub fn end_if_begin_else(&mut self, output_var_ids: &[VarId]) {
        // Temporary fix
        self.reset_pipeline();

        write!(self.code_writer, " }} else {{ ");
    }

    pub fn end_else(&mut self, output_var_ids: &[VarId]) {
        write!(self.code_writer, " }}\n");

        // Temporary fix
        self.reset_pipeline();
    }

    pub fn build_external_cpu_function_call(
        &mut self,
        external_function_id: ir::ExternalFunctionId,
        argument_vars: &[VarId],
    ) -> Box<[VarId]> {
        let external_cpu_function = &self.native_interface.external_functions
            [external_function_id.0]
            .get_cpu_pure_operation()
            .unwrap();
        let call_result_var = self.variable_tracker.generate();
        let mut argument_string = String::new();
        for (index, argument) in argument_vars.iter().enumerate() {
            let src = if self.variable_tracker.is_temp(*argument) {
                format!("{}", self.access_val_str(*argument),)
            } else {
                format!("{}", self.get_var_name(*argument))
            };
            argument_string += &src;
            if index + 1 < argument_vars.len() {
                argument_string += ", ";
            }
        }
        self.code_writer.write(format!(
            "let {} = instance.cpu_functions.{}(instance.state, {});\n",
            self.get_var_name(call_result_var),
            external_cpu_function.name,
            argument_string
        ));
        let mut output_variables = Vec::<VarId>::new();
        for (i, output_type) in external_cpu_function.output_types.iter().enumerate() {
            let var = self.variable_tracker.create_local_alloc(Some(*output_type));
            output_variables.push(var);
            self.code_writer.write(format!(
                "*instance.locals.calloc::<{}>({}, {}.{});\n",
                self.get_stripped_type_name(*output_type),
                var.0,
                self.get_var_name(call_result_var),
                i
            ));
        }
        output_variables.into_boxed_slice()
    }

    pub fn build_alloc_temp_gpu(
        &mut self,
        type_id: ffi::TypeId,
        buffer_flags: ir::BufferFlags,
    ) -> VarId {
        let variable_id = self.variable_tracker.create_local_gpu(Some(type_id));
        let type_binding_info = self.get_type_binding_info(type_id);
        let type_name = self.get_stripped_type_name(type_id);
        write!(
            self.code_writer,
            "instance.glocals.alloc_gpu::<{type_name}>(instance.state, {}, wgpu::BufferUsages::empty()",
            variable_id.0
        );
        if buffer_flags.map_read {
            write!(self.code_writer, " | wgpu::BufferUsages::MAP_READ");
        }
        if buffer_flags.map_write {
            write!(self.code_writer, " | wgpu::BufferUsages::MAP_WRITE");
        }
        if buffer_flags.copy_src {
            write!(self.code_writer, " | wgpu::BufferUsages::COPY_SRC");
        }
        if buffer_flags.copy_dst {
            write!(self.code_writer, " | wgpu::BufferUsages::COPY_DST");
        }
        if buffer_flags.storage {
            write!(self.code_writer, " | wgpu::BufferUsages::STORAGE");
        }
        if buffer_flags.uniform {
            write!(self.code_writer, " | wgpu::BufferUsages::UNIFORM");
        }
        write!(self.code_writer, ");\n");
        //self.code_writer.write(format!("let mut {} = instance.state.get_device_mut().create_buffer(& wgpu::BufferDescriptor {{ label : None, size : {}, usage : wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::MAP_WRITE, mapped_at_creation : false}});\n", self.get_var_name(variable_id), type_binding_info.size));
        variable_id
    }

    pub fn build_buffer_ref(&mut self, buffer_var_id: VarId, type_id: ffi::TypeId) -> VarId {
        buffer_var_id
    }

    pub fn build_buffer_suballocate_ref(
        &mut self,
        buffer_allocator_var_id: VarId,
        type_id: ffi::TypeId,
    ) -> VarId {
        todo!();
        let variable_id = self.variable_tracker.create_local_data(None);
        let type_binding_info = self.get_type_binding_info(type_id);
        let type_name = self.get_type_name(type_id);
        write!(
            self.code_writer,
            "let {} = {}.suballocate_ref::<'callee, {}>().unwrap();\n",
            self.get_var_name(variable_id),
            self.get_var_name(buffer_allocator_var_id),
            type_name
        );
        variable_id
    }

    pub fn build_buffer_suballocate_slice(
        &mut self,
        buffer_allocator_var_id: VarId,
        type_id: ffi::TypeId,
        count_var_id: VarId,
    ) -> VarId {
        todo!();
        let variable_id = self.variable_tracker.create_local_data(None);
        let type_binding_info = self.get_type_binding_info(type_id);
        let type_name = self.get_type_name(type_id);
        write!(
            self.code_writer,
            "let {} = {}.suballocate_slice::<'callee, {}>({}).unwrap();\n",
            self.get_var_name(variable_id),
            self.get_var_name(buffer_allocator_var_id),
            type_name,
            self.get_var_name(count_var_id)
        );
        variable_id
    }

    pub fn build_test_suballocate_many(
        &mut self,
        buffer_allocator_var_id: VarId,
        type_id_and_count_var_id_pairs: &[(ffi::TypeId, Option<VarId>)],
    ) -> VarId {
        todo!();
        let mut layouts_string = String::from("");
        let mut element_counts_string = String::from("");

        for (type_id, count_var_id_opt) in type_id_and_count_var_id_pairs.iter() {
            let type_binding_info = self.get_type_binding_info(*type_id);
            write!(
                layouts_string,
                "caiman_rt::TypeLayout{{byte_size : {}, alignment : {}}}, ",
                type_binding_info.size, type_binding_info.alignment
            );
            if let Some(count_var_id) = count_var_id_opt {
                write!(
                    element_counts_string,
                    "Some({}), ",
                    self.get_var_name(*count_var_id)
                );
            } else {
                write!(element_counts_string, "None, ");
            }
        }

        let success_var_id = self.variable_tracker.generate();
        write!(
            self.code_writer,
            "let {} = {}.test_suballocate_many(&[{}], &[{}]);\n",
            self.get_var_name(success_var_id),
            self.get_var_name(buffer_allocator_var_id),
            layouts_string,
            element_counts_string
        );

        success_var_id
    }

    pub fn build_push_serialized_join(
        &mut self,
        funclet_id: ir::FuncletId,
        capture_var_ids: &[VarId],
        capture_types: &[ffi::TypeId],
        argument_types: &[ffi::TypeId],
        output_types: &[ffi::TypeId],
    ) {
        let _closure_id = self.lookup_closure_id(funclet_id, capture_types, argument_types);
        let _argument_dispatcher_id = self.lookup_dispatcher_id(argument_types);
        println!(
            "Pushed serialzed join for funclet {}: {:?}",
            funclet_id,
            self.active_closures.get(&(funclet_id, capture_types.len()))
        );

        let tuple_definition_string = self.get_tuple_definition_string(capture_types);
        write!(
            self.code_writer,
            "{{ let join_data : Funclet{}Capturing{}CapturedTuple<'callee> = (",
            funclet_id,
            capture_types.len()
        );
        for (var_id, var_type) in capture_var_ids.iter().zip(capture_types.iter()) {
            if self.is_cpu_ref(*var_type) {
                write!(self.code_writer, "{}, ", self.make_stack_ref(*var_id));
            } else if self.is_gpu_buf(*var_type) {
                write!(self.code_writer, "{}, ", self.access_gpu_str(*var_id));
            } else {
                write!(self.code_writer, "{}, ", self.access_val_str(*var_id));
            }
        }
        write!(self.code_writer, "); let closure_header = ClosureHeader::Funclet{}Capturing{}; unsafe {{ join_stack.push_unsafe_unaligned(join_data).expect(\"Ran out of memory while serializing join\"); join_stack.push_unsafe_unaligned(closure_header).expect(\"Ran out of memory while serializing join\"); }}", funclet_id, capture_types.len());

        write!(self.code_writer, "}}");
    }

    pub fn encode_clone_local_data_from_buffer(
        &mut self,
        source_var: VarId,
        type_id: ffi::TypeId,
    ) -> VarId {
        //let type_id = self.variable_tracker.get_type_id(source_var);

        let range_var_id = self.variable_tracker.generate();
        let output_temp_var_id = self.variable_tracker.generate();
        let slice_var_id = self.variable_tracker.generate();
        let future_var_id = self.variable_tracker.generate();
        let type_binding_info = self.get_type_binding_info(type_id);
        let type_name = self.get_type_name(type_id);

        let output_var_id = self.variable_tracker.create_local_data(Some(type_id));

        self.code_writer
            .write(format!("let {} = {{\n", self.get_var_name(output_var_id)));
        self.code_writer.write(format!(
            "let {} = {}.slice();\n",
            self.get_var_name(slice_var_id),
            self.build_get_gpu_ref(source_var, None)
        ));
        self.code_writer.write(format!(
            "let ({0}_send, {0}_recv) = futures::channel::oneshot::channel::<()>();\n",
            self.get_var_name(future_var_id)
        ));
        self.code_writer.write(format!("{1}.map_async(wgpu::MapMode::Read, |res| {{res.unwrap(); {0}_send.send(()).unwrap(); }});\n", self.get_var_name(future_var_id), self.get_var_name(slice_var_id)));
        self.code_writer.write(format!(
            "instance.state.get_device_mut().poll(wgpu::Maintain::Wait);\n"
        ));
        self.code_writer.write(format!(
            "futures::executor::block_on({}_recv);\n",
            self.get_var_name(future_var_id)
        ));
        self.code_writer.write(format!(
            "let {} = {}.get_mapped_range();\n",
            self.get_var_name(range_var_id),
            self.get_var_name(slice_var_id)
        ));
        self.code_writer.write(format!(
            "*unsafe {{ std::mem::transmute::<* const u8, & {}>({}.as_ptr()) }}\n",
            type_name,
            self.get_var_name(range_var_id)
        ));
        self.code_writer.write(String::from("};\n"));
        return output_var_id;
    }

    /// Returns a string representing variable `var` as a slice of little-endian bytes.
    fn local_as_le_bytes(&self, var: VarId, var_type_id: ffi::TypeId) -> String {
        use ffi::Type::*;
        //let var_type_id = self.variable_tracker.get_type_id(var);
        let var_type = self.native_interface.types.get(var_type_id.0).unwrap();

        // TODO: This should really be expanded to encompass all types, but I'm
        // doing the bare minimum to get this working
        match var_type {
            F32 | F64 | U8 | U16 | U32 | U64 | USize | I8 | I16 | I32 | I64 => {
                return format!("&{}.to_le_bytes()", self.access_val_str(var),)
            }
            Array {
                element_type,
                length,
            } => return format!("bytemuck::cast_slice(&{})", self.access_val_str(var)),
            _ => panic!("type {:?} not yet supported", var_type),
        }
    }

    fn local_ref_content_as_le_bytes(&self, var: VarId, var_type_id: ffi::TypeId) -> String {
        use ffi::Type::*;
        let var_name = self.get_var_name(var);
        //let var_type_id = self.variable_tracker.get_type_id(var);
        let var_type = self.native_interface.types.get(var_type_id.0).unwrap();
        let data_type_id = match var_type {
            ConstRef { element_type } => *element_type,
            MutRef { element_type } => *element_type,
            _ => panic!("type {:?} not yet supported", var_type),
        };
        let data_type = self.native_interface.types.get(data_type_id.0).unwrap();

        // TODO: This should really be expanded to encompass all types, but I'm
        // doing the bare minimum to get this working
        match data_type {
            F32 | F64 | U8 | U16 | U32 | U64 | USize | I8 | I16 | I32 | I64 => {
                return format!("&{}.to_le_bytes()", var_name)
            }
            Array {
                element_type,
                length,
            } => return format!("bytemuck::cast_slice(&{})", var_name),
            _ => panic!("type {:?} not yet supported", var_type),
        }
    }

    /// Builds a string which represents the Rust code for getting a reference to
    /// the GPU buffer for the given variable.
    /// If the variable is a temporary, this will be a call to `instance.glocals.get_gpu_ref`.
    /// If the variable is not a temporary, this will be the variable name itself.
    /// # Arguments
    /// * `var` - The variable to get the GPU buffer reference for
    /// * `type_id` - The type of the variable, if known. If `None`, the type will be looked up
    ///  from the variable tracker.
    fn build_get_gpu_ref(&self, var: VarId, type_id: Option<ffi::TypeId>) -> String {
        if self.variable_tracker.is_temp(var) {
            let type_name = type_id
                .map(|x| self.get_type_name(x))
                .unwrap_or_else(|| self.get_stripped_var_type_name(var));
            format!("instance.glocals.get_gpu_ref::<{type_name}>({})", var.0)
        } else {
            self.get_var_name(var)
        }
    }

    pub fn encode_copy_buffer_from_local_data(
        &mut self,
        destination_var: VarId,
        source_var: VarId,
        type_id: ffi::TypeId,
    ) {
        let buffer_view_var_name = self.get_var_name(destination_var);
        write!(self.code_writer, "{{ \n");
        let source_bytes = self.local_as_le_bytes(source_var, type_id);
        write!(self.code_writer, "let _t = {};\n", source_bytes);
        self.code_writer.write(format!(
            "let {} = {};\n",
            buffer_view_var_name,
            self.build_get_gpu_ref(destination_var, Some(type_id))
        ));
        self.code_writer.write(format!(
            "instance.state.get_queue_mut().write_buffer(unsafe {{ &*{}.buffer }}, {}.base_address, _t);\n",
            buffer_view_var_name, buffer_view_var_name
        ));
        write!(self.code_writer, "}}\n");
    }

    pub fn encode_copy_buffer_from_buffer(
        &mut self,
        destination_var: VarId,
        source_var: VarId,
        type_id: ffi::TypeId,
    ) {
        //let type_id = self.variable_tracker.get_type_id(source_var);
        //assert_eq!(type_id, self.variable_tracker.get_type_id(destination_var));
        let type_binding_info = self.get_type_binding_info(type_id);
        let type_name = self.get_type_name(type_id);
        self.begin_command_encoding();
        write!(
            self.code_writer,
            "command_encoder.copy_buffer_to_buffer(& {}, 0, \
                & {}, 0, {});\n",
            self.build_get_gpu_ref(destination_var, Some(type_id)),
            self.build_get_gpu_ref(source_var, Some(type_id)),
            type_binding_info.size
        );
        let command_buffer_id = self.end_command_encoding();
        self.enqueue_command_buffer(command_buffer_id);
    }

    /*fn build_create_buffer_with_data(&mut self, data_var: VarId, type_id: ffi::TypeId, buffer_flags : ir::BufferFlags) -> VarId {
        let variable_id = self.variable_tracker.generate();
        let type_binding_info = self.get_type_binding_info(type_id);
        let buffer_view_var_name = self.get_var_name(variable_id);
        let data_bytes = self.local_as_le_bytes(data_var, type_id);
        self.code_writer.write(format!("let mut {} = instance.state.get_device_mut().create_buffer(& wgpu::BufferDescriptor {{ label : None, size : {}, usage : wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::MAP_WRITE, mapped_at_creation : false}});\n", self.get_var_name(variable_id), type_binding_info.size));
        self.code_writer.write(format!(
            "instance.state.get_queue_mut().write_buffer(& {}.buffer, {}.base_address, {} );\n",
            buffer_view_var_name, buffer_view_var_name, data_bytes
        ));
        variable_id
    }*/

    /*fn build_create_buffer_with_buffer_data(
        &mut self,
        data_var: VarId,
        type_id: ffi::TypeId,
        buffer_flags : ir::BufferFlags
    ) -> VarId {
        let variable_id = self.variable_tracker.generate();
        let type_binding_info = self.get_type_binding_info(type_id);
        let type_name = self.get_type_name(type_id);
        self.code_writer.write(format!("let mut {} = instance.state.get_device_mut().create_buffer(& wgpu::BufferDescriptor {{ label : None, size : {}, usage : wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::MAP_WRITE, mapped_at_creation : false}});\n", self.get_var_name(variable_id), type_binding_info.size));
        write!(self.code_writer, "{{\n");
        self.code_writer.write("let mut command_encoder = instance.state.get_device_mut().create_command_encoder(& wgpu::CommandEncoderDescriptor {label : None});\n".to_string());
        write!(
            self.code_writer,
            "command_encoder.copy_buffer_to_buffer(& {}, 0, & {}, 0, {});\n",
            self.get_var_name(data_var),
            self.get_var_name(variable_id),
            type_binding_info.size
        );
        self.code_writer
            .write("let command_buffer = command_encoder.finish();\n".to_string());
        self.code_writer
            .write("instance.state.get_queue_mut().submit([command_buffer]);\n".to_string());
        self.code_writer.write(
            "let (submit_send, submit_recv) = futures::channel::oneshot::channel::<()>();\n"
                .to_string(),
        );
        self.code_writer.write("instance.state.get_queue_mut().on_submitted_work_done(|| submit_send.send(()).unwrap());\n".to_string());
        self.code_writer
            .write("instance.state.get_device_mut().poll(wgpu::Maintain::Wait);\n".to_string());
        self.code_writer
            .write("futures::executor::block_on(submit_recv);\n".to_string());
        write!(self.code_writer, "}}\n");

        variable_id
    }*/

    pub fn build_compute_dispatch_with_outputs(
        &mut self,
        kernel: &ffi::GpuKernel,
        dimension_vars: &[VarId; 3],
        argument_vars: &[VarId],
        output_vars: &[VarId],
    ) {
        self.generate_compute_dispatch(kernel, dimension_vars, argument_vars, output_vars);
    }

    pub fn build_alloc_temp_local_ref(&mut self, type_id: ffi::TypeId) -> VarId {
        let variable_id = self.variable_tracker.create_local_alloc(Some(type_id));
        let type_name = self.get_stripped_type_name(type_id);
        write!(
            self.code_writer,
            "instance.locals.malloc::<{type_name}>({});\n",
            variable_id.0,
        );
        variable_id
    }

    pub fn build_write_local_ref(&mut self, dst_ref_var_id: VarId, src_var_id: VarId) {
        let src = self.access_val_str(src_var_id);
        write!(
            self.code_writer,
            "*{} = {};\n",
            self.access_ref_str(dst_ref_var_id, true),
            src,
        );
    }

    fn make_stack_ref(&self, var_id: VarId) -> String {
        if self.variable_tracker.is_arg(var_id) {
            self.get_var_name(var_id)
        } else {
            format!("StackRef::local({})", var_id.0)
        }
    }

    /// Returns a string that is the code that can access the reference to a variable
    fn access_ref_str(&self, var_id: VarId, is_mut: bool) -> String {
        if self.variable_tracker.is_temp(var_id) {
            // safe bc of the caiman type system
            format!(
                "instance.locals.get{}::<{}>({})",
                if is_mut { "_mut" } else { "" },
                self.get_stripped_var_type_name(var_id),
                var_id.0,
            )
        } else if self.variable_tracker.is_arg(var_id)
            && self.is_cpu_ref(self.variable_tracker.variable_types[&var_id])
        {
            format!(
                "{}.get{}(&{} instance)",
                self.get_var_name(var_id),
                if is_mut { "_mut" } else { "" },
                if is_mut { "mut" } else { "" }
            )
        } else if self.variable_tracker.variable_types.contains_key(&var_id)
            && self.is_cpu_ref(self.variable_tracker.variable_types[&var_id])
        {
            self.get_var_name(var_id)
        } else {
            format!(
                "(&{}{})",
                if is_mut { "mut " } else { "" },
                self.get_var_name(var_id)
            )
        }
    }

    /// Returns a string that is the code which can accesss the value of a variable
    fn access_val_str(&self, var_id: VarId) -> String {
        if self.variable_tracker.is_temp(var_id) {
            // safe bc of the caiman type system
            format!(
                "(*instance.locals.get::<{}>({}))",
                self.get_stripped_var_type_name(var_id),
                var_id.0,
            )
        } else if self.variable_tracker.is_arg(var_id)
            && self.is_cpu_ref(self.variable_tracker.variable_types[&var_id])
        {
            format!("(*{}.get(&instance))", self.get_var_name(var_id))
        } else if self.variable_tracker.variable_types.contains_key(&var_id)
            && self.is_cpu_ref(self.variable_tracker.variable_types[&var_id])
        {
            format!("(*{})", self.get_var_name(var_id))
        } else {
            self.get_var_name(var_id)
        }
    }

    /// Returns a string that is the code which can accesss the value of a variable
    /// which is a GPU buffer
    fn access_gpu_str(&self, var_id: VarId) -> String {
        if self.variable_tracker.is_temp(var_id) {
            format!(
                "instance.glocals.get_gpu_ref::<{}>({})",
                self.get_stripped_var_type_name(var_id),
                var_id.0,
            )
        } else {
            self.get_var_name(var_id)
        }
    }

    pub fn build_read_local_ref(&mut self, src_ref_var_id: VarId, type_id: ffi::TypeId) -> VarId {
        let variable_id = self.variable_tracker.create_local_data(Some(type_id));
        write!(
            self.code_writer,
            "let {} = {};\n",
            self.get_var_name(variable_id),
            self.access_val_str(src_ref_var_id)
        );
        write!(
            self.code_writer,
            "instance.locals.calloc::<{}>({}, {});\n",
            self.get_stripped_type_name(type_id),
            variable_id.0,
            self.get_var_name(variable_id)
        );
        variable_id
    }

    pub fn build_borrow_local_ref(&mut self, src_var_id: VarId, type_id: ffi::TypeId) -> VarId {
        let variable_id = self.variable_tracker.create_local_data(Some(type_id));
        let type_name = self.get_type_name(type_id);
        write!(
            self.code_writer,
            "let {} = & {};\n",
            self.get_var_name(variable_id),
            self.get_var_name(src_var_id)
        );
        variable_id
    }
}
