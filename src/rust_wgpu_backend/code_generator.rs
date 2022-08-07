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

// Submissions represent groups of tasks that are executing in a logical sequence
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Default)]
pub struct SubmissionId(usize);

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Default)]
pub struct CommandBufferId(usize);

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Default)]
pub struct VarId(usize);

#[derive(Debug, Default)]
struct SubmissionQueue
{
	//most_recently_synchronized_submission_id : Option<SubmissionId>,
	next_submission_id : SubmissionId
}

enum ResourceUsage
{

}

// Tracks where the data is
#[derive(PartialEq, Eq, Debug)]
enum VariableState
{
	Dead,
	InEncoding,
	InQueue,
	OnGpu,
	Local,
}

#[derive(Default)]
struct VariableTracker
{
	id_generator : IdGenerator,
	variable_states : HashMap<usize, VariableState>,
	variable_types : HashMap<usize, ir::TypeId>
}

impl VariableTracker
{
	fn new() -> Self
	{
		Self { id_generator : IdGenerator::new(), variable_states : HashMap::<usize, VariableState>::new(), variable_types : HashMap::<usize, ir::TypeId>::new() }
	}

	fn generate(&mut self) -> usize
	{
		self.id_generator.generate()
	}

	fn create(&mut self, state : VariableState) -> usize
	{
		let id = self.generate();
		self.variable_states.insert(id, state);
		id
	}

	fn create_local(&mut self, type_id : ir::TypeId) -> usize
	{
		let id = self.create(VariableState::Local);
		self.variable_types.insert(id, type_id);
		id
	}

	fn create_in_encoding(&mut self, type_id : ir::TypeId) -> usize
	{
		let id = self.create(VariableState::InEncoding);
		self.variable_types.insert(id, type_id);
		id
	}

	fn transition_to_queue(&mut self, var_id : usize)
	{
		if let VariableState::InEncoding = self.variable_states[& var_id]
		{
			self.variable_states.insert(var_id, VariableState::InQueue);
		}
		else
		{
			panic!("Var {} is not in encoding", var_id);
		}
	}

	fn transition_to_on_gpu(&mut self, var_id : usize)
	{
		assert_eq!(self.variable_states[& var_id], VariableState::InQueue);
		self.variable_states.insert(var_id, VariableState::OnGpu);
	}

	fn transition_to_local(&mut self, var_id : usize)
	{
		assert_eq!(self.variable_states[& var_id], VariableState::OnGpu);
		self.variable_states.insert(var_id, VariableState::Local);
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
	result_type_id : ir::TypeId,
	next_funclet_ids : Option<Box<[ir::FuncletId]>>,
	capture_count : usize,
	output_count : usize,
	output_type_ids : Box<[ir::TypeId]>,
	next_funclet_input_types : Option<Box<[Box<[ir::TypeId]>]>>
}

struct TypeBindingInfo
{
	size : usize,
	alignment : usize,
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

pub struct CodeGenerator<'program>
{
	type_code_writer : CodeWriter,
	state_code_writer : CodeWriter,
	code_writer : CodeWriter, // the "everything else" for now
	types : Arena<ir::Type>,
	external_cpu_functions : & 'program [ir::ExternalCpuFunction],
	external_gpu_functions : & 'program [ir::ExternalGpuFunction],
	has_been_generated : HashSet<usize>,
	variable_tracker : VariableTracker,
	active_pipeline_name : Option<String>,
	active_funclet_result_type_id : Option<ir::TypeId>,
	active_funclet_state : Option<ActiveFuncletState>,
	use_recording : bool,
	active_submission_encoding_state : Option<SubmissionEncodingState>,
	active_external_gpu_function_id : Option<ir::ExternalGpuFunctionId>,
	active_shader_module_key : Option<ShaderModuleKey>,
	shader_modules : BTreeMap<ShaderModuleKey, shadergen::ShaderModule>,
	submission_queue : SubmissionQueue,
	next_command_buffer_id : CommandBufferId,
	gpu_function_invocations : Vec<GpuFunctionInvocation>
}

impl<'program> CodeGenerator<'program>
{
	pub fn new(types : Arena<ir::Type>, external_cpu_functions : & 'program [ir::ExternalCpuFunction], external_gpu_functions : & 'program [ir::ExternalGpuFunction]) -> Self
	{
		let variable_tracker = VariableTracker::new();
		let type_code_writer = CodeWriter::new();
		let state_code_writer = CodeWriter::new();
		let code_writer = CodeWriter::new();
		let has_been_generated = HashSet::new();
		Self {type_code_writer, state_code_writer, code_writer, types, has_been_generated, variable_tracker, external_cpu_functions, external_gpu_functions, active_pipeline_name : None, active_funclet_result_type_id : None, active_funclet_state : None, use_recording : true, active_submission_encoding_state : None, active_external_gpu_function_id : None, active_shader_module_key : None, shader_modules : BTreeMap::new(), submission_queue : Default::default(), next_command_buffer_id : CommandBufferId(0), gpu_function_invocations : Vec::new()}
	}

	pub fn finish(&mut self) -> String
	{
		self.write_states();
		self.type_code_writer.finish() + self.state_code_writer.finish().as_str() + self.code_writer.finish().as_str()
	}

	fn generate_compute_dispatch_outputs(&mut self, external_function_id : ir::ExternalCpuFunctionId) -> Box<[usize]>
	{
		let mut output_vars = Vec::<usize>::new();

		let external_gpu_function = & self.external_gpu_functions[external_function_id];
		for (output_index, output_type_id) in external_gpu_function.output_types.iter().enumerate()
		{
			let variable_id = self.variable_tracker.create_in_encoding(* output_type_id);
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
			let external_gpu_function = & self.external_gpu_functions[external_function_id];
	
			let mut shader_module = match & external_gpu_function.shader_module_content
			{
				ir::ShaderModuleContent::Wgsl(text) => shadergen::ShaderModule::new_with_wgsl(text.as_str())
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

	fn set_active_bindings(&mut self, argument_vars : &[usize], output_vars : &[usize]) -> Box<[usize]>
	{
		let external_function_id = self.active_external_gpu_function_id.unwrap();
		let external_gpu_function = & self.external_gpu_functions[external_function_id];

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

		let mut input_staging_variables = Vec::<usize>::new();
		assert_eq!(argument_vars.len(), external_gpu_function.input_types.len());
		for input_index in 0 .. external_gpu_function.input_types.len()
		{
			let type_id = external_gpu_function.input_types[input_index];
			//let variable_id = self.build_create_buffer_with_data(argument_vars[input_index], type_id);
			let input_variable_id = argument_vars[input_index];

			let binding = input_binding_map[& input_index];
			if let (_, Some(_output)) = bindings[& binding]
			{
				let variable_id = self.build_create_buffer_with_buffer_data(input_variable_id, type_id);
				input_staging_variables.push(variable_id);
			}
			else
			{
				input_staging_variables.push(input_variable_id);
			}
		}

		let mut output_staging_variables = Vec::<usize>::new();
		for output_index in 0 .. external_gpu_function.output_types.len()
		{
			let binding = output_binding_map[& output_index];
			if let (Some(input), _) = bindings[& binding]
			{
				let variable_id = input_staging_variables[input];
				output_staging_variables.push(variable_id);
			}
			else
			{
				let type_id = external_gpu_function.output_types[output_index];
				let variable_id = self.build_create_buffer(type_id);
				output_staging_variables.push(variable_id);
			}
		};

		let invocation_id = self.gpu_function_invocations.len();
		self.gpu_function_invocations.push(GpuFunctionInvocation{external_gpu_function_id : external_function_id, bindings, shader_module_key : self.active_shader_module_key.unwrap()});
		let gpu_function_invocation = & self.gpu_function_invocations[invocation_id];
		
		self.code_writer.write("let entries = [".to_string());
		for (binding, (input_opt, output_opt)) in gpu_function_invocation.bindings.iter()
		{
			let mut variable_id : Option<usize> = None;
			
			if let Some(input) = input_opt
			{
				variable_id = Some(input_staging_variables[*input]);
			}

			if let Some(output) = output_opt
			{
				variable_id = Some(output_staging_variables[*output]);
			}

			assert_eq!(variable_id.is_some(), true, "Binding must be input or output");
			self.code_writer.write(format!("wgpu::BindGroupEntry {{binding : {}, resource : wgpu::BindingResource::Buffer(wgpu::BufferBinding{{buffer : & var_{}, offset : 0, size : None}}) }}, ", binding, variable_id.unwrap()));
		}
		self.code_writer.write("];\n".to_string());
		write!(self.code_writer, "let bind_group = instance.state.get_device_mut().create_bind_group(& wgpu::BindGroupDescriptor {{label : None, layout : & instance.static_bind_group_layout_{}, entries : & entries}});\n", invocation_id);
		write!(self.code_writer, "let pipeline = & instance.static_pipeline_{};\n", invocation_id);

		output_staging_variables.into_boxed_slice()
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

	pub fn require_local(&self, variable_ids : &[usize])
	{
		for variable_id in variable_ids.iter()
		{
			match self.variable_tracker.variable_states[variable_id]
			{
				//VariableState::InEncoding => self.flush_submission(),
				_ => ()
			}

			match self.variable_tracker.variable_states[variable_id]
			{
				VariableState::Local => (),
				VariableState::OnGpu => panic!("Not already local"),
				_ => panic!("Unimplemented")
			}
		}
	}

	pub fn require_on_gpu(&self, variable_ids : &[usize])
	{
		for variable_id in variable_ids.iter()
		{
			match self.variable_tracker.variable_states[variable_id]
			{
				VariableState::InEncoding => (),
				VariableState::Local => panic!("Not already on gpu"),
				VariableState::OnGpu => (),
				_ => panic!("Unimplemented")
			}
		}
	}

	pub fn make_local_copy(&mut self, variable_id : usize) -> Option<usize>
	{
		match self.variable_tracker.variable_states[& variable_id]
		{
			//VariableState::InEncoding => self.flush_submission(),
			_ => ()
		}

		match self.variable_tracker.variable_states[& variable_id]
		{
			VariableState::Local => (),
			VariableState::OnGpu =>
			{
				let type_id = self.variable_tracker.variable_types[& variable_id];
				let range_var_id = self.variable_tracker.generate();
				let output_temp_var_id = self.variable_tracker.generate();
				let slice_var_id = self.variable_tracker.generate();
				let future_var_id = self.variable_tracker.generate();
				let type_binding_info = self.get_type_binding_info(type_id); 
				let type_name = self.get_type_name(type_id);

				let output_var_id = self.variable_tracker.create_local(type_id);
				
				self.code_writer.write(format!("let var_{} = var_{}.slice(0..);\n", slice_var_id, variable_id));
				self.code_writer.write(format!("let var_{} = var_{}.map_async(wgpu::MapMode::Read);\n", future_var_id, slice_var_id));
				self.code_writer.write(format!("instance.state.get_device_mut().poll(wgpu::Maintain::Wait);\n"));
				self.code_writer.write(format!("futures::executor::block_on(var_{});;\n", future_var_id));
				self.code_writer.write(format!("let var_{} = var_{}.get_mapped_range();\n", range_var_id, slice_var_id));
				self.code_writer.write(format!("let var_{} = * unsafe {{ std::mem::transmute::<* const u8, & {}>(var_{}.as_ptr()) }};\n", output_var_id, type_name, range_var_id));
				return Some(output_var_id);
			}
			_ => panic!("Unimplemented")
		}
		
		//self.variable_tracker.variable_states.insert(* variable_id, VariableState::Local);
		None
	}

	pub fn make_on_gpu_copy(&mut self, variable_id : usize) -> Option<usize>
	{
		match self.variable_tracker.variable_states[& variable_id]
		{
			VariableState::InEncoding => (),
			VariableState::Local =>
			{
				let type_id = self.variable_tracker.variable_types[& variable_id];
				let type_binding_info = self.get_type_binding_info(type_id); 
				let type_name = self.get_type_name(type_id);
				let output_var_id = self.variable_tracker.create_in_encoding(type_id);
				self.code_writer.write(format!("let mut var_{} = instance.state.get_device_mut().create_buffer(& wgpu::BufferDescriptor {{ label : None, size : {}, usage : wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::MAP_WRITE, mapped_at_creation : false}});\n", output_var_id, type_binding_info.size));
				self.code_writer.write(format!("instance.state.get_queue_mut().write_buffer(& var_{}, 0, & var_{}.to_ne_bytes() );\n", output_var_id, variable_id));
				//self.code_writer.write(format!("let var_{} = var_{};\n", output_var_id, temp_id));
				//self.variable_tracker.variable_states.insert(* variable_id, VariableState::OnGpu);
				return Some(output_var_id);
			}
			VariableState::OnGpu => (),
			_ => panic!("Unimplemented")
		}

		None
	}

	fn require_exclusive(&mut self, variable_ids : &[usize])
	{

	}

	fn encode_compute_dispatch(&mut self, external_function_id : ir::ExternalGpuFunctionId, dimension_vars : &[usize; 3], argument_vars : &[usize], output_vars : &[usize])
	{

	}

	fn generate_compute_dispatch(&mut self, external_function_id : ir::ExternalGpuFunctionId, dimension_vars : &[usize; 3], argument_vars : &[usize], output_vars : &[usize])
	{
		self.require_local(dimension_vars);
		self.require_on_gpu(argument_vars);

		let external_gpu_function = & self.external_gpu_functions[external_function_id];
		assert_eq!(external_gpu_function.input_types.len(), argument_vars.len());

		self.set_active_external_gpu_function(external_function_id);
		let output_staging_variables = self.set_active_bindings(argument_vars, output_vars);
		
		self.begin_command_encoding();

		//let mut output_variables = Vec::<usize>::new();
		self.code_writer.write(format!("let ("));
		//self.code_writer.write(format!("let (old_command_buffer_{}, ", command_buffer_id));
		for output_index in 0 .. external_gpu_function.output_types.len()
		{
			//let var_id = self.variable_tracker.generate();
			//output_variables.push(var_id);
			let var_id = output_vars[output_index];
			self.code_writer.write(format!("var_{}, ", var_id));
		}
		self.code_writer.write(format!(") = "));

		self.code_writer.write("{\n".to_string());
		
		self.code_writer.write_str("{\n");
		self.code_writer.write("let mut compute_pass = command_encoder.begin_compute_pass(& wgpu::ComputePassDescriptor {label : None});\n".to_string());
		self.code_writer.write("compute_pass.set_pipeline(& pipeline);\n".to_string());
		self.code_writer.write("compute_pass.set_bind_group(0, & bind_group, & []);\n".to_string());
		self.code_writer.write(format!("compute_pass.dispatch(var_{}.try_into().unwrap(), var_{}.try_into().unwrap(), var_{}.try_into().unwrap());\n", dimension_vars[0], dimension_vars[1], dimension_vars[2]));
		self.code_writer.write_str("}\n");

		//self.code_writer.write("let command_buffer = command_encoder.finish();\n".to_string());
		//self.code_writer.write("queue.submit([command_buffer]);\n".to_string());
		//self.code_writer.write(format!("device.poll(wgpu::Maintain::Wait);\n"));
		//self.code_writer.write("futures::executor::block_on(queue.on_submitted_work_done());\n".to_string());

		let mut output_temp_variables = Vec::<usize>::new();
		for output_index in 0 .. external_gpu_function.output_types.len()
		{
			let staging_var_id = output_staging_variables[output_index];
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
			self.code_writer.write(format!("var_{}, ", output_temp_var_id));
		}
		self.code_writer.write(format!(")"));

		self.code_writer.write("};\n".to_string());

		let command_buffer_id = self.end_command_encoding();
		self.enqueue_command_buffer(command_buffer_id);

		for var_id in output_vars.iter()
		{
			// These are wrong
			self.variable_tracker.transition_to_queue(* var_id);
			self.variable_tracker.transition_to_on_gpu(* var_id);
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

		self.code_writer.write(format!("let future_var_{} = instance.state.get_queue_mut().on_submitted_work_done();\n", submission_id.0));

		submission_id
	}

	pub fn sync_submission(&mut self, submission_id : SubmissionId)
	//pub fn sync_submissions(&mut self)
	{
		//self.code_writer.write("futures::executor::block_on(queue.on_submitted_work_done());\n".to_string());
		self.code_writer.write(format!("instance.state.get_device_mut().poll(wgpu::Maintain::Wait);\n"));
		self.code_writer.write(format!("futures::executor::block_on(future_var_{});\n", submission_id.0));
		//self.code_writer.write("futures::executor::block_on(queue.on_submitted_work_done());\n".to_string());
	}

	pub fn insert_comment(&mut self, comment_string : &str)
	{
		self.code_writer.write(format!("// {}\n", comment_string));
	}

	fn write_states(&mut self)
	{
		//self.state_code_writer
		let code_string = "
		use caiman_rt::wgpu;
		
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

		self.active_pipeline_name = Some(String::from(pipeline_name));
		self.code_writer.begin_module(pipeline_name);
		write!(self.code_writer, "use super::*;\n");

		self.code_writer.begin_module("outputs");
		{
			for external_cpu_function in self.external_cpu_functions.iter()
			{
				let mut tuple_fields = Vec::<ir::TypeId>::new();
				for (output_index, output_type) in external_cpu_function.output_types.iter().enumerate()
				{
					tuple_fields.push(*output_type);
				}
				let type_id = self.types.create(ir::Type::Tuple{fields : tuple_fields.into_boxed_slice()});
				self.generate_type_definition(type_id);
				write!(self.code_writer, "pub type {} = super::super::{};\n", external_cpu_function.name, self.get_type_name(type_id));
			}
		}
		self.code_writer.end_module();

		self.code_writer.write(format!("pub trait CpuFunctions\n{{\n"));
		for external_cpu_function in self.external_cpu_functions.iter()
		{
			self.code_writer.write(format!("\tfn {}(&self, state : &mut caiman_rt::State", external_cpu_function.name));
			for (input_index, input_type) in external_cpu_function.input_types.iter().enumerate()
			{
				self.generate_type_definition(* input_type);
				self.code_writer.write(format!(", _ : {}", self.get_type_name(*input_type)));
			}
			self.code_writer.write(format!(") -> outputs::{};\n", external_cpu_function.name));
		}
		self.code_writer.write(format!("}}\n"));
	}

	pub fn begin_oneshot_entry_funclet(&mut self, input_types : &[ir::TypeId], output_types : &[ir::TypeId]) -> Box<[usize]>
	{
		self.code_writer.begin_module("pipeline_outputs");
		{
			let mut tuple_fields = Vec::<ir::TypeId>::new();
			for output_index in 0 .. output_types.len()
			{
				let output_type = output_types[output_index];
				tuple_fields.push(output_type);
				self.generate_type_definition(output_type);
			}
			let type_id = self.types.create(ir::Type::Tuple{fields : tuple_fields.into_boxed_slice()});
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
			let variable_id = self.variable_tracker.create_local(* input_type);
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
	}

	pub fn begin_funclet(&mut self, funclet_id : ir::FuncletId, input_types : &[ir::TypeId], output_types : &[ir::TypeId]) -> Box<[usize]>
	{
		// Temporarily need to do this until pipelines are constructed correctly
		self.reset_pipeline();

		//self.code_writer.begin_module("funclet_outputs");
		let funclet_result_type_id = {
			let mut tuple_fields = Vec::<ir::TypeId>::new();
			for output_index in 0 .. output_types.len()
			{
				let output_type = output_types[output_index];
				self.generate_type_definition(output_type);
				tuple_fields.push(output_type);
			}
			let type_id = self.types.create(ir::Type::Tuple{fields : tuple_fields.into_boxed_slice()});
			self.generate_type_definition(type_id);
			//write!(self.code_writer, "pub type {} = super::super::{};\n", self.active_pipeline_name.as_ref().unwrap().as_str(), self.get_type_name(type_id));
			type_id
		};
		//self.code_writer.end_module();

		self.active_funclet_result_type_id = Some(funclet_result_type_id);

		//self.code_writer.write(format!("pub struct Funclet{}Result<'state, 'cpu_functions, Callbacks : CpuFunctions> {{instance : Instance<'state, 'cpu_functions, F>, intermediates : super::{}}}", funclet_id, self.get_type_name(funclet_result_type_id)));
		//self.code_writer.write(format!("pub struct Funclet{}<'state, 'cpu_functions, F : CpuFunctions> {{instance : Instance<'state, 'cpu_functions, F>, intermediates : super::{}}}", funclet_id, self.get_type_name(funclet_result_type_id)));

		//self.code_writer.write(format!("}}"));

		//self.code_writer.write(format!("impl<'state,  'cpu_functions, F : CpuFunctions> Funclet{}<'state,  'cpu_functions, F>\n{{\n", funclet_id));

		let mut next_trait_index = 0usize;

		let mut argument_variable_ids = Vec::<usize>::new();
		write!(self.code_writer, "fn funclet{}_func<'state,  'cpu_functions, Callbacks : CpuFunctions>(instance : Instance<'state, 'cpu_functions, Callbacks>", funclet_id);

		/*for (input_index, input_type) in input_types.iter().enumerate()
		{
			self.code_writer.write(", ".to_string());
			match & self.types[input_type]
			{
				ir::Type::Slot{ .. } =>
				{

				}
				ir::Type::SchedulingJoin { input_types, output_types, extra } =>
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
			/*match & self.types[input_type]
			{
				ir::Type::Slot{ .. } =>
				{
					self.generate_type_definition(* input_type);

					//let variable_id = self.variable_tracker.generate();
					let variable_id = self.variable_tracker.create_local(* input_type);
					argument_variable_ids.push(variable_id);
					let type_name = self.get_type_name(*input_type);
					self.code_writer.write(format!("var_{} : {}", variable_id, type_name));
				}
				ir::Type::SchedulingJoin { input_types, output_types, extra } =>
				{

				}
				_ => panic!("Unknown type")
			}*/

			self.generate_type_definition(* input_type);

			//let variable_id = self.variable_tracker.generate();
			let variable_id = self.variable_tracker.create_local(* input_type);
			argument_variable_ids.push(variable_id);
			let type_name = self.get_type_name(*input_type);
			self.code_writer.write(format!("var_{} : {}", variable_id, type_name));
		}

		self.active_funclet_state = Some(ActiveFuncletState{funclet_id, result_type_id : funclet_result_type_id, next_funclet_ids : None, capture_count : 0, output_count : 0, output_type_ids : output_types.to_vec().into_boxed_slice(), next_funclet_input_types : None});

		write!(self.code_writer, " ) -> FuncletResult<'state, 'cpu_functions, Callbacks, {}>", self.get_type_name(funclet_result_type_id));
		self.code_writer.write("\n{\n\tuse std::convert::TryInto;\n".to_string());
		argument_variable_ids.into_boxed_slice()
	}

	pub fn emit_oneshot_pipeline_entry_point(&mut self, funclet_id : ir::FuncletId, input_types : &[ir::TypeId], output_types : &[ir::TypeId])
	{
		self.code_writer.begin_module("pipeline_outputs");
		{
			let mut tuple_fields = Vec::<ir::TypeId>::new();
			for output_index in 0 .. output_types.len()
			{
				let output_type = output_types[output_index];
				tuple_fields.push(output_type);
			}
			let type_id = self.types.create(ir::Type::Tuple{fields : tuple_fields.into_boxed_slice()});
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
			let variable_id = self.variable_tracker.create_local(* input_type);
			argument_variable_ids.push(variable_id);
			let type_name = self.get_type_name(*input_type);
			self.code_writer.write(format!("var_{} : {}", variable_id, type_name));

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
			write!(self.code_writer, ", var_{}", *var_id);
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
	}

	pub fn emit_pipeline_entry_point(&mut self, funclet_id : ir::FuncletId, input_types : &[ir::TypeId], output_types : &[ir::TypeId])
	{
		let pipeline_name = self.active_pipeline_name.as_ref().unwrap();

		let funclet_result_definition_string = "
		pub struct FuncletResult<'state, 'cpu_functions, Callbacks : CpuFunctions, Intermediates>
		{
			instance : Instance<'state, 'cpu_functions, Callbacks>,
			intermediates : Intermediates
		}
		";

		write!(self.code_writer, "{}", funclet_result_definition_string);

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

		/*write!(self.code_writer, "\t\tpub fn start(mut self");

		for (input_index, input_type) in input_types.iter().enumerate()
		{
			let type_name = self.get_type_name(*input_type);
			self.code_writer.write(format!(", input_{} : {}", input_index, type_name));
		}

		write!(self.code_writer, ") -> FuncletResult<'state, 'cpu_functions, F> \n{{\n\t Funclet{}::new(self", funclet_id, funclet_id);

		for (input_index, input_type) in input_types.iter().enumerate()
		{
			self.code_writer.write(format!(", input_{}", input_index));
		}

		write!(self.code_writer, ")\n}}\n");*/

		write!(self.code_writer, "{}", "
		}
		");
	}

	pub fn build_return(&mut self, output_var_ids : &[usize])
	{
		self.require_local(output_var_ids);
		//self.get_type_name(self.active_funclet_result_type_id.unwrap())
		//self.active_funclet_state.as_ref().unwrap().funclet_id
		write!(self.code_writer, "return FuncletResult {{instance, intermediates : (");
		for (return_index, var_id) in output_var_ids.iter().enumerate()
		{
			write!(self.code_writer, "var_{}, ", var_id);
		}
		write!(self.code_writer, ")}};");
	}

	pub fn build_yield(&mut self, next_funclet_ids : &[ir::FuncletId], next_funclet_input_types : Box<[Box<[ir::TypeId]>]>, capture_var_ids : &[usize], output_var_ids : &[usize])
	{
		self.active_funclet_state.as_mut().unwrap().next_funclet_ids = Some(next_funclet_ids.to_vec().into_boxed_slice());

		self.require_local(capture_var_ids);
		self.require_local(output_var_ids);
		self.code_writer.write(format!("return Funclet{} {{instance, intermediates : super::{} {{", self.active_funclet_state.as_ref().unwrap().funclet_id, self.get_type_name(self.active_funclet_result_type_id.unwrap())));
		for (return_index, var_id) in capture_var_ids.iter().enumerate()
		{
			self.code_writer.write(format!("field_{} : var_{}, ", return_index, var_id));
		}
		for (return_index, var_id) in output_var_ids.iter().enumerate()
		{
			self.code_writer.write(format!("field_{} : var_{}, ", return_index + capture_var_ids.len(), var_id));
		}
		self.active_funclet_state.as_mut().unwrap().capture_count = capture_var_ids.len();
		self.active_funclet_state.as_mut().unwrap().output_count = output_var_ids.len();
		self.active_funclet_state.as_mut().unwrap().next_funclet_input_types = Some(next_funclet_input_types);
		self.code_writer.write(format!("}}}};"));
	}

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
		self.active_funclet_result_type_id = None;
		self.active_funclet_state = None;
	}

	pub fn end_pipeline(&mut self)
	{
		self.code_writer.end_module();
		self.active_pipeline_name = None;
		self.reset_pipeline();
	}

	fn generate_type_definition(&mut self, type_id : ir::TypeId)
	{
		if self.has_been_generated.contains(& type_id)
		{
			return;
		}

		self.has_been_generated.insert(type_id);

		let typ = & self.types[& type_id];
		write!(self.type_code_writer, "// Type #{}: {:?}\n", type_id, typ);
		match typ
		{
			ir::Type::F32 => (),
			ir::Type::F64 => (),
			ir::Type::U8 => (),
			ir::Type::U16 => (),
			ir::Type::U32 => (),
			ir::Type::U64 => (),
			ir::Type::I8 => (),
			ir::Type::I16 => (),
			ir::Type::I32 => (),
			ir::Type::I64 => (),
			ir::Type::ConstRef { element_type } => (),
			ir::Type::MutRef { element_type } => (),
			ir::Type::ConstSlice { element_type } => (),
			ir::Type::MutSlice { element_type } => (),
			ir::Type::Array { element_type, length } => (),
			ir::Type::Tuple { fields } =>
			{
				write!(self.type_code_writer, "pub type type_{} = (", type_id);
				for (index, field_type_id) in fields.iter().enumerate()
				{
					let type_name = self.get_type_name(* field_type_id);
					write!(self.type_code_writer, "{}, ", type_name);
				}
				self.type_code_writer.write_str(");\n");
			}
			ir::Type::Struct { fields, byte_alignment, byte_size } =>
			{
				write!(self.type_code_writer, "pub struct type_{}", type_id);
				self.type_code_writer.write_str("{\n");
				for field in fields.iter()
				{
					let type_name = self.get_type_name(field.type_id);
					write!(self.type_code_writer, "\tpub {} : {},\n", field.name, type_name);
				}
				self.type_code_writer.write_str("}\n\n");
			}
			ir::Type::Slot { value_type, queue_stage, queue_place } =>
			{
				write!(self.type_code_writer, "pub type type_{} = {};\n", type_id, self.get_type_name(* value_type));

			}
			ir::Type::SchedulingJoin { input_types, output_types, extra } =>
			{

			}
			_ => panic!("Unimplemented type #{}: {:?}", type_id, typ),
			//_ => panic!("Unimplemented")
		}
	}

	fn get_type_name(& self, type_id : ir::TypeId) -> String
	{
		match & self.types[& type_id]
		{
			ir::Type::F32 => "f32".to_string(),
			ir::Type::F64 => "f64".to_string(),
			ir::Type::U8 => "u8".to_string(),
			ir::Type::U16 => "u16".to_string(),
			ir::Type::U32 => "u32".to_string(),
			ir::Type::U64 => "u64".to_string(),
			ir::Type::I8 => "i8".to_string(),
			ir::Type::I16 => "i16".to_string(),
			ir::Type::I32 => "i32".to_string(),
			ir::Type::I64 => "i64".to_string(),
			ir::Type::ConstRef { element_type } => ("& ").to_string() + self.get_type_name(* element_type).as_str(),
			ir::Type::MutRef { element_type } => ("&mut ").to_string() + self.get_type_name(* element_type).as_str(),
			ir::Type::ConstSlice { element_type } => ("& [").to_string() + self.get_type_name(* element_type).as_str() + "]",
			ir::Type::MutSlice { element_type } => ("&mut [").to_string() + self.get_type_name(* element_type).as_str() + "]",
			ir::Type::Array { element_type, length } => format!("[{}; {}]", self.get_type_name(* element_type), length),
			ir::Type::SchedulingJoin { input_types, output_types, extra } =>
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
			}
			_ => format!("type_{}", type_id)
		}
	}

	fn get_type_binding_info(&self, type_id : ir::TypeId) -> TypeBindingInfo
	{
		match & self.types[& type_id]
		{
			ir::Type::F32 => TypeBindingInfo { size : std::mem::size_of::<f32>(), alignment : std::mem::align_of::<f32>() },
			ir::Type::F64 => TypeBindingInfo { size : std::mem::size_of::<f64>(), alignment : std::mem::align_of::<f64>() },
			ir::Type::U8 => TypeBindingInfo { size : std::mem::size_of::<u8>(), alignment : std::mem::align_of::<u8>() },
			ir::Type::U16 => TypeBindingInfo { size : std::mem::size_of::<u16>(), alignment : std::mem::align_of::<u16>() },
			ir::Type::U32 => TypeBindingInfo { size : std::mem::size_of::<u32>(), alignment : std::mem::align_of::<u32>() },
			ir::Type::U64 => TypeBindingInfo { size : std::mem::size_of::<u64>(), alignment : std::mem::align_of::<u64>() },
			ir::Type::I8 => TypeBindingInfo { size : std::mem::size_of::<i8>(), alignment : std::mem::align_of::<i8>() },
			ir::Type::I16 => TypeBindingInfo { size : std::mem::size_of::<i16>(), alignment : std::mem::align_of::<i16>() },
			ir::Type::I32 => TypeBindingInfo { size : std::mem::size_of::<i32>(), alignment : std::mem::align_of::<i32>() },
			ir::Type::I64 => TypeBindingInfo { size : std::mem::size_of::<i64>(), alignment : std::mem::align_of::<i64>() },
			ir::Type::ConstRef { element_type } => panic!("Unimplemented"),
			ir::Type::MutRef { element_type } => panic!("Unimplemented"),
			ir::Type::ConstSlice { element_type } => panic!("Unimplemented"),
			ir::Type::MutSlice { element_type } => panic!("Unimplemented"),
			ir::Type::Array { element_type, length } => panic!("Unimplemented"),
			ir::Type::Struct { fields, byte_alignment, byte_size } => panic!("Unimplemented"),
			ir::Type::Slot{ value_type, .. } => self.get_type_binding_info(* value_type), // Probably not quite right
			_ => panic!("Unimplemented")
		}
	}

	pub fn build_constant_integer(&mut self, value : i64, type_id : ir::TypeId) -> usize
	{
		//let variable_id = self.variable_tracker.generate();
		self.generate_type_definition(type_id);
		let variable_id = self.variable_tracker.create_local(type_id);
		write!(self.code_writer, "let var_{} : {} = {};\n", variable_id, self.get_type_name(type_id), value);
		variable_id
	}

	pub fn build_constant_unsigned_integer(&mut self, value : u64, type_id : ir::TypeId) -> usize
	{
		//let variable_id = self.variable_tracker.generate();
		self.generate_type_definition(type_id);
		let variable_id = self.variable_tracker.create_local(type_id);
		write!(self.code_writer, "let var_{} : {} = {};\n", variable_id, self.get_type_name(type_id), value);
		variable_id
	}

	pub fn build_select_hack(&mut self, condition_var_id : usize, true_case_var_id : usize, false_case_var_id : usize) -> usize
	{
		//let variable_id = self.variable_tracker.generate();
		let true_type_id = self.variable_tracker.variable_types[& true_case_var_id];
		let false_type_id = self.variable_tracker.variable_types[& false_case_var_id];
		assert_eq!(true_type_id, false_type_id);
		let type_id = true_type_id;
		self.generate_type_definition(type_id);
		let variable_id = self.variable_tracker.create_local(type_id);
		// Too lazy to implement booleans for now
		write!(self.code_writer, "let var_{} : {} = if var_{} != 0 {{ var_{} }} else {{ var_{} }};\n", variable_id, self.get_type_name(type_id), condition_var_id, true_case_var_id, false_case_var_id);
		variable_id
	}

	pub fn begin_if_else(&mut self, condition_var_id : usize, output_type_ids : &[ir::TypeId]) -> Box<[usize]>
	{
		// Temporary fix
		self.reset_pipeline();

		write!(self.code_writer, "let ( ");
		let mut var_ids = Vec::<usize>::new();
		for (i, type_id) in output_type_ids.iter().enumerate()
		{
			self.generate_type_definition(* type_id);
			let var_id = self.variable_tracker.create_local(* type_id);

			write!(self.code_writer, "var_{} : {}", var_id, self.get_type_name(* type_id));
			if i < output_type_ids.len() - 1
			{
				write!(self.code_writer, ", ");
			}

			var_ids.push(var_id);
		}
		write!(self.code_writer, " ) = if var_{} !=0 {{ ", condition_var_id);

		var_ids.into_boxed_slice()
	}

	pub fn end_if_begin_else(&mut self, output_var_ids : &[usize])
	{
		// Temporary fix
		self.reset_pipeline();
		
		write!(self.code_writer, " ( ");
		for (i, var_id) in output_var_ids.iter().enumerate()
		{
			write!(self.code_writer, "{}", var_id);
			if i < output_var_ids.len() - 1
			{
				write!(self.code_writer, ", ");
			}
		}
		write!(self.code_writer, " ) }} else {{ ");
	}

	pub fn end_else(&mut self, output_var_ids : &[usize])
	{
		write!(self.code_writer, " ( ");
		for (i, var_id) in output_var_ids.iter().enumerate()
		{
			write!(self.code_writer, "{}", var_id);
			if i < output_var_ids.len() - 1
			{
				write!(self.code_writer, ", ");
			}
		}
		write!(self.code_writer, " ) }};\n");

		// Temporary fix
		self.reset_pipeline();
	}

	pub fn build_join(&mut self, callee_funclet_id : ir::FuncletId, captured_var_ids : &[usize], input_type_ids : &[ir::TypeId], output_type_ids : &[ir::TypeId]) -> usize
	{
		// Temporary fix
		self.reset_pipeline();

		let join_var_id = self.variable_tracker.generate();

		write!(self.code_writer, "let mut var_{} = move |instance : Instance<'state, 'cpu_functions, Callbacks>", join_var_id);
		for (i, type_id) in input_type_ids.iter().enumerate()
		{
			self.generate_type_definition(* type_id);
			write!(self.code_writer, ", arg_{} : {}", i + captured_var_ids.len(), self.get_type_name(* type_id));
		}
		write!(self.code_writer, "| {{ funclet{}_func(instance", callee_funclet_id);

		assert!(captured_var_ids.len() <= input_type_ids.len());

		for (i, var_id) in captured_var_ids.iter().enumerate()
		{
			write!(self.code_writer, ", var_{}", var_id);
		}

		for i in captured_var_ids.len() .. input_type_ids.len()
		{
			write!(self.code_writer, ", arg_{}", i);
		}

		write!(self.code_writer, " ) }};\n");

		join_var_id
	}

	pub fn begin_join(&mut self, input_type_ids : &[ir::TypeId], output_type_ids : &[ir::TypeId]) -> (usize, Box<[usize]>)
	{
		let join_var_id = self.variable_tracker.generate();
		write!(self.code_writer, "let mut var_{} = move |instance", join_var_id);
		let mut var_ids = Vec::<usize>::new();
		for (i, type_id) in input_type_ids.iter().enumerate()
		{
			self.generate_type_definition(* type_id);
			let var_id = self.variable_tracker.create_local(* type_id);

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

	pub fn end_join(&mut self, output_var_ids : &[usize])
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

	pub fn call_join(&mut self, join_var_id : usize, input_var_ids : &[usize])
	{
		// Temporary fix
		self.reset_pipeline();

		write!(self.code_writer, "return var_{}(instance", join_var_id);
		for (i, var_id) in input_var_ids.iter().enumerate()
		{
			write!(self.code_writer, ", var_{}", var_id);
		}
		write!(self.code_writer, ");");
	}

	pub fn build_external_cpu_function_call(&mut self, external_function_id : ir::ExternalCpuFunctionId, argument_vars : &[usize]) -> Box<[usize]>
	{
		let external_cpu_function = & self.external_cpu_functions[external_function_id];
		let call_result_var = self.variable_tracker.generate();
		let mut argument_string = String::new();
		for (index, argument) in argument_vars.iter().enumerate()
		{
			argument_string += format!("var_{}", * argument).as_str();
			if index + 1 < argument_vars.len()
			{
				argument_string += ", ";
			}
		}
		self.code_writer.write(format!("let var_{} = instance.cpu_functions.{}(instance.state, {});\n", call_result_var, external_cpu_function.name, argument_string));
		let mut output_variables = Vec::<usize>::new();
		for (i, output_type) in external_cpu_function.output_types.iter().enumerate()
		{
			//let var = self.variable_tracker.generate();
			let var = self.variable_tracker.create_local(* output_type);
			output_variables.push(var);
			self.code_writer.write(format!("let var_{} = var_{}.{};\n", var, call_result_var, i));
		};
		output_variables.into_boxed_slice()
	}

	fn build_create_buffer(&mut self, type_id : ir::TypeId) -> usize
	{
		let variable_id = self.variable_tracker.generate();

		let type_binding_info = self.get_type_binding_info(type_id); 
		let type_name = self.get_type_name(type_id);
		self.code_writer.write(format!("let mut var_{} = instance.state.get_device_mut().create_buffer(& wgpu::BufferDescriptor {{ label : None, size : {}, usage : wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::MAP_WRITE, mapped_at_creation : false}});\n", variable_id, type_binding_info.size));
		variable_id
	}

	fn build_create_buffer_with_data(&mut self, data_var : usize, type_id : ir::TypeId) -> usize
	{
		let variable_id = self.variable_tracker.generate();
		let type_binding_info = self.get_type_binding_info(type_id); 
		let type_name = self.get_type_name(type_id);
		self.code_writer.write(format!("let mut var_{} = instance.state.get_device_mut().create_buffer(& wgpu::BufferDescriptor {{ label : None, size : {}, usage : wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::MAP_WRITE, mapped_at_creation : false}});\n", variable_id, type_binding_info.size));
		self.code_writer.write(format!("instance.state.get_queue_mut().write_buffer(& var_{}, 0, & var_{}.to_ne_bytes() );\n", variable_id, data_var));
		variable_id
	}

	fn build_create_buffer_with_buffer_data(&mut self, data_var : usize, type_id : ir::TypeId) -> usize
	{
		let variable_id = self.variable_tracker.generate();
		let type_binding_info = self.get_type_binding_info(type_id); 
		let type_name = self.get_type_name(type_id);
		self.code_writer.write(format!("let mut var_{} = instance.state.get_device_mut().create_buffer(& wgpu::BufferDescriptor {{ label : None, size : {}, usage : wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::MAP_WRITE, mapped_at_creation : false}});\n", variable_id, type_binding_info.size));
		write!(self.code_writer, "{{\n");
		self.code_writer.write("let mut command_encoder = instance.state.get_device_mut().create_command_encoder(& wgpu::CommandEncoderDescriptor {label : None});\n".to_string());
		write!(self.code_writer, "command_encoder.copy_buffer_to_buffer(& var_{}, 0, & var_{}, 0, {});\n", data_var, variable_id, type_binding_info.size);
		self.code_writer.write("let command_buffer = command_encoder.finish();\n".to_string());
		self.code_writer.write("instance.state.get_queue_mut().submit([command_buffer]);\n".to_string());
		self.code_writer.write(format!("instance.state.get_device_mut().poll(wgpu::Maintain::Wait);\n"));
		self.code_writer.write("futures::executor::block_on(instance.state.get_queue_mut().on_submitted_work_done());\n".to_string());
		write!(self.code_writer, "}}\n");
		//self.code_writer.write(format!("queue.write_buffer(& var_{}, 0, & var_{}.to_ne_bytes() );\n", variable_id, data_var));
		variable_id
	}

	pub fn build_compute_dispatch(&mut self, external_function_id : ir::ExternalGpuFunctionId, dimension_vars : &[usize; 3], argument_vars : &[usize]) -> Box<[usize]>
	{
		let output_vars = self.generate_compute_dispatch_outputs(external_function_id);
		self.generate_compute_dispatch(external_function_id, dimension_vars, argument_vars, & output_vars);
		return output_vars;
	}

	pub fn build_compute_dispatch_with_outputs(&mut self, external_function_id : ir::ExternalGpuFunctionId, dimension_vars : &[usize; 3], argument_vars : &[usize], output_vars : &[usize])
	{
		self.generate_compute_dispatch(external_function_id, dimension_vars, argument_vars, output_vars);
	}
}