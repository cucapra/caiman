use crate::ir;
use crate::shadergen;
use crate::arena::Arena;
use std::default::Default;
use std::collections::HashMap;
use std::collections::BTreeMap;
use std::collections::HashSet;
use crate::rust_wgpu_backend::code_writer::CodeWriter;
use std::fmt::Write;
use crate::id_generator::IdGenerator;
use super::ffi;

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
pub struct YieldPoint
{
	pub name : String,
	pub yielded_types : Box<[ffi::TypeId]>,
	pub resuming_types : Box<[ffi::TypeId]>,
}

#[derive(Debug, Default)]
struct SubmissionQueue
{
	//most_recently_synchronized_submission_id : Option<SubmissionId>,
	next_submission_id : SubmissionId,
	next_fence_id : FenceId
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum VariableKind
{
	Dead,
	Buffer,
	LocalData,
}

#[derive(Default)]
struct VariableTracker
{
	id_generator : IdGenerator,
	variable_kinds : HashMap<VarId, VariableKind>,
	variable_types : HashMap<VarId, ffi::TypeId>
}

impl VariableTracker
{
	fn new() -> Self
	{
		Self { id_generator : IdGenerator::new(), variable_kinds : HashMap::<VarId, VariableKind>::new(), variable_types : HashMap::<VarId, ffi::TypeId>::new() }
	}

	fn generate(&mut self) -> VarId
	{
		VarId(self.id_generator.generate())
	}

	fn create(&mut self, kind : VariableKind, type_id : ffi::TypeId) -> VarId
	{
		let id = self.generate();
		self.variable_kinds.insert(id, kind);
		self.variable_types.insert(id, type_id);
		id
	}

	fn create_local_data(&mut self, type_id : ffi::TypeId) -> VarId
	{
		let id = self.create(VariableKind::LocalData, type_id);
		id
	}

	fn create_buffer(&mut self, type_id : ffi::TypeId) -> VarId
	{
		let id = self.create(VariableKind::Buffer, type_id);
		id
	}

	fn get_type_id(& self, variable_id : VarId) -> ffi::TypeId
	{
		self.variable_types[& variable_id]
	}

	fn get_kind(& self, variable_id : VarId) -> VariableKind
	{
		self.variable_kinds[& variable_id]
	}
	
	fn get_var_name(& self, variable_id : VarId) -> String
	{
		format!("var_{}", variable_id.0)
	}
}

enum Binding
{
	Buffer(usize)
}

#[derive(Default)]
struct SubmissionEncodingState
{
	command_buffer_ids : Vec<CommandBufferId>
}

struct ActiveFuncletState
{
	funclet_id : ir::FuncletId,
	result_type_ids : Box<[ffi::TypeId]>,
	next_funclet_ids : Option<Box<[ir::FuncletId]>>,
	capture_count : usize,
	output_count : usize,
	output_type_ids : Box<[ffi::TypeId]>,
	next_funclet_input_types : Option<Box<[Box<[ffi::TypeId]>]>>
}

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Default)]
struct ShaderModuleKey
{
	external_gpu_function_id : ir::ExternalGpuFunctionId
}

impl ShaderModuleKey
{
	fn instance_field_name(&self) -> String
	{
		format!("external_gpu_function_{}_module", self.external_gpu_function_id)
	}
}

/*#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Default)]
struct BindGroupLayoutKey
{
	external_gpu_function_id : ir::ExternalGpuFunctionId
}

impl BindGroupLayoutKey
{
	fn instance_field_name(&self) -> String
	{
		format!("external_gpu_function_{}_bind_group_layout", self.external_gpu_function_id)
	}
}

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Default)]
struct PipelineLayoutKey
{
	external_gpu_function_id : ir::ExternalGpuFunctionId
}

impl PipelineLayoutKey
{
	fn instance_field_name(&self) -> String
	{
		format!("external_gpu_function_{}_pipeline_layout", self.external_gpu_function_id)
	}
}*/

struct GpuFunctionInvocation
{
	external_gpu_function_id : ir::ExternalGpuFunctionId,
	bindings : BTreeMap<usize, (Option<usize>, Option<usize>)>,
	shader_module_key : ShaderModuleKey,
}

#[derive(Debug)]
struct Closure
{
	capture_types : Box<[ir::ffi::TypeId]>,
	argument_types : Box<[ir::ffi::TypeId]>,
	closure_id : ClosureId,
	dispatcher_id : DispatcherId
}

struct Dispatcher
{
	dispatcher_id : DispatcherId
}

pub struct CodeGenerator<'program>
{
	type_code_writer : CodeWriter,
	state_code_writer : CodeWriter,
	code_writer : CodeWriter, // the "everything else" for now
	//types : Arena<ffi::Type>,
	//external_cpu_functions : & 'program [ir::ExternalCpuFunction],
	//external_gpu_functions : & 'program [ir::ExternalGpuFunction],
	has_been_generated : HashSet<ffi::TypeId>,
	variable_tracker : VariableTracker,
	active_pipeline_name : Option<String>,
	active_funclet_result_type_ids : Option<Box<[ffi::TypeId]>>,
	active_funclet_state : Option<ActiveFuncletState>,
	use_recording : bool,
	active_submission_encoding_state : Option<SubmissionEncodingState>,
	active_external_gpu_function_id : Option<ir::ExternalGpuFunctionId>,
	active_shader_module_key : Option<ShaderModuleKey>,
	shader_modules : BTreeMap<ShaderModuleKey, shadergen::ShaderModule>,
	submission_queue : SubmissionQueue,
	next_command_buffer_id : CommandBufferId,
	gpu_function_invocations : Vec<GpuFunctionInvocation>,
	original_native_interface : & 'program ffi::NativeInterface,
	native_interface : ffi::NativeInterface,
	active_closures : HashMap<(ir::FuncletId, usize), Closure>,
	closure_id_generator : IdGenerator,
	active_yield_point_ids : HashSet<ir::PipelineYieldPointId>,
	dispatcher_id_generator : IdGenerator,
	active_dispatchers : HashMap<Box<[ffi::TypeId]>, Dispatcher>,
}

impl<'program> CodeGenerator<'program>
{
	pub fn new(native_interface : & 'program ffi::NativeInterface/*, types : Arena<ffi::Type>, external_cpu_functions : & 'program [ir::ExternalCpuFunction], external_gpu_functions : & 'program [ir::ExternalGpuFunction]*/) -> Self
	{
		let variable_tracker = VariableTracker::new();
		let type_code_writer = CodeWriter::new();
		let state_code_writer = CodeWriter::new();
		let code_writer = CodeWriter::new();
		let has_been_generated = HashSet::new();
		let mut code_generator = Self {original_native_interface : native_interface, native_interface : native_interface.clone(), type_code_writer, state_code_writer, code_writer, /*types,*/ has_been_generated, variable_tracker, /*external_cpu_functions, external_gpu_functions,*/ active_pipeline_name : None, active_funclet_result_type_ids : None, active_funclet_state : None, use_recording : true, active_submission_encoding_state : None, active_external_gpu_function_id : None, active_shader_module_key : None, shader_modules : BTreeMap::new(), submission_queue : Default::default(), next_command_buffer_id : CommandBufferId(0), gpu_function_invocations : Vec::new(), active_closures : HashMap::new(), closure_id_generator : IdGenerator::new(), active_yield_point_ids : HashSet::new(), dispatcher_id_generator : IdGenerator::new(), active_dispatchers : HashMap::new()};

		let type_ids = code_generator.native_interface.types.iter().map(|(type_id, _)| ffi::TypeId(* type_id)).collect::<Box<[ffi::TypeId]>>();
		for & type_id in type_ids.iter()
		{
			code_generator.generate_type_definition(type_id);
		}

		code_generator
	}

	pub fn finish(&mut self) -> String
	{
		self.write_states();
		self.type_code_writer.finish() + self.state_code_writer.finish().as_str() + self.code_writer.finish().as_str()
	}

	fn get_tuple_definition_string(& self, type_ids : &[ffi::TypeId]) -> String
	{
		let mut output_string = String::new();
		write!(output_string, "(");
		for (index, type_id) in type_ids.iter().enumerate()
		{
			let type_name = self.get_type_name(* type_id);
			write!(output_string, "{}, ", type_name);
		}
		write!(output_string, ")");
		output_string
	}

	fn generate_compute_dispatch_outputs(&mut self, external_function_id : ir::ExternalCpuFunctionId) -> Box<[VarId]>
	{
		let mut output_vars = Vec::<VarId>::new();

		let external_gpu_function = & self.native_interface.external_gpu_functions[& external_function_id];
		for (output_index, output_type_id) in external_gpu_function.output_types.iter().enumerate()
		{
			let variable_id = self.variable_tracker.create_buffer(* output_type_id);
			output_vars.push(variable_id);
		}

		return output_vars.into_boxed_slice();
	}

	// This will need to be reassessed if modifying bindings from the coordinator becomes possible
	/*fn compile_external_gpu_function(&mut self, external_function_id : ir::ExternalGpuFunctionId)
	{
		let external_gpu_function = & self.external_gpu_functions[external_function_id];

		let mut shader_module = match & external_gpu_function.shader_module_content
		{
			ir::ShaderModuleContent::Wgsl(text) => shadergen::ShaderModule::new_with_wgsl(text.as_str())
		};

		self.code_writer.write_str("let module = state.get_device_mut().create_shader_module(& wgpu::ShaderModuleDescriptor { label : None, source : wgpu::ShaderSource::Wgsl(std::borrow::Cow::from(\"");
		/*match & external_gpu_function.shader_module_content
		{
			ir::ShaderModuleContent::Wgsl(text) => self.code_writer.write_str(text.as_str())
		}*/
		self.code_writer.write_str(shader_module.compile_wgsl_text().as_str());
		self.code_writer.write_str("\"))});\n");

		self.active_external_gpu_function_id = Some(external_function_id);
		self.active_shader_module = Some(shader_module);
	}*/

	fn set_active_external_gpu_function(&mut self, external_function_id : ir::ExternalGpuFunctionId)
	{
		// Will need to be more careful with this check once modules no longer correspond to external gpu functions one to one
		if let Some(previous_id) = self.active_external_gpu_function_id.as_ref()
		{
			if * previous_id == external_function_id
			{
				return;
			}
		}

		self.active_external_gpu_function_id = None;

		let shader_module_key = ShaderModuleKey{external_gpu_function_id : external_function_id};

		write!(self.code_writer, "let module = & instance.{};\n", shader_module_key.instance_field_name());

		if ! self.shader_modules.contains_key(& shader_module_key)
		{
			let external_gpu_function = & self.native_interface.external_gpu_functions[& external_function_id];
	
			let mut shader_module = match & external_gpu_function.shader_module_content
			{
				ffi::ShaderModuleContent::Wgsl(text) => shadergen::ShaderModule::new_with_wgsl(text.as_str())
			};
	
			//self.code_writer.write_str("let module = instance.state.get_device_mut().create_shader_module(& wgpu::ShaderModuleDescriptor { label : None, source : wgpu::ShaderSource::Wgsl(std::borrow::Cow::from(\"");
			/*match & external_gpu_function.shader_module_content
			{
				ir::ShaderModuleContent::Wgsl(text) => self.code_writer.write_str(text.as_str())
			}*/
			//self.code_writer.write_str(shader_module.compile_wgsl_text().as_str());
			//self.code_writer.write_str("\"))});\n");

			self.shader_modules.insert(shader_module_key, shader_module);
		}

		self.active_external_gpu_function_id = Some(external_function_id);
		self.active_shader_module_key = Some(shader_module_key);
	}

	fn set_active_bindings(&mut self, argument_vars : &[VarId], output_vars : &[VarId])// -> Box<[usize]>
	{
		let external_function_id = self.active_external_gpu_function_id.unwrap();
		let external_gpu_function = & self.native_interface.external_gpu_functions[& external_function_id];

		let mut bindings = std::collections::BTreeMap::<usize, (Option<usize>, Option<usize>)>::new();
		let mut output_binding_map = std::collections::BTreeMap::<usize, usize>::new();
		let mut input_binding_map = std::collections::BTreeMap::<usize, usize>::new();

		for resource_binding in external_gpu_function.resource_bindings.iter()
		{
			assert_eq!(resource_binding.group, 0);
			bindings.insert(resource_binding.binding, (resource_binding.input, resource_binding.output));

			if let Some(input) = resource_binding.input
			{
				input_binding_map.insert(input, resource_binding.binding);
			}

			if let Some(output) = resource_binding.output
			{
				output_binding_map.insert(output, resource_binding.binding);
			}
		}

		let mut input_staging_variables = Vec::<VarId>::new();
		assert_eq!(argument_vars.len(), external_gpu_function.input_types.len());
		for input_index in 0 .. external_gpu_function.input_types.len()
		{
			let type_id = external_gpu_function.input_types[input_index];
			//let variable_id = self.build_create_buffer_with_data(argument_vars[input_index], type_id);
			let input_variable_id = argument_vars[input_index];

			let binding = input_binding_map[& input_index];
			if let (_, Some(_output)) = bindings[& binding]
			{
				//panic!("Incorrectly handled");
				//let variable_id = self.build_create_buffer_with_buffer_data(input_variable_id, type_id);
				input_staging_variables.push(input_variable_id);
			}
			else
			{
				input_staging_variables.push(input_variable_id);
			}
		}

		let mut output_staging_variables = Vec::<VarId>::new();
		for output_index in 0 .. external_gpu_function.output_types.len()
		{
			let binding = output_binding_map[& output_index];
			if let (Some(input), _) = bindings[& binding]
			{
				//panic!("Incorrectly handled");
				let variable_id = input_staging_variables[input];
				assert_eq!(variable_id, output_vars[output_index]);
				output_staging_variables.push(variable_id);
			}
			else
			{
				let type_id = external_gpu_function.output_types[output_index];
				//let variable_id = self.build_create_buffer(type_id);
				let variable_id = output_vars[output_index];
				output_staging_variables.push(variable_id);
			}
		};

		let invocation_id = self.gpu_function_invocations.len();
		self.gpu_function_invocations.push(GpuFunctionInvocation{external_gpu_function_id : external_function_id, bindings, shader_module_key : self.active_shader_module_key.unwrap()});
		let gpu_function_invocation = & self.gpu_function_invocations[invocation_id];
		
		self.code_writer.write("let entries = [".to_string());
		for (binding, (input_opt, output_opt)) in gpu_function_invocation.bindings.iter()
		{
			let mut variable_id : Option<VarId> = None;
			
			if let Some(input) = input_opt
			{
				variable_id = Some(input_staging_variables[*input]);
			}

			if let Some(output) = output_opt
			{
				variable_id = Some(output_staging_variables[*output]);
			}

			assert_eq!(variable_id.is_some(), true, "Binding must be input or output");
			self.code_writer.write(format!("wgpu::BindGroupEntry {{binding : {}, resource : {}.as_binding_resource() }}, ", binding, self.variable_tracker.get_var_name(variable_id.unwrap())));
			//wgpu::BindingResource::Buffer(wgpu::BufferBinding{{buffer : & {}, offset : 0, size : None}})
		}
		self.code_writer.write("];\n".to_string());
		write!(self.code_writer, "let bind_group = instance.state.get_device_mut().create_bind_group(& wgpu::BindGroupDescriptor {{label : None, layout : & instance.static_bind_group_layout_{}, entries : & entries}});\n", invocation_id);
		write!(self.code_writer, "let pipeline = & instance.static_pipeline_{};\n", invocation_id);

		//output_staging_variables.into_boxed_slice()
	}

	fn begin_command_encoding(&mut self)
	{
		self.code_writer.write("let mut command_encoder = instance.state.get_device_mut().create_command_encoder(& wgpu::CommandEncoderDescriptor {label : None});\n".to_string());
	}

	fn end_command_encoding(&mut self) -> CommandBufferId
	{
		let command_buffer_id = self.next_command_buffer_id;
		self.next_command_buffer_id.0 += 1;
		self.code_writer.write(format!("let command_buffer_{} = command_encoder.finish();\n", command_buffer_id.0));
		return command_buffer_id;
	}

	fn enqueue_command_buffer(&mut self, command_buffer_id : CommandBufferId)
	{
		if self.active_submission_encoding_state.is_none()
		{
			self.active_submission_encoding_state = Some(Default::default());
		}

		if let Some(submission_encoding_state) = self.active_submission_encoding_state.as_mut()
		{
			submission_encoding_state.command_buffer_ids.push(command_buffer_id)
		}
	}

	fn reset_pipeline(&mut self)
	{
		self.active_external_gpu_function_id = None;
		self.active_shader_module_key = None;
	}

	pub fn require_local(&self, variable_ids : &[VarId])
	{
		/*for variable_id in variable_ids.iter()
		{
			match self.variable_tracker.variable_kinds[variable_id]
			{
				//VariableState::InEncoding => self.flush_submission(),
				_ => ()
			}

			match self.variable_tracker.variable_kinds[variable_id]
			{
				VariableState::Local => (),
				VariableState::OnGpu => panic!("Not already local"),
				_ => panic!("Unimplemented")
			}
		}*/
	}

	pub fn require_on_gpu(&self, variable_ids : &[VarId])
	{
		/*for variable_id in variable_ids.iter()
		{
			match self.variable_tracker.variable_states[variable_id]
			{
				VariableState::InEncoding => (),
				VariableState::Local => panic!("Not already on gpu"),
				VariableState::OnGpu => (),
				_ => panic!("Unimplemented")
			}
		}*/
	}

	fn generate_compute_dispatch(&mut self, external_function_id : ir::ExternalGpuFunctionId, dimension_vars : &[VarId; 3], argument_vars : &[VarId], output_vars : &[VarId])
	{
		self.require_local(dimension_vars);
		self.require_on_gpu(argument_vars);

		self.set_active_external_gpu_function(external_function_id);
		//let output_staging_variables = 
		self.set_active_bindings(argument_vars, output_vars);
		
		self.begin_command_encoding();

		let external_gpu_function = & self.native_interface.external_gpu_functions[& external_function_id];
		assert_eq!(external_gpu_function.input_types.len(), argument_vars.len());
		//let mut output_variables = Vec::<usize>::new();
		self.code_writer.write(format!("let ("));
		//self.code_writer.write(format!("let (old_command_buffer_{}, ", command_buffer_id));
		for output_index in 0 .. external_gpu_function.output_types.len()
		{
			//let var_id = self.variable_tracker.generate();
			//output_variables.push(var_id);
			let var_id = output_vars[output_index];
			self.code_writer.write(format!("{}, ", self.variable_tracker.get_var_name(var_id)));
		}
		self.code_writer.write(format!(") = "));

		self.code_writer.write("{\n".to_string());
		
		self.code_writer.write_str("{\n");
		self.code_writer.write("let mut compute_pass = command_encoder.begin_compute_pass(& wgpu::ComputePassDescriptor {label : None});\n".to_string());
		self.code_writer.write("compute_pass.set_pipeline(& pipeline);\n".to_string());
		self.code_writer.write("compute_pass.set_bind_group(0, & bind_group, & []);\n".to_string());
		self.code_writer.write(format!("compute_pass.dispatch({}.try_into().unwrap(), {}.try_into().unwrap(), {}.try_into().unwrap());\n", self.variable_tracker.get_var_name(dimension_vars[0]), self.variable_tracker.get_var_name(dimension_vars[1]), self.variable_tracker.get_var_name(dimension_vars[2])));
		self.code_writer.write_str("}\n");

		//self.code_writer.write("let command_buffer = command_encoder.finish();\n".to_string());
		//self.code_writer.write("queue.submit([command_buffer]);\n".to_string());
		//self.code_writer.write(format!("device.poll(wgpu::Maintain::Wait);\n"));
		//self.code_writer.write("futures::executor::block_on(queue.on_submitted_work_done());\n".to_string());

		let mut output_temp_variables = Vec::<VarId>::new();
		for output_index in 0 .. external_gpu_function.output_types.len()
		{
			let staging_var_id = output_vars[output_index];
			let type_id = external_gpu_function.output_types[output_index];
			let range_var_id = self.variable_tracker.generate();
			let output_temp_var_id = self.variable_tracker.generate();
			let slice_var_id = self.variable_tracker.generate();
			let future_var_id = self.variable_tracker.generate();
			//output_temp_variables.push(output_temp_var_id);
			let type_binding_info = self.get_type_binding_info(type_id); 
			let type_name = self.get_type_name(type_id);

			output_temp_variables.push(staging_var_id);
			
			/*self.code_writer.write(format!("let var_{} = var_{}.slice(0..);\n", slice_var_id, staging_var_id));
			self.code_writer.write(format!("let var_{} = var_{}.map_async(wgpu::MapMode::Read);\n", future_var_id, slice_var_id));
			self.code_writer.write(format!("device.poll(wgpu::Maintain::Wait);\n"));
			self.code_writer.write(format!("futures::executor::block_on(var_{});;\n", future_var_id));
			self.code_writer.write(format!("let var_{} = var_{}.get_mapped_range();\n", range_var_id, slice_var_id));
			self.code_writer.write(format!("let var_{} = * unsafe {{ std::mem::transmute::<* const u8, & {}>(var_{}.as_ptr()) }};\n", output_temp_var_id, type_name, range_var_id));*/
		}

		self.code_writer.write(format!("("));
		for output_temp_var_id in output_temp_variables.iter()
		{
			self.code_writer.write(format!("{}, ", self.variable_tracker.get_var_name(* output_temp_var_id)));
		}
		self.code_writer.write(format!(")"));

		self.code_writer.write("};\n".to_string());

		let command_buffer_id = self.end_command_encoding();
		self.enqueue_command_buffer(command_buffer_id);

		for var_id in output_vars.iter()
		{
			// These are wrong
			//self.variable_tracker.transition_to_queue(* var_id);
			//self.variable_tracker.transition_to_on_gpu(* var_id);
			//self.variable_tracker.transition_to_local(* var_id);
		}
	}

	pub fn flush_submission(&mut self) -> SubmissionId
	{
		let mut active_submission_encoding_state = None;
		std::mem::swap(&mut self.active_submission_encoding_state, &mut active_submission_encoding_state);

		if let Some(submission_encoding_state) = active_submission_encoding_state
		{
			if submission_encoding_state.command_buffer_ids.len() > 0
			{
				self.code_writer.write("instance.state.get_queue_mut().submit([".to_string());
				for & command_buffer_id in submission_encoding_state.command_buffer_ids.iter()
				{
					self.code_writer.write(format!("command_buffer_{}, ", command_buffer_id.0));
				}
				self.code_writer.write("]);\n".to_string());
			}

			/*for command in submission_encoding_state.commands.iter()
			{
				match command
				{
					Command::DispatchCompute{external_function_id, dimension_vars, argument_vars, output_vars} =>
					{
						self.generate_compute_dispatch(* external_function_id, dimension_vars, argument_vars, output_vars);
					}
				}
			}*/

		}

		//self.active_submission_encoding_state = None;
		let submission_id = self.submission_queue.next_submission_id;
		self.submission_queue.next_submission_id.0 += 1;

		//self.code_writer.write(format!("let future_var_{} = instance.state.get_queue_mut().on_submitted_work_done();\n", submission_id.0));

		submission_id
	}

	pub fn sync_submission(&mut self, submission_id : SubmissionId)
	//pub fn sync_submissions(&mut self)
	{
		//self.code_writer.write("futures::executor::block_on(queue.on_submitted_work_done());\n".to_string());
		//self.code_writer.write(format!("instance.state.get_device_mut().poll(wgpu::Maintain::Wait);\n"));
		//self.code_writer.write(format!("futures::executor::block_on(future_var_{});\n", submission_id.0));
		//self.code_writer.write("futures::executor::block_on(queue.on_submitted_work_done());\n".to_string());
	}

	pub fn encode_gpu_fence(&mut self) -> FenceId
	{
		let fence_id = self.submission_queue.next_fence_id;
		self.submission_queue.next_fence_id.0 += 1;
		self.code_writer.write(format!("let future_var_{} = instance.state.get_queue_mut().on_submitted_work_done();\n", fence_id.0));
		fence_id
	}

	pub fn sync_gpu_fence(&mut self, fence_id : FenceId)
	{
		self.code_writer.write(format!("instance.state.get_device_mut().poll(wgpu::Maintain::Wait);\n"));
		self.code_writer.write(format!("futures::executor::block_on(future_var_{});\n", fence_id.0));
	}

	pub fn insert_comment(&mut self, comment_string : &str)
	{
		self.code_writer.write(format!("// {}\n", comment_string));
	}

	fn write_states(&mut self)
	{
		//self.state_code_writer
		// todo: chec if removing "use caiman_rt::wgpu;" is going to break things
		let code_string = "
		//use caiman_rt::wgpu;
		
		/*pub struct CpuFunctionInvocationState<'parent>
		{
			parent_state : & 'parent mut dyn caiman_rt::State
		}*/
";

		write!(self.state_code_writer, "{}", code_string);
	}

	pub fn begin_pipeline(&mut self, pipeline_name : &str)
	{
		self.reset_pipeline();
		self.active_closures.clear();
		self.active_yield_point_ids.clear();
		self.active_dispatchers.clear();

		self.active_pipeline_name = Some(String::from(pipeline_name));
		self.code_writer.begin_module(pipeline_name);
		write!(self.code_writer, "use super::*;\n");

		self.code_writer.begin_module("outputs");
		{
			for (_, external_cpu_function) in self.native_interface.external_cpu_functions.iter()
			{
				let mut tuple_fields = Vec::<ffi::TypeId>::new();
				for (output_index, output_type) in external_cpu_function.output_types.iter().enumerate()
				{
					tuple_fields.push(*output_type);
				}
				//let type_id = self.native_interface.types.create(ffi::Type::Tuple{fields : tuple_fields.into_boxed_slice()});
				//self.generate_type_definition(ffi::TypeId(type_id));
				//write!(self.code_writer, "pub type {} = super::super::{};\n", external_cpu_function.name, self.get_type_name(ffi::TypeId(type_id)));
				write!(self.code_writer, "pub type {} = {};\n", external_cpu_function.name, self.get_tuple_definition_string(tuple_fields.as_slice()));
			}
		}
		self.code_writer.end_module();

		self.code_writer.write(format!("pub trait CpuFunctions\n{{\n"));
		for (_, external_cpu_function) in self.native_interface.external_cpu_functions.iter()
		{
			self.code_writer.write(format!("\tfn {}(&self, state : &mut caiman_rt::State", external_cpu_function.name));
			for (input_index, input_type) in external_cpu_function.input_types.iter().enumerate()
			{
				//self.generate_type_definition(* input_type);
				self.code_writer.write(format!(", _ : {}", self.get_type_name(*input_type)));
			}
			self.code_writer.write(format!(") -> outputs::{};\n", external_cpu_function.name));
		}
		self.code_writer.write(format!("}}\n"));
	}

	/*pub fn begin_oneshot_entry_funclet(&mut self, input_types : &[ffi::TypeId], output_types : &[ffi::TypeId]) -> Box<[usize]>
	{
		self.code_writer.begin_module("pipeline_outputs");
		{
			let mut tuple_fields = Vec::<ffi::TypeId>::new();
			for output_index in 0 .. output_types.len()
			{
				let output_type = output_types[output_index];
				tuple_fields.push(output_type);
				self.generate_type_definition(output_type);
			}
			let type_id = self.native_interface.types.create(ffi::Type::Tuple{fields : tuple_fields.into_boxed_slice()});
			self.generate_type_definition(type_id);
			write!(self.code_writer, "pub type {} = super::super::{};\n", self.active_pipeline_name.as_ref().unwrap().as_str(), self.get_type_name(type_id));
		}
		self.code_writer.end_module();

		let mut argument_variable_ids = Vec::<usize>::new();
		self.code_writer.write(format!("pub fn run<F>(state : &mut caiman_rt::State, cpu_functions : & F"));
		//self.code_strings.push("(".to_string());
		for (input_index, input_type) in input_types.iter().enumerate()
		{
			self.code_writer.write(", ".to_string());

			//let variable_id = self.variable_tracker.generate();
			let variable_id = self.variable_tracker.create_local_data(* input_type);
			argument_variable_ids.push(variable_id);
			let type_name = self.get_type_name(*input_type);
			self.code_writer.write(format!("var_{} : {}", variable_id, type_name));

			/*if input_index + 1 < funclet.input_types.len()
			{
				self.code_strings.push(", ".to_string());
			}*/
		}

		self.code_writer.write(format!(" ) -> pipeline_outputs::{}\n\twhere F : CpuFunctions", self.active_pipeline_name.as_ref().unwrap().as_str()));
		self.code_writer.write("\n{\n\tuse std::convert::TryInto;\n".to_string());
		argument_variable_ids.into_boxed_slice()
	}*/

	pub fn begin_funclet(&mut self, funclet_id : ir::FuncletId, input_types : &[ffi::TypeId], output_types : &[ffi::TypeId]) -> Box<[VarId]>
	{
		// Temporarily need to do this until pipelines are constructed correctly
		self.reset_pipeline();

		//self.code_writer.begin_module("funclet_outputs");
		let funclet_result_type_ids = {
			let mut tuple_fields = Vec::<ffi::TypeId>::new();
			for output_index in 0 .. output_types.len()
			{
				let output_type = output_types[output_index];
				//self.generate_type_definition(output_type);
				tuple_fields.push(output_type);
			}
			/*let type_id = self.native_interface.types.create(ffi::Type::Tuple{fields : tuple_fields.into_boxed_slice()});
			self.generate_type_definition(ffi::TypeId(type_id));
			//write!(self.code_writer, "pub type {} = super::super::{};\n", self.active_pipeline_name.as_ref().unwrap().as_str(), self.get_type_name(type_id));
			type_id*/
			tuple_fields.into_boxed_slice()
		};
		//self.code_writer.end_module();

		self.active_funclet_result_type_ids = Some(funclet_result_type_ids.clone());

		//self.code_writer.write(format!("pub struct Funclet{}Result<'state, 'cpu_functions, Callbacks : CpuFunctions> {{instance : Instance<'state, 'cpu_functions, F>, intermediates : super::{}}}", funclet_id, self.get_type_name(funclet_result_type_id)));
		//self.code_writer.write(format!("pub struct Funclet{}<'state, 'cpu_functions, F : CpuFunctions> {{instance : Instance<'state, 'cpu_functions, F>, intermediates : super::{}}}", funclet_id, self.get_type_name(funclet_result_type_id)));

		//self.code_writer.write(format!("}}"));

		//self.code_writer.write(format!("impl<'state,  'cpu_functions, F : CpuFunctions> Funclet{}<'state,  'cpu_functions, F>\n{{\n", funclet_id));

		let mut next_trait_index = 0usize;

		let mut argument_variable_ids = Vec::<VarId>::new();
		write!(self.code_writer, "fn funclet{}_func<'state,  'cpu_functions, 'callee, Callbacks : CpuFunctions>(instance : Instance<'state, 'cpu_functions, Callbacks>, join_stack : &mut caiman_rt::JoinStack<'callee>", funclet_id);

		/*for (input_index, input_type) in input_types.iter().enumerate()
		{
			self.code_writer.write(", ".to_string());
			match & self.native_interface.types[input_type]
			{
				ffi::Type::Slot{ .. } =>
				{

				}
				ffi::Type::SchedulingJoin { input_types, output_types, extra } =>
				{
					write!(self.code_writer, ", F{} : FnOnce(", next_trait_index, next_trait_index);
					for (input_index, input_type) in input_types.iter().enumerate()
					{
						write!(self.code_writer, "{}", self.get_type_name(* input_type));

						if input_index + 1 < input_types.len()
						{
							write!(self.code_writer, ", ");
						}
					}
					write!(self.code_writer, ") -> (");
					for (output_index, output_type) in output_types.iter().enumerate()
					{
						write!(self.code_writer, "{}", self.get_type_name(* output_type));

						if output_index + 1 < output_types.len()
						{
							write!(self.code_writer, ", ");
						}
					}
					write!(self.code_writer, ") ");
					//self.get_type_name(funclet_result_type_id)
					next_trait_index += 1;
				}
				_ => panic!("Unknown type")
			}
		}*/

		//self.code_strings.push("(".to_string());
		for (input_index, input_type) in input_types.iter().enumerate()
		{
			write!(self.code_writer, ", ");
			/*match & self.native_interface.types[input_type]
			{
				ffi::Type::Slot{ .. } =>
				{
					self.generate_type_definition(* input_type);

					//let variable_id = self.variable_tracker.generate();
					let variable_id = self.variable_tracker.create_local(* input_type);
					argument_variable_ids.push(variable_id);
					let type_name = self.get_type_name(*input_type);
					self.code_writer.write(format!("var_{} : {}", variable_id, type_name));
				}
				ffi::Type::SchedulingJoin { input_types, output_types, extra } =>
				{

				}
				_ => panic!("Unknown type")
			}*/

			//self.generate_type_definition(* input_type);

			//let variable_id = self.variable_tracker.generate();
			let variable_id = self.variable_tracker.create_local_data(* input_type);
			argument_variable_ids.push(variable_id);
			let type_name = self.get_type_name(*input_type);
			let is_mutable = match & self.native_interface.types[& input_type.0]
			{
				ffi::Type::GpuBufferAllocator => true,
				_ => false
			};
			if is_mutable
			{
				self.code_writer.write(format!("mut {} : {}", self.variable_tracker.get_var_name(variable_id), type_name));
			}
			else
			{
				self.code_writer.write(format!("{} : {}", self.variable_tracker.get_var_name(variable_id), type_name));
			}
		}

		write!(self.code_writer, " ) -> FuncletResult<'state, 'cpu_functions, 'callee, Callbacks, {}>", self.get_tuple_definition_string(& funclet_result_type_ids));
		self.code_writer.write("\n{\n\tuse std::convert::TryInto;\n".to_string());

		self.active_funclet_state = Some(ActiveFuncletState{funclet_id, result_type_ids : funclet_result_type_ids, next_funclet_ids : None, capture_count : 0, output_count : 0, output_type_ids : output_types.to_vec().into_boxed_slice(), next_funclet_input_types : None});

		argument_variable_ids.into_boxed_slice()
	}

	/*pub fn emit_oneshot_pipeline_entry_point(&mut self, funclet_id : ir::FuncletId, input_types : &[ffi::TypeId], output_types : &[ffi::TypeId])
	{
		self.code_writer.begin_module("pipeline_outputs");
		{
			let mut tuple_fields = Vec::<ffi::TypeId>::new();
			for output_index in 0 .. output_types.len()
			{
				let output_type = output_types[output_index];
				tuple_fields.push(output_type);
			}
			//let type_id = self.native_interface.types.create(ffi::Type::Tuple{fields : tuple_fields.into_boxed_slice()});
			//self.generate_type_definition(type_id);
			write!(self.code_writer, "pub type {} = {};\n", self.active_pipeline_name.as_ref().unwrap().as_str(), self.get_tuple_definition_string(tuple_fields.as_slice()));
		}
		self.code_writer.end_module();

		let mut argument_variable_ids = Vec::<VarId>::new();
		self.code_writer.write(format!("pub fn run<F>(state : &mut caiman_rt::State, cpu_functions : & F"));
		//self.code_strings.push("(".to_string());
		for (input_index, input_type) in input_types.iter().enumerate()
		{
			self.code_writer.write(", ".to_string());

			//let variable_id = self.variable_tracker.generate();
			let variable_id = self.variable_tracker.create_local_data(* input_type);
			argument_variable_ids.push(variable_id);
			let type_name = self.get_type_name(*input_type);
			self.code_writer.write(format!("{} : {}", self.variable_tracker.get_var_name(variable_id), type_name));

			/*if input_index + 1 < funclet.input_types.len()
			{
				self.code_strings.push(", ".to_string());
			}*/
		}

		self.code_writer.write(format!(" ) -> pipeline_outputs::{}\n\twhere F : CpuFunctions", self.active_pipeline_name.as_ref().unwrap().as_str()));
		self.code_writer.write("\n{\n\tuse std::convert::TryInto;\n\tlet mut instance = Instance::new(state, cpu_functions);\n".to_string());
		//self.code_writer.write("{\n".to_string());
		write!(self.code_writer, "\tlet result = Funclet{}::new(instance", funclet_id);
		for (_, var_id) in argument_variable_ids.iter().enumerate()
		{
			write!(self.code_writer, ", {}", self.variable_tracker.get_var_name(*var_id));
		}
		write!(self.code_writer, ").complete();\n");
		write!(self.code_writer, "return pipeline_outputs::{} {{", self.active_pipeline_name.as_ref().unwrap().as_str());
		for (output_index, output_type) in output_types.iter().enumerate()
		{
			if output_index != 0
			{
				write!(self.code_writer, ", ");
			}
			write!(self.code_writer, "field_{} : result.field_{}", output_index, output_index);
		}
		write!(self.code_writer, "}};\n}}\n");
	}*/

	fn emit_pipeline_entry_point(&mut self, funclet_id : ir::FuncletId, input_types : &[ffi::TypeId], output_types : &[ffi::TypeId], yield_points_opt : Option<& [(ir::PipelineYieldPointId, YieldPoint)]>)
	{
		//Option<(ir::PipelineYieldPointId, Box<[ffi::TypeId])>>
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
		write!(self.code_writer, "type PipelineOutputTuple<'callee> = {};\n", pipeline_output_tuple_string);

		write!(self.code_writer, "enum FuncletResultIntermediates<Intermediates>\n{{ Return(Intermediates), ");
		let mut yield_point_ref_map = HashMap::<ir::PipelineYieldPointId, & YieldPoint>::new();
		if let Some(yield_points) = yield_points_opt
		{
			for (yield_point_id, yield_point) in yield_points.iter()
			{
				yield_point_ref_map.insert(* yield_point_id, yield_point);
				write!(self.code_writer, "Yield{}{{ yielded : {} }}, ", yield_point_id.0, self.get_tuple_definition_string(& yield_point.yielded_types));
			}
		}
		write!(self.code_writer, "}}");


		write!(self.code_writer, "impl<'state, 'cpu_functions, 'callee, Callbacks : CpuFunctions, Intermediates> FuncletResult<'state, 'cpu_functions, 'callee, Callbacks, Intermediates>\n{{\n");
		if let Some(yield_points) = yield_points_opt
		{
			for (yield_point_id, yield_point) in yield_points.iter()
			{
				write!(self.code_writer, "pub fn yielded_at_{}(&self) -> Option<& {}> {{ if let FuncletResultIntermediates::Yield{}{{yielded}} = & self.intermediates {{ Some(yielded) }} else {{ None }} }}\n", yield_point.name, self.get_tuple_definition_string(& yield_point.yielded_types), yield_point_id.0);
				
				//let dispatcher_id = self.lookup_dispatcher_id(& yield_point.resuming_types);
				//write!(self.code_writer, "pub fn resume_at_{}(self) -> "
			}
		}
		write!(self.code_writer, "}}");

		/*for yield_point_id in self.active_yield_point_ids.iter()
		{
			write!(self.code_writer, "fn pop_and_dispatch_join_from_yield_at_{}<'state, 'cpu_functions, 'callee, Callbacks : CpuFunctions, Intermediates>(instance : Instance<'state, 'cpu_functions, Callbacks>, join_stack : &mut JoinStack<'callee>", yield_point_id);

			let yield_point = yield_point_ref_map[yield_point_id];

			for (resuming_argument_index, resuming_type) in yield_point.resuming_types.iter().enumerate()
			{
				write!(self.code_writer, ", arg_{} : {}", resuming_argument_index, self.get_type_name(* resuming_type));
			}
			write!(self.code_writer, " ) -> FuncletResult<'state, 'cpu_functions, 'callee, Callbacks, {}>\n", pipeline_output_tuple_string);
			write!(self.code_writer, "{{\n",);
			write!(self.code_writer, "match ",);

			for ((funclet_id, capture_count), closure) in self.active_closures.iter()
			{
				write!(self.code_writer, "Funclet{}Capturing{}, ", funclet_id, capture_count);
			}
			
			write!(self.code_writer, "}}");
		}*/

		/**/
		//panic!("Implement closure table");

		// Write the instance state
		write!(self.code_writer, "pub struct Instance<'state, 'cpu_functions, F : CpuFunctions>{{state : & 'state mut dyn caiman_rt::State, cpu_functions : & 'cpu_functions F");

		for (shader_module_key, shader_module) in self.shader_modules.iter()
		{
			write!(self.code_writer, ", {} : wgpu::ShaderModule", shader_module_key.instance_field_name());
		}

		for (gpu_function_invocation_id, gpu_function_invocation) in self.gpu_function_invocations.iter().enumerate()
		{
			write!(self.code_writer, ", static_bind_group_layout_{} : wgpu::BindGroupLayout, static_pipeline_layout_{} : wgpu::PipelineLayout, static_pipeline_{} : wgpu::ComputePipeline", gpu_function_invocation_id, gpu_function_invocation_id, gpu_function_invocation_id);
		}

		write!(self.code_writer, "}}\n");

		write!(self.code_writer, "{}", "
		impl<'state, 'cpu_functions, F : CpuFunctions> Instance<'state, 'cpu_functions, F> 
		{
			pub fn new(state : & 'state mut dyn caiman_rt::State, cpu_functions : & 'cpu_functions F) -> Self
			{
				");
		
		for (shader_module_key, shader_module) in self.shader_modules.iter_mut()
		{
			write!(self.code_writer, "let {} = state.get_device_mut().create_shader_module(& wgpu::ShaderModuleDescriptor {{ label : None, source : wgpu::ShaderSource::Wgsl(std::borrow::Cow::from(\"{}\"))}});\n", shader_module_key.instance_field_name(), shader_module.compile_wgsl_text().as_str());
		}

		for (gpu_function_invocation_id, gpu_function_invocation) in self.gpu_function_invocations.iter().enumerate()
		{
			self.code_writer.write("let bind_group_layout_entries = [".to_string());
			for (binding, (_input_opt, output_opt)) in gpu_function_invocation.bindings.iter()
			{
				let is_read_only : bool = output_opt.is_none();
				self.code_writer.write("wgpu::BindGroupLayoutEntry { ".to_string());
				self.code_writer.write(format!("binding : {}, visibility : wgpu::ShaderStages::COMPUTE, ty : wgpu::BindingType::Buffer{{ ty : wgpu::BufferBindingType::Storage {{ read_only : {} }}, has_dynamic_offset : false, min_binding_size : None}}, count : None", binding, is_read_only));
				self.code_writer.write(" }, ".to_string());
			}
			self.code_writer.write("];\n".to_string());

			write!(self.code_writer, "let static_bind_group_layout_{} = state.get_device_mut().create_bind_group_layout(& wgpu::BindGroupLayoutDescriptor {{ label : None, entries : & bind_group_layout_entries}});\n", gpu_function_invocation_id);

			write!(self.code_writer, "let static_pipeline_layout_{} = state.get_device_mut().create_pipeline_layout(& wgpu::PipelineLayoutDescriptor {{ label : None, bind_group_layouts : & [& static_bind_group_layout_{}], push_constant_ranges : & []}});\n", gpu_function_invocation_id, gpu_function_invocation_id);
			write!(self.code_writer, "let static_pipeline_{} = state.get_device_mut().create_compute_pipeline(& wgpu::ComputePipelineDescriptor {{label : None, layout : Some(& static_pipeline_layout_{}), module : & {}, entry_point : & \"main\"}});\n", gpu_function_invocation_id, gpu_function_invocation_id, gpu_function_invocation.shader_module_key.instance_field_name());
		}

		write!(self.code_writer, "{}", "
				Self{state, cpu_functions");

		for (shader_module_key, shader_module) in self.shader_modules.iter()
		{
			write!(self.code_writer, ", {}", shader_module_key.instance_field_name());
		}

		for (gpu_function_invocation_id, gpu_function_invocation) in self.gpu_function_invocations.iter().enumerate()
		{
			write!(self.code_writer, ", static_bind_group_layout_{}, static_pipeline_layout_{}, static_pipeline_{}", gpu_function_invocation_id, gpu_function_invocation_id, gpu_function_invocation_id);
		}

		write!(self.code_writer, "{}", "}
			}

		");

		write!(self.code_writer, "{}", "
		}
		");


		write!(self.code_writer, "impl<'state, 'cpu_functions, F : CpuFunctions> Instance<'state, 'cpu_functions, F>\n");
		write!(self.code_writer, "{{\n");
		{
			write!(self.code_writer, "pub fn start<'callee>(self, join_stack : &mut caiman_rt::JoinStack<'callee>");
			for (input_index, input_type) in input_types.iter().enumerate()
			{
				write!(self.code_writer, ", arg_{} : {}", input_index, self.get_type_name(* input_type));
			}
			write!(self.code_writer, ") -> FuncletResult<'state, 'cpu_functions, 'callee, F, PipelineOutputTuple<'callee>> {{ funclet{}_func(self, join_stack", funclet_id);
			for (input_index, input_type) in input_types.iter().enumerate()
			{
				write!(self.code_writer, ", arg_{}", input_index);
			}
			write!(self.code_writer, ") }}", );
		}
		if let Some(yield_points) = yield_points_opt
		{
			for (yield_point_id, yield_point) in yield_points.iter()
			{
				//write!(self.code_writer, "pub fn yielded_at_{}(&self) -> Option<& {}> {{ if let FuncletResultIntermediates::Yield{}{{yielded}} = & self.intermediates {{ Some(yielded) }} else {{ None }} }}\n", yield_point.name, self.get_tuple_definition_string(& yield_point.yielded_types), yield_point_id.0);
				
				let dispatcher_id = self.lookup_dispatcher_id(& yield_point.resuming_types);
				write!(self.code_writer, "pub fn resume_at_{}<'callee>(self, join_stack : &mut caiman_rt::JoinStack<'callee>", yield_point.name);
				for (resuming_argument_index, resuming_type) in yield_point.resuming_types.iter().enumerate()
				{
					write!(self.code_writer, ", arg_{} : {}", resuming_argument_index, self.get_type_name(* resuming_type));
				}
				write!(self.code_writer, ") -> FuncletResult<'state, 'cpu_functions, 'callee, F, PipelineOutputTuple<'callee>> {{ pop_join_and_dispatch_at_{}::<F, PipelineOutputTuple<'callee>>(self, join_stack", dispatcher_id.0);
				for (resuming_argument_index, resuming_type) in yield_point.resuming_types.iter().enumerate()
				{
					write!(self.code_writer, ", arg_{}", resuming_argument_index);
				}
				write!(self.code_writer, ") }}\n");

				
			}
		}
		write!(self.code_writer, "}}\n");

		// Generate closures all the way at the end

		write!(self.code_writer, "#[derive(Debug)] enum ClosureHeader {{ Root, ");
		for ((funclet_id, capture_count), closure) in self.active_closures.iter()
		{
			write!(self.code_writer, "Funclet{}Capturing{}, ", funclet_id, capture_count);
		}
		write!(self.code_writer, "}}\n");


		for ((funclet_id, capture_count), closure) in self.active_closures.iter()
		{
			write!(self.code_writer, "type Funclet{}Capturing{}CapturedTuple<'callee> = {};\n", funclet_id, capture_count, self.get_tuple_definition_string(& closure.capture_types));
		}

		for (argument_types, dispatcher) in self.active_dispatchers.iter()
		{
			write!(self.code_writer, "fn pop_join_and_dispatch_at_{}<'state, 'cpu_functions, 'callee, Callbacks : CpuFunctions, Intermediates>(instance : Instance<'state, 'cpu_functions, Callbacks>, join_stack : &mut caiman_rt::JoinStack<'callee>", dispatcher.dispatcher_id.0);

			for (resuming_argument_index, resuming_type) in argument_types.iter().enumerate()
			{
				write!(self.code_writer, ", arg_{} : {}", resuming_argument_index, self.get_type_name(* resuming_type));
			}
			write!(self.code_writer, " ) -> FuncletResult<'state, 'cpu_functions, 'callee, Callbacks, {}>\n", pipeline_output_tuple_string);
			write!(self.code_writer, "{{\n",);

			write!(self.code_writer, "let closure_header = unsafe {{ join_stack.pop_unsafe_unaligned::<ClosureHeader>().unwrap() }}; match closure_header {{\n",);

			for ((funclet_id, capture_count), closure) in self.active_closures.iter()
			{
				if closure.dispatcher_id != dispatcher.dispatcher_id
				{
					continue;
				}

				//write!(self.code_writer, "ClosureHeader::Funclet{}Capturing{} => {{ let join_captures = unsafe {{ join_stack.pop_unsafe_unaligned::<{}>().unwrap() }}; funclet{}_func(instance, join_stack", funclet_id, capture_count, self.get_tuple_definition_string(& closure.capture_types), funclet_id);
				write!(self.code_writer, "ClosureHeader::Funclet{}Capturing{} => {{ let join_captures = unsafe {{ join_stack.pop_unsafe_unaligned::<Funclet{}Capturing{}CapturedTuple<'callee>>().unwrap() }}; funclet{}_func(instance, join_stack", funclet_id, capture_count, funclet_id, capture_count, funclet_id);
				for capture_index in 0 .. * capture_count
				{
					write!(self.code_writer, ", join_captures.{}", capture_index);
				}
				for (argument_index, _argument_type) in argument_types.iter().enumerate()
				{
					write!(self.code_writer, ", arg_{}", argument_index);
				}
				write!(self.code_writer, ") }}\n");
			}
			
			write!(self.code_writer, "_ => panic!(\"Dispatcher cannot dispatch given closure {{:?}}\", closure_header), }} }}", );
		}
	}


	pub fn emit_oneshot_pipeline_entry_point(&mut self, funclet_id : ir::FuncletId, input_types : &[ffi::TypeId], output_types : &[ffi::TypeId])
	{
		self.emit_pipeline_entry_point(funclet_id, input_types, output_types, None)
	}

	pub fn emit_yieldable_pipeline_entry_point(&mut self, funclet_id : ir::FuncletId, input_types : &[ffi::TypeId], output_types : &[ffi::TypeId], yield_points : & [(ir::PipelineYieldPointId, YieldPoint)])
	{
		self.emit_pipeline_entry_point(funclet_id, input_types, output_types, Some(yield_points))
	}

	pub fn build_indirect_stack_jump_to_popped_serialized_join(&mut self, argument_var_ids : &[VarId], argument_types : & [ffi::TypeId])
	{
		let dispatcher_id = self.lookup_dispatcher_id(argument_types);
		write!(self.code_writer, "return pop_join_and_dispatch_at_{}::<Callbacks, PipelineOutputTuple<'callee>>", dispatcher_id.0);
		write!(self.code_writer, "(instance, join_stack");
		for (argument_index, var_id) in argument_var_ids.iter().enumerate()
		{
			write!(self.code_writer, ", {}", self.variable_tracker.get_var_name(* var_id));
		}
		write!(self.code_writer, ")\n");
	}

	pub fn build_return(&mut self, output_var_ids : &[VarId])
	{
		//self.get_type_name(self.active_funclet_result_type_id.unwrap())
		//self.active_funclet_state.as_ref().unwrap().funclet_id
		if let Some(result_type_ids) = & self.active_funclet_result_type_ids
		{
			let result_type_ids = result_type_ids.clone(); // Make a copy for now to satisfy the borrowchecking gods...
			let dispatcher_id = self.lookup_dispatcher_id(& result_type_ids);
			write!(self.code_writer, "if join_stack.used_bytes().len() > 0 {{ ");
			write!(self.code_writer, "return pop_join_and_dispatch_at_{}::<Callbacks, PipelineOutputTuple<'callee>>", dispatcher_id.0);//::<'state, 'cpu_functions, 'callee>
			write!(self.code_writer, "(instance, join_stack");
			for (return_index, var_id) in output_var_ids.iter().enumerate()
			{
				write!(self.code_writer, ", {}", self.variable_tracker.get_var_name(* var_id));
			}
			write!(self.code_writer, ") }}");
		}
		write!(self.code_writer, "return FuncletResult::<'state, 'cpu_functions, 'callee, Callbacks, _> {{instance, phantom : std::marker::PhantomData::<& 'callee ()>, intermediates : FuncletResultIntermediates::<_>::Return((");
		for (return_index, var_id) in output_var_ids.iter().enumerate()
		{
			write!(self.code_writer, "{}, ", self.variable_tracker.get_var_name(* var_id));
		}
		write!(self.code_writer, "))}};");
	}

	pub fn build_yield(&mut self, yield_point_id : ir::PipelineYieldPointId, yielded_var_ids : &[VarId])
	{
		//self.get_type_name(self.active_funclet_result_type_id.unwrap())
		//self.active_funclet_state.as_ref().unwrap().funclet_id
		write!(self.code_writer, "return FuncletResult::<'state, 'cpu_functions, 'callee, Callbacks, _> {{instance, phantom : std::marker::PhantomData::<& 'callee ()>, intermediates : FuncletResultIntermediates::<_>::Yield{}{{ yielded : (", yield_point_id.0);
		for (return_index, var_id) in yielded_var_ids.iter().enumerate()
		{
			write!(self.code_writer, "{}, ", self.variable_tracker.get_var_name(* var_id));
		}
		write!(self.code_writer, ") }} }};");
	}

	/*pub fn build_yield(&mut self, next_funclet_ids : &[ir::FuncletId], next_funclet_input_types : Box<[Box<[ffi::TypeId]>]>, capture_var_ids : &[VarId], output_var_ids : &[VarId])
	{
		self.active_funclet_state.as_mut().unwrap().next_funclet_ids = Some(next_funclet_ids.to_vec().into_boxed_slice());

		self.require_local(capture_var_ids);
		self.require_local(output_var_ids);
		self.code_writer.write(format!("return Funclet{} {{instance, intermediates : {} {{", self.active_funclet_state.as_ref().unwrap().funclet_id, self.get_tuple_definition_string(self.active_funclet_result_type_ids.as_ref().unwrap())));
		for (return_index, var_id) in capture_var_ids.iter().enumerate()
		{
			self.code_writer.write(format!("field_{} : {}, ", return_index, self.variable_tracker.get_var_name(* var_id)));
		}
		for (return_index, var_id) in output_var_ids.iter().enumerate()
		{
			self.code_writer.write(format!("field_{} : {}, ", return_index + capture_var_ids.len(), self.variable_tracker.get_var_name(* var_id)));
		}
		self.active_funclet_state.as_mut().unwrap().capture_count = capture_var_ids.len();
		self.active_funclet_state.as_mut().unwrap().output_count = output_var_ids.len();
		self.active_funclet_state.as_mut().unwrap().next_funclet_input_types = Some(next_funclet_input_types);
		self.code_writer.write(format!("}}}};"));
	}*/

	//fn build_oneshot_entry_point()

	pub fn end_funclet(&mut self)
	{
		self.code_writer.write("}\n".to_string());

		//self.code_writer.write(format!("pub fn step_{}<'next_state, F>(self, state : & 'next_state, mut super::State, cpu_functions : & F"));

		/*if let Some(active_funclet_state) = & self.active_funclet_state
		{
			if let Some(next_funclet_ids) = & active_funclet_state.next_funclet_ids
			{
				write!(self.code_writer, "\tpub fn get_yielded(self) -> (");

				for index in active_funclet_state.capture_count .. active_funclet_state.output_type_ids.len()
				{
					write!(self.code_writer, "{}, ", self.get_type_name(active_funclet_state.output_type_ids[index - active_funclet_state.capture_count]));
				}

				self.code_writer.write(format!(") {{ ("));

				for index in active_funclet_state.capture_count .. active_funclet_state.output_type_ids.len()
				{
					write!(self.code_writer, "self.intermediates.field_{}, ", index);
				}

				self.code_writer.write(format!(") }}\n"));

				for (funclet_index, next_funclet_id) in next_funclet_ids.iter().enumerate()
				{
					let input_types = & active_funclet_state.next_funclet_input_types.as_ref().unwrap()[funclet_index];

					self.code_writer.write(format!("\tpub fn can_step_{}(&mut self) -> bool {{ true }}", next_funclet_id));

					self.code_writer.write(format!("\tpub fn step_{}(mut self", next_funclet_id));
					//for index in  0 .. active_funclet_state.capture_count
					//{
					//	write!(self.code_writer, ", input_{} : {}", index, active_funclet_state.output_type_ids[index]);
					//}
					//for index in  active_funclet_state.capture_count .. active_funclet_state.output_type_ids.len()
					//{
					//	write!(self.code_writer, ", input_{} : {}", index, active_funclet_state.output_type_ids[index]);
					//}
					//write!(self.code_writer, ")\n{{\n\treturn {}::new(state, cpu_functions");

					//for (index, input_type) in input_types.iter().enumerate()
					for index in active_funclet_state.capture_count .. input_types.len()
					{
						let input_type = & input_types[index];
						write!(self.code_writer, ", input_{} : {}", index, self.get_type_name(*input_type));
					}
					write!(self.code_writer, ") -> Funclet{}<'state, 'cpu_functions, F> \n{{\n\treturn Funclet{}::<'state, 'cpu_functions, F>::new(self.instance", next_funclet_id, next_funclet_id);
					for index in 0 .. active_funclet_state.capture_count
					{
						write!(self.code_writer, ", self.intermediates.field_{}", index);
					}
					for index in active_funclet_state.capture_count .. input_types.len()
					{
						write!(self.code_writer, ", input_{}", index);
					}
					write!(self.code_writer, ");\n\t}}\n");
				}
			}
			else
			{
				self.code_writer.write(format!("\tpub fn complete(self) -> super::{} {{ self.intermediates }}", self.get_type_name(self.active_funclet_result_type_id.unwrap())));
			}
		}

		self.code_writer.write("}\n".to_string());*/
		self.active_funclet_result_type_ids = None;
		self.active_funclet_state = None;
	}

	pub fn end_pipeline(&mut self)
	{
		self.code_writer.end_module();
		self.active_pipeline_name = None;
		self.reset_pipeline();
	}

	fn generate_type_definition(&mut self, type_id : ffi::TypeId)
	{
		if self.has_been_generated.contains(& type_id)
		{
			return;
		}

		self.has_been_generated.insert(type_id);

		let typ = & self.native_interface.types[& type_id.0];
		write!(self.type_code_writer, "// Type #{}: {:?}\n", type_id.0, typ);
		match typ
		{
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
			ffi::Type::Array { element_type, length } => (),
			ffi::Type::Tuple { fields } =>
			{
				write!(self.type_code_writer, "pub type type_{} = (", type_id.0);
				for (index, field_type_id) in fields.iter().enumerate()
				{
					let type_name = self.get_type_name(* field_type_id);
					write!(self.type_code_writer, "{}, ", type_name);
				}
				self.type_code_writer.write_str(");\n");
			}
			ffi::Type::Struct { fields, byte_alignment, byte_size } =>
			{
				write!(self.type_code_writer, "pub struct type_{}", type_id.0);
				self.type_code_writer.write_str("{\n");
				for field in fields.iter()
				{
					let type_name = self.get_type_name(field.type_id);
					write!(self.type_code_writer, "\tpub {} : {},\n", field.name, type_name);
				}
				self.type_code_writer.write_str("}\n\n");
			}
			ffi::Type::GpuBufferRef { element_type } => (),
			ffi::Type::GpuBufferSlice { element_type } => (),
			ffi::Type::GpuBufferAllocator => (),
			/*ffi::Type::Slot { value_type, queue_stage, queue_place } =>
			{
				write!(self.type_code_writer, "pub type type_{} = {};\n", type_id, self.get_type_name(* value_type));

			}
			ffi::Type::SchedulingJoin { input_types, output_types, extra } =>
			{

			}*/
			_ => panic!("Unimplemented type #{}: {:?}", type_id.0, typ),
			//_ => panic!("Unimplemented")
		}
	}

	fn get_type_binding_info(& self, type_id : ffi::TypeId) -> ffi::TypeBindingInfo
	{
		self.native_interface.calculate_type_binding_info(type_id)
	}

	fn get_type_name(& self, type_id : ffi::TypeId) -> String
	{
		match & self.native_interface.types[& type_id.0]
		{
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
			ffi::Type::ConstRef { element_type } => ("& ").to_string() + self.get_type_name(* element_type).as_str(),
			ffi::Type::MutRef { element_type } => ("&mut ").to_string() + self.get_type_name(* element_type).as_str(),
			ffi::Type::ConstSlice { element_type } => ("& [").to_string() + self.get_type_name(* element_type).as_str() + "]",
			ffi::Type::MutSlice { element_type } => ("&mut [").to_string() + self.get_type_name(* element_type).as_str() + "]",
			ffi::Type::Array { element_type, length } => format!("[{}; {}]", self.get_type_name(* element_type), length),
			ffi::Type::GpuBufferRef { element_type } => format!("caiman_rt::GpuBufferRef<'callee, {}>", self.get_type_name(* element_type)),
			ffi::Type::GpuBufferSlice { element_type } => format!("caiman_rt::GpuBufferSlice<'callee, {}>", self.get_type_name(* element_type)),
			ffi::Type::GpuBufferAllocator => format!("caiman_rt::GpuBufferAllocator<'callee>"),
			/*ffi::Type::SchedulingJoin { input_types, output_types, extra } =>
			{
				let mut output_string = String::new();
				// Temporary hack
				write!(output_string, "&mut (FnMut(");
				for (input_index, input_type) in input_types.iter().enumerate()
				{
					write!(output_string, "{}", self.get_type_name(* input_type));

					if input_index + 1 < input_types.len()
					{
						write!(output_string, ", ");
					}
				}
				write!(output_string, ") -> (");
				for (output_index, output_type) in output_types.iter().enumerate()
				{
					write!(output_string, "{}", self.get_type_name(* output_type));

					if output_index + 1 < output_types.len()
					{
						write!(output_string, ", ");
					}
				}
				write!(output_string, "))");
				output_string
			}*/
			_ => format!("type_{}", type_id.0)
		}
	}

	pub fn create_ffi_type(&mut self, typ : ffi::Type) -> ffi::TypeId
	{
		let type_id = ffi::TypeId(self.native_interface.types.create(typ));
		self.generate_type_definition(type_id);
		type_id
	}

	pub fn lookup_closure_id(&mut self, funclet_id : ir::FuncletId, capture_types : &[ffi::TypeId], argument_types : &[ffi::TypeId]) -> ClosureId
	{
		if let Some(closure) = self.active_closures.get(& (funclet_id, capture_types.len()))
		{
			for (capture_index, capture_type) in capture_types.iter().enumerate()
			{
				assert_eq!(closure.capture_types[capture_index], * capture_type);
			}
			closure.closure_id
		}
		else
		{
			let closure_id = ClosureId(self.closure_id_generator.generate());
			let dispatcher_id = self.lookup_dispatcher_id(argument_types);
			let old = self.active_closures.insert((funclet_id, capture_types.len()), Closure{capture_types : capture_types.to_vec().into_boxed_slice(), argument_types : argument_types.to_vec().into_boxed_slice(), closure_id, dispatcher_id});
			assert!(old.is_none());
			closure_id
		}
	}

	pub fn lookup_dispatcher_id(&mut self, argument_types : &[ffi::TypeId]) -> DispatcherId
	{
		if let Some(dispatcher) = self.active_dispatchers.get(argument_types)
		{
			dispatcher.dispatcher_id
		}
		else
		{
			let dispatcher_id = DispatcherId(self.dispatcher_id_generator.generate());
			let old = self.active_dispatchers.insert(argument_types.to_vec().into_boxed_slice(), Dispatcher{dispatcher_id});
			assert!(old.is_none());
			dispatcher_id
		}
	}

	pub fn build_constant_integer(&mut self, value : i64, type_id : ffi::TypeId) -> VarId
	{
		//let variable_id = self.variable_tracker.generate();
		//self.generate_type_definition(type_id);
		let variable_id = self.variable_tracker.create_local_data(type_id);
		write!(self.code_writer, "let {} : {} = {};\n", self.variable_tracker.get_var_name(variable_id), self.get_type_name(type_id), value);
		variable_id
	}

	pub fn build_constant_unsigned_integer(&mut self, value : u64, type_id : ffi::TypeId) -> VarId
	{
		//let variable_id = self.variable_tracker.generate();
		//self.generate_type_definition(type_id);
		let variable_id = self.variable_tracker.create_local_data(type_id);
		write!(self.code_writer, "let {} : {} = {};\n", self.variable_tracker.get_var_name(variable_id), self.get_type_name(type_id), value);
		variable_id
	}

	pub fn build_select_hack(&mut self, condition_var_id : VarId, true_case_var_id : VarId, false_case_var_id : VarId) -> VarId
	{
		//let variable_id = self.variable_tracker.generate();
		let true_type_id = self.variable_tracker.variable_types[& true_case_var_id];
		let false_type_id = self.variable_tracker.variable_types[& false_case_var_id];
		assert_eq!(true_type_id, false_type_id);
		let type_id = true_type_id;
		//self.generate_type_definition(type_id);
		let variable_kind = self.variable_tracker.get_kind(true_case_var_id);
		assert_eq!(variable_kind, self.variable_tracker.get_kind(false_case_var_id));
		let variable_id = self.variable_tracker.create(variable_kind, type_id);
		// Too lazy to implement booleans for now
		write!(self.code_writer, "let {} : {} = if {} != 0 {{ {} }} else {{ {} }};\n", self.variable_tracker.get_var_name(variable_id), self.get_type_name(type_id), self.variable_tracker.get_var_name(condition_var_id), self.variable_tracker.get_var_name(true_case_var_id), self.variable_tracker.get_var_name(false_case_var_id));
		variable_id
	}

	pub fn begin_if_else(&mut self, condition_var_id : VarId, output_type_ids : &[ffi::TypeId]) -> Box<[VarId]>
	{
		// Temporary fix
		self.reset_pipeline();

		write!(self.code_writer, "let ( ");
		let mut var_ids = Vec::<VarId>::new();
		for (i, type_id) in output_type_ids.iter().enumerate()
		{
			//self.generate_type_definition(* type_id);
			let var_id = self.variable_tracker.create_local_data(* type_id);

			write!(self.code_writer, "{} : {}", self.variable_tracker.get_var_name(var_id), self.get_type_name(* type_id));
			if i < output_type_ids.len() - 1
			{
				write!(self.code_writer, ", ");
			}

			var_ids.push(var_id);
		}
		write!(self.code_writer, " ) = if {} !=0 {{ ", self.variable_tracker.get_var_name(condition_var_id));

		var_ids.into_boxed_slice()
	}

	pub fn end_if_begin_else(&mut self, output_var_ids : &[VarId])
	{
		// Temporary fix
		self.reset_pipeline();
		
		write!(self.code_writer, " ( ");
		for (i, var_id) in output_var_ids.iter().enumerate()
		{
			write!(self.code_writer, "{}", self.variable_tracker.get_var_name(* var_id));
			if i < output_var_ids.len() - 1
			{
				write!(self.code_writer, ", ");
			}
		}
		write!(self.code_writer, " ) }} else {{ ");
	}

	pub fn end_else(&mut self, output_var_ids : &[VarId])
	{
		write!(self.code_writer, " ( ");
		for (i, var_id) in output_var_ids.iter().enumerate()
		{
			write!(self.code_writer, "{}", self.variable_tracker.get_var_name(* var_id));
			if i < output_var_ids.len() - 1
			{
				write!(self.code_writer, ", ");
			}
		}
		write!(self.code_writer, " ) }};\n");

		// Temporary fix
		self.reset_pipeline();
	}

	/*pub fn build_join(&mut self, callee_funclet_id : ir::FuncletId, captured_var_ids : &[VarId], input_type_ids : &[ffi::TypeId], output_type_ids : &[ffi::TypeId]) -> VarId
	{
		// Temporary fix
		self.reset_pipeline();

		let join_var_id = self.variable_tracker.generate();

		write!(self.code_writer, "let mut {} = move |instance : Instance<'state, 'cpu_functions, Callbacks>", self.variable_tracker.get_var_name(join_var_id));
		for (i, type_id) in input_type_ids.iter().enumerate()
		{
			self.generate_type_definition(* type_id);
			write!(self.code_writer, ", arg_{} : {}", i + captured_var_ids.len(), self.get_type_name(* type_id));
		}
		write!(self.code_writer, "| {{ funclet{}_func(instance", callee_funclet_id);

		assert!(captured_var_ids.len() <= input_type_ids.len());

		for (i, var_id) in captured_var_ids.iter().enumerate()
		{
			write!(self.code_writer, ", {}", self.variable_tracker.get_var_name(var_id));
		}

		for i in captured_var_ids.len() .. input_type_ids.len()
		{
			write!(self.code_writer, ", arg_{}", i);
		}

		write!(self.code_writer, " ) }};\n");

		join_var_id
	}

	fn begin_join(&mut self, input_type_ids : &[ffi::TypeId], output_type_ids : &[ffi::TypeId]) -> (usize, Box<[usize]>)
	{
		let join_var_id = self.variable_tracker.generate();
		write!(self.code_writer, "let mut var_{} = move |instance", join_var_id);
		let mut var_ids = Vec::<usize>::new();
		for (i, type_id) in input_type_ids.iter().enumerate()
		{
			self.generate_type_definition(* type_id);
			let var_id = self.variable_tracker.create_local_data(* type_id);

			write!(self.code_writer, ", var_{} : {}", var_id, self.get_type_name(* type_id));

			var_ids.push(var_id);
		}
		write!(self.code_writer, "| -> (");

		for (i, type_id) in output_type_ids.iter().enumerate()
		{
			write!(self.code_writer, "{}", self.get_type_name(* type_id));
			if i < output_type_ids.len() - 1
			{
				write!(self.code_writer, ", ");
			}
		}
		write!(self.code_writer, ") {{");

		(join_var_id, var_ids.into_boxed_slice())
	}

	fn end_join(&mut self, output_var_ids : &[usize])
	{
		// Temporary fix
		self.reset_pipeline();

		write!(self.code_writer, " ( ");
		for (i, var_id) in output_var_ids.iter().enumerate()
		{
			write!(self.code_writer, "var_{}", var_id);
			if i < output_var_ids.len() - 1
			{
				write!(self.code_writer, ", ");
			}
		}
		write!(self.code_writer, " ) }};\n");
	}

	fn call_join(&mut self, join_var_id : usize, input_var_ids : &[usize])
	{
		// Temporary fix
		self.reset_pipeline();

		write!(self.code_writer, "return var_{}(instance", join_var_id);
		for (i, var_id) in input_var_ids.iter().enumerate()
		{
			write!(self.code_writer, ", var_{}", var_id);
		}
		write!(self.code_writer, ");");
	}*/

	pub fn build_external_cpu_function_call(&mut self, external_function_id : ir::ExternalCpuFunctionId, argument_vars : &[VarId]) -> Box<[VarId]>
	{
		let external_cpu_function = & self.native_interface.external_cpu_functions[& external_function_id];
		let call_result_var = self.variable_tracker.generate();
		let mut argument_string = String::new();
		for (index, argument) in argument_vars.iter().enumerate()
		{
			argument_string += format!("{}", self.variable_tracker.get_var_name(* argument)).as_str();
			if index + 1 < argument_vars.len()
			{
				argument_string += ", ";
			}
		}
		self.code_writer.write(format!("let {} = instance.cpu_functions.{}(instance.state, {});\n", self.variable_tracker.get_var_name(call_result_var), external_cpu_function.name, argument_string));
		let mut output_variables = Vec::<VarId>::new();
		for (i, output_type) in external_cpu_function.output_types.iter().enumerate()
		{
			//let var = self.variable_tracker.generate();
			let var = self.variable_tracker.create_local_data(* output_type);
			output_variables.push(var);
			self.code_writer.write(format!("let {} = {}.{};\n", self.variable_tracker.get_var_name(var), self.variable_tracker.get_var_name(call_result_var), i));
		};
		output_variables.into_boxed_slice()
	}

	/*fn build_create_uninit_cpu_local_slot(&mut self, type_id : ffi::TypeId) -> usize
	{

	}*/

	pub fn build_create_buffer(&mut self, type_id : ffi::TypeId) -> VarId
	{
		let variable_id = self.variable_tracker.create_buffer(type_id);
		let type_binding_info = self.get_type_binding_info(type_id); 
		let type_name = self.get_type_name(type_id);
		self.code_writer.write(format!("let mut {} = instance.state.get_device_mut().create_buffer(& wgpu::BufferDescriptor {{ label : None, size : {}, usage : wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::MAP_WRITE, mapped_at_creation : false}});\n", self.variable_tracker.get_var_name(variable_id), type_binding_info.size));
		variable_id
	}

	pub fn build_buffer_ref(&mut self, buffer_var_id : VarId, offset_var_id : VarId, type_id : ffi::TypeId) -> VarId
	{
		let variable_id = self.variable_tracker.create_local_data(type_id);
		let type_binding_info = self.get_type_binding_info(type_id);
		let type_name = self.get_type_name(type_id);
		write!(self.code_writer, "let {} = caiman_rt::GpuBufferRef::<'_, {}>::new(& {}, {});\n", self.variable_tracker.get_var_name(variable_id), type_name, self.variable_tracker.get_var_name(buffer_var_id), self.variable_tracker.get_var_name(offset_var_id));
		variable_id
	}

	pub fn build_buffer_suballocate_ref(&mut self, buffer_allocator_var_id : VarId, type_id : ffi::TypeId) -> VarId
	{
		let variable_id = self.variable_tracker.create_local_data(type_id);
		let type_binding_info = self.get_type_binding_info(type_id);
		let type_name = self.get_type_name(type_id);
		write!(self.code_writer, "let {} = {}.suballocate_ref::<'callee, {}>().unwrap();\n", self.variable_tracker.get_var_name(variable_id), self.variable_tracker.get_var_name(buffer_allocator_var_id), type_name);
		variable_id
	}

	pub fn build_buffer_suballocate_slice(&mut self, buffer_allocator_var_id : VarId, type_id : ffi::TypeId, count_var_id : VarId) -> VarId
	{
		let variable_id = self.variable_tracker.create_local_data(type_id);
		let type_binding_info = self.get_type_binding_info(type_id);
		let type_name = self.get_type_name(type_id);
		write!(self.code_writer, "let {} = {}.suballocate_slice::<'callee, {}>({}).unwrap();\n", self.variable_tracker.get_var_name(variable_id), self.variable_tracker.get_var_name(buffer_allocator_var_id), type_name, self.variable_tracker.get_var_name(count_var_id));
		variable_id
	}

	pub fn build_test_suballocate_many(&mut self, buffer_allocator_var_id : VarId, type_id_and_count_var_id_pairs : &[(ffi::TypeId, Option<VarId>)] ) -> VarId
	{
		let mut layouts_string = String::from("");
		let mut element_counts_string = String::from("");

		for (type_id, count_var_id_opt) in type_id_and_count_var_id_pairs.iter()
		{
			let type_binding_info = self.get_type_binding_info(* type_id);
			write!(layouts_string, "caiman_rt::TypeLayout{{byte_size : {}, alignment : {}}}, ", type_binding_info.size, type_binding_info.alignment);
			if let Some(count_var_id) = count_var_id_opt
			{
				write!(element_counts_string, "Some({}), ", self.variable_tracker.get_var_name(* count_var_id));
			}
			else
			{
				write!(element_counts_string, "None, ");
			}
		}

		let success_var_id = self.variable_tracker.generate();
		write!(self.code_writer, "let {} = {}.test_suballocate_many(&[{}], &[{}]);\n", self.variable_tracker.get_var_name(success_var_id), self.variable_tracker.get_var_name(buffer_allocator_var_id), layouts_string, element_counts_string);
		
		success_var_id
	}

	pub fn build_push_serialized_join(&mut self, funclet_id : ir::FuncletId, capture_var_ids : & [VarId], capture_types : &[ffi::TypeId], argument_types : & [ffi::TypeId], output_types : & [ffi::TypeId])
	{
		let _closure_id = self.lookup_closure_id(funclet_id, capture_types, argument_types);
		let _argument_dispatcher_id = self.lookup_dispatcher_id(argument_types);
		//let _output_dispatcher_id = self.lookup_dispatcher_id(output_types);
		println!("Pushed serialzed join for funclet {}: {:?}", funclet_id, self.active_closures.get(& (funclet_id, capture_types.len())));


		let tuple_definition_string = self.get_tuple_definition_string(capture_types);
		//write!(self.code_writer, "{{ let join_data : {} = (", tuple_definition_string);
		write!(self.code_writer, "{{ let join_data : Funclet{}Capturing{}CapturedTuple<'callee> = (", funclet_id, capture_types.len());
		for var_id in capture_var_ids.iter()
		{
			write!(self.code_writer, "{}, ", self.variable_tracker.get_var_name(* var_id));
		}
		write!(self.code_writer, "); let closure_header = ClosureHeader::Funclet{}Capturing{}; unsafe {{ join_stack.push_unsafe_unaligned(join_data).expect(\"Ran out of memory while serializing join\"); join_stack.push_unsafe_unaligned(closure_header).expect(\"Ran out of memory while serializing join\"); }}", funclet_id, capture_types.len());

		write!(self.code_writer, "}}");
	}

	/*fn encode_copy_cpu_from_gpu(&mut self, destination_var : usize, source_var : usize)
	{
		
	}*/

	/*pub fn encode_copy_local_data_from_local_data(&mut self, destination_var : usize, source_var : usize)
	{
		write!(self.code_writer, "let var_{} = var_{};\n", destination_var, source_var);
	}*/

	pub fn encode_clone_local_data_from_buffer(&mut self, source_var : VarId) -> VarId
	{
		let type_id = self.variable_tracker.get_type_id(source_var);
		//assert_eq!(type_id, self.variable_tracker.get_type_id(destination_var));

		let range_var_id = self.variable_tracker.generate();
		let output_temp_var_id = self.variable_tracker.generate();
		let slice_var_id = self.variable_tracker.generate();
		let future_var_id = self.variable_tracker.generate();
		let type_binding_info = self.get_type_binding_info(type_id); 
		let type_name = self.get_type_name(type_id);

		let output_var_id = self.variable_tracker.create_local_data(type_id);
		
		self.code_writer.write(format!("let {} = {}.slice();\n", self.variable_tracker.get_var_name(slice_var_id), self.variable_tracker.get_var_name(source_var)));
		self.code_writer.write(format!("let {} = {}.map_async(wgpu::MapMode::Read);\n", self.variable_tracker.get_var_name(future_var_id), self.variable_tracker.get_var_name(slice_var_id)));
		self.code_writer.write(format!("instance.state.get_device_mut().poll(wgpu::Maintain::Wait);\n"));
		self.code_writer.write(format!("futures::executor::block_on({});;\n", self.variable_tracker.get_var_name(future_var_id)));
		self.code_writer.write(format!("let {} = {}.get_mapped_range();\n", self.variable_tracker.get_var_name(range_var_id), self.variable_tracker.get_var_name(slice_var_id)));
		self.code_writer.write(format!("let {} = * unsafe {{ std::mem::transmute::<* const u8, & {}>({}.as_ptr()) }};\n", self.variable_tracker.get_var_name(output_var_id), type_name, self.variable_tracker.get_var_name(range_var_id)));
		return output_var_id;
	}

	pub fn encode_copy_buffer_from_local_data(&mut self, destination_var : VarId, source_var : VarId)
	{
		let buffer_view_var_name = self.variable_tracker.get_var_name(destination_var);
		self.code_writer.write(format!("instance.state.get_queue_mut().write_buffer({}.buffer, {}.base_address, & {}.to_ne_bytes() );\n", buffer_view_var_name, buffer_view_var_name, self.variable_tracker.get_var_name(source_var)));
	}

	pub fn encode_copy_buffer_from_buffer(&mut self, destination_var : VarId, source_var : VarId)
	{
		let type_id = self.variable_tracker.get_type_id(source_var);
		assert_eq!(type_id, self.variable_tracker.get_type_id(destination_var));
		let type_binding_info = self.get_type_binding_info(type_id); 
		self.begin_command_encoding();
		write!(self.code_writer, "command_encoder.copy_buffer_to_buffer(& {}, 0, & {}, 0, {});\n", self.variable_tracker.get_var_name(destination_var), self.variable_tracker.get_var_name(source_var), type_binding_info.size);
		let command_buffer_id = self.end_command_encoding();
		self.enqueue_command_buffer(command_buffer_id);
	}

	fn build_create_buffer_with_data(&mut self, data_var : VarId, type_id : ffi::TypeId) -> VarId
	{
		let variable_id = self.variable_tracker.generate();
		let type_binding_info = self.get_type_binding_info(type_id); 
		let type_name = self.get_type_name(type_id);
		let buffer_view_var_name = self.variable_tracker.get_var_name(variable_id);
		self.code_writer.write(format!("let mut {} = instance.state.get_device_mut().create_buffer(& wgpu::BufferDescriptor {{ label : None, size : {}, usage : wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::MAP_WRITE, mapped_at_creation : false}});\n", self.variable_tracker.get_var_name(variable_id), type_binding_info.size));
		self.code_writer.write(format!("instance.state.get_queue_mut().write_buffer(& {}.buffer, {}.base_address, & {}.to_ne_bytes() );\n", buffer_view_var_name, buffer_view_var_name, self.variable_tracker.get_var_name(data_var)));
		variable_id
	}

	fn build_create_buffer_with_buffer_data(&mut self, data_var : VarId, type_id : ffi::TypeId) -> VarId
	{
		let variable_id = self.variable_tracker.generate();
		let type_binding_info = self.get_type_binding_info(type_id); 
		let type_name = self.get_type_name(type_id);
		self.code_writer.write(format!("let mut {} = instance.state.get_device_mut().create_buffer(& wgpu::BufferDescriptor {{ label : None, size : {}, usage : wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::MAP_WRITE, mapped_at_creation : false}});\n", self.variable_tracker.get_var_name(variable_id), type_binding_info.size));
		write!(self.code_writer, "{{\n");
		self.code_writer.write("let mut command_encoder = instance.state.get_device_mut().create_command_encoder(& wgpu::CommandEncoderDescriptor {label : None});\n".to_string());
		write!(self.code_writer, "command_encoder.copy_buffer_to_buffer(& {}, 0, & {}, 0, {});\n", self.variable_tracker.get_var_name(data_var), self.variable_tracker.get_var_name(variable_id), type_binding_info.size);
		self.code_writer.write("let command_buffer = command_encoder.finish();\n".to_string());
		self.code_writer.write("instance.state.get_queue_mut().submit([command_buffer]);\n".to_string());
		self.code_writer.write(format!("instance.state.get_device_mut().poll(wgpu::Maintain::Wait);\n"));
		self.code_writer.write("futures::executor::block_on(instance.state.get_queue_mut().on_submitted_work_done());\n".to_string());
		write!(self.code_writer, "}}\n");
		//self.code_writer.write(format!("queue.write_buffer(& var_{}, 0, & var_{}.to_ne_bytes() );\n", variable_id, data_var));
		variable_id
	}

	/*pub fn build_compute_dispatch(&mut self, external_function_id : ir::ExternalGpuFunctionId, dimension_vars : &[usize; 3], argument_vars : &[usize]) -> Box<[usize]>
	{
		let output_vars = self.generate_compute_dispatch_outputs(external_function_id);
		self.generate_compute_dispatch(external_function_id, dimension_vars, argument_vars, & output_vars);
		return output_vars;
	}*/

	pub fn build_compute_dispatch_with_outputs(&mut self, external_function_id : ir::ExternalGpuFunctionId, dimension_vars : &[VarId; 3], argument_vars : &[VarId], output_vars : &[VarId])
	{
		self.generate_compute_dispatch(external_function_id, dimension_vars, argument_vars, output_vars);
	}
}