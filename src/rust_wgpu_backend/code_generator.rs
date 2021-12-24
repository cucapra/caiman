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

enum Command
{
	//SetExternalFunction(usize),
	//SetBindings{bindings : BTreeMap<(usize, usize), Binding>},
	DispatchCompute{external_function_id : ir::ExternalGpuFunctionId, dimension_vars : [usize; 3], argument_vars : Box<[usize]>, output_vars : Box<[usize]>},
}

#[derive(Default)]
struct SubmissionEncodingState
{
	//shader_module : shadergen::ShaderModule,
	//bindings : BTreeMap<(usize, usize), Binding>,
	commands : Vec<Command>,
}

struct TypeBindingInfo
{
	size : usize,
	alignment : usize,
}

pub struct CodeGenerator<'program>
{
	type_code_writer : CodeWriter,
	code_writer : CodeWriter, // the "everything else" for now
	types : Arena<ir::Type>,
	external_cpu_functions : & 'program [ir::ExternalCpuFunction],
	external_gpu_functions : & 'program [ir::ExternalGpuFunction],
	has_been_generated : HashSet<usize>,
	variable_tracker : VariableTracker,
	active_pipeline_name : Option<String>,
	use_recording : bool,
	active_submission_encoding_state : Option<SubmissionEncodingState>,
	active_external_gpu_function_id : Option<ir::ExternalGpuFunctionId>,
	active_shader_module : Option<shadergen::ShaderModule>
}

impl<'program> CodeGenerator<'program>
{
	pub fn new(types : Arena<ir::Type>, external_cpu_functions : & 'program [ir::ExternalCpuFunction], external_gpu_functions : & 'program [ir::ExternalGpuFunction]) -> Self
	{
		let variable_tracker = VariableTracker::new();
		let type_code_writer = CodeWriter::new();
		let code_writer = CodeWriter::new();
		let has_been_generated = HashSet::new();
		Self {type_code_writer, code_writer, types, has_been_generated, variable_tracker, external_cpu_functions, external_gpu_functions, active_pipeline_name : None, use_recording : true, active_submission_encoding_state : None, active_external_gpu_function_id : None, active_shader_module : None}
	}

	pub fn finish(&mut self) -> String
	{
		self.type_code_writer.finish() + self.code_writer.finish().as_str()
	}

	fn enqueue_compute_dispatch(&mut self, external_function_id : ir::ExternalCpuFunctionId, dimensions : &[usize; 3], argument_vars : &[usize]) -> Option<Box<[usize]>>
	{
		if self.active_submission_encoding_state.is_none()
		{
			self.active_submission_encoding_state = Some(Default::default());
		}

		if let Some(submission_encoding_state) = self.active_submission_encoding_state.as_mut()
		{
			let mut output_vars = Vec::<usize>::new();

			let external_gpu_function = & self.external_gpu_functions[external_function_id];
			for (output_index, output_type_id) in external_gpu_function.output_types.iter().enumerate()
			{
				let variable_id = self.variable_tracker.create_in_encoding(* output_type_id);
				output_vars.push(variable_id);
			}

			submission_encoding_state.commands.push(Command::DispatchCompute{external_function_id, dimension_vars : * dimensions, argument_vars : argument_vars.to_vec().into_boxed_slice(), output_vars : output_vars.clone().into_boxed_slice()});
			return Some(output_vars.into_boxed_slice());
		}

		None
	}

	fn set_active_external_gpu_function(&mut self, external_function_id : ir::ExternalGpuFunctionId)
	{
		if let Some(previous_id) = self.active_external_gpu_function_id.as_ref()
		{
			if * previous_id == external_function_id
			{
				return;
			}
		}

		self.active_external_gpu_function_id = None;

		let external_gpu_function = & self.external_gpu_functions[external_function_id];

		let mut shader_module = match & external_gpu_function.shader_module_content
		{
			ir::ShaderModuleContent::Wgsl(text) => shadergen::ShaderModule::new_with_wgsl(text.as_str())
		};

		self.code_writer.write_str("let module = device.create_shader_module(& wgpu::ShaderModuleDescriptor { label : None, source : wgpu::ShaderSource::Wgsl(std::borrow::Cow::from(\"");
		/*match & external_gpu_function.shader_module_content
		{
			ir::ShaderModuleContent::Wgsl(text) => self.code_writer.write_str(text.as_str())
		}*/
		self.code_writer.write_str(shader_module.compile_wgsl_text().as_str());
		self.code_writer.write_str("\"))});\n");

		self.active_external_gpu_function_id = Some(external_function_id);
		self.active_shader_module = Some(shader_module);
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

		self.code_writer.write("let bind_group_layout_entries = [".to_string());
		for (binding, (_input_opt, output_opt)) in bindings.iter()
		{
			let is_read_only : bool = output_opt.is_none();
			self.code_writer.write("wgpu::BindGroupLayoutEntry { ".to_string());
			self.code_writer.write(format!("binding : {}, visibility : wgpu::ShaderStages::COMPUTE, ty : wgpu::BindingType::Buffer{{ ty : wgpu::BufferBindingType::Storage {{ read_only : {} }}, has_dynamic_offset : false, min_binding_size : None}}, count : None", binding, is_read_only));
			self.code_writer.write(" }, ".to_string());
		}
		self.code_writer.write("];\n".to_string());

		self.code_writer.write("let bind_group_layout = device.create_bind_group_layout(& wgpu::BindGroupLayoutDescriptor { label : None, entries : & bind_group_layout_entries});\n".to_string());

		self.code_writer.write("let entries = [".to_string());
		for (binding, (input_opt, output_opt)) in bindings.iter()
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
		self.code_writer.write("let bind_group = device.create_bind_group(& wgpu::BindGroupDescriptor {label : None, layout : & bind_group_layout, entries : & entries});\n".to_string());

		self.code_writer.write("let pipeline_layout = device.create_pipeline_layout(& wgpu::PipelineLayoutDescriptor { label : None, bind_group_layouts : & [& bind_group_layout], push_constant_ranges : & []});\n".to_string());
		self.code_writer.write("let pipeline = device.create_compute_pipeline(& wgpu::ComputePipelineDescriptor {label : None, layout : Some(& pipeline_layout), module : & module, entry_point : & \"main\"});\n".to_string());

		output_staging_variables.into_boxed_slice()
	}

	fn reset_pipeline(&mut self)
	{
		self.active_external_gpu_function_id = None;
		self.active_shader_module = None;
	}

	/*fn get_var_name(&mut self, var_id : usize) -> String
	{

	}*/

	fn require_local(&mut self, variable_ids : &[usize])
	{
		for variable_id in variable_ids.iter()
		{
			match self.variable_tracker.variable_states[variable_id]
			{
				VariableState::InEncoding => self.flush_submission(),
				_ => ()
			}

			match self.variable_tracker.variable_states[variable_id]
			{
				VariableState::Local => (),
				VariableState::OnGpu =>
				{
					let type_id = self.variable_tracker.variable_types[variable_id];
					let range_var_id = self.variable_tracker.generate();
					let output_temp_var_id = self.variable_tracker.generate();
					let slice_var_id = self.variable_tracker.generate();
					let future_var_id = self.variable_tracker.generate();
					let type_binding_info = self.get_type_binding_info(type_id); 
					let type_name = self.get_type_name(type_id);
					
					self.code_writer.write(format!("let var_{} = var_{}.slice(0..);\n", slice_var_id, variable_id));
					self.code_writer.write(format!("let var_{} = var_{}.map_async(wgpu::MapMode::Read);\n", future_var_id, slice_var_id));
					self.code_writer.write(format!("device.poll(wgpu::Maintain::Wait);\n"));
					self.code_writer.write(format!("futures::executor::block_on(var_{});;\n", future_var_id));
					self.code_writer.write(format!("let var_{} = var_{}.get_mapped_range();\n", range_var_id, slice_var_id));
					self.code_writer.write(format!("let var_{} = * unsafe {{ std::mem::transmute::<* const u8, & {}>(var_{}.as_ptr()) }};\n", variable_id, type_name, range_var_id));
				}
				_ => panic!("Unimplemented")
			}

			self.variable_tracker.variable_states.insert(* variable_id, VariableState::Local);
			/*let type_id = match variable_state
			{

			};*/
		}
	}

	fn require_on_gpu(&mut self, variable_ids : &[usize])
	{
		for variable_id in variable_ids.iter()
		{
			/*match self.variable_tracker.variable_states[variable_id]
			{
				VariableState::InEncoding => self.flush_submission(),
				_ => ()
			}*/

			match self.variable_tracker.variable_states[variable_id]
			{
				VariableState::InEncoding => (),
				VariableState::Local =>
				{
					let type_id = self.variable_tracker.variable_types[variable_id];
					let type_binding_info = self.get_type_binding_info(type_id); 
					let type_name = self.get_type_name(type_id);
					let temp_id = self.variable_tracker.generate();
					self.code_writer.write(format!("let mut var_{} = device.create_buffer(& wgpu::BufferDescriptor {{ label : None, size : {}, usage : wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::MAP_WRITE, mapped_at_creation : false}});\n", temp_id, type_binding_info.size));
					self.code_writer.write(format!("queue.write_buffer(& var_{}, 0, & var_{}.to_ne_bytes() );\n", temp_id, variable_id));
					self.code_writer.write(format!("let var_{} = var_{};\n", variable_id, temp_id));
				}
				VariableState::OnGpu => (),
				_ => panic!("Unimplemented")
			}

			self.variable_tracker.variable_states.insert(* variable_id, VariableState::Local);
			/*let type_id = match variable_state
			{

			};*/
		}
	}

	fn require_exclusive(&mut self, variable_ids : &[usize])
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
		
		//let mut output_variables = Vec::<usize>::new();
		self.code_writer.write(format!("let ("));
		for output_index in 0 .. external_gpu_function.output_types.len()
		{
			//let var_id = self.variable_tracker.generate();
			//output_variables.push(var_id);
			let var_id = output_vars[output_index];
			self.code_writer.write(format!("var_{}, ", var_id));
		}
		self.code_writer.write(format!(") = "));

		self.code_writer.write("{\n".to_string());

		self.code_writer.write("let mut command_encoder = device.create_command_encoder(& wgpu::CommandEncoderDescriptor {label : None});\n".to_string());
		
		self.code_writer.write_str("{\n");
		self.code_writer.write("let mut compute_pass = command_encoder.begin_compute_pass(& wgpu::ComputePassDescriptor {label : None});\n".to_string());
		self.code_writer.write("compute_pass.set_pipeline(& pipeline);\n".to_string());
		self.code_writer.write("compute_pass.set_bind_group(0, & bind_group, & []);\n".to_string());
		self.code_writer.write(format!("compute_pass.dispatch(var_{}.try_into().unwrap(), var_{}.try_into().unwrap(), var_{}.try_into().unwrap());\n", dimension_vars[0], dimension_vars[1], dimension_vars[2]));
		self.code_writer.write_str("}\n");

		self.code_writer.write("let command_buffer = command_encoder.finish();\n".to_string());
		self.code_writer.write("queue.submit([command_buffer]);\n".to_string());
		self.code_writer.write(format!("device.poll(wgpu::Maintain::Wait);\n"));
		self.code_writer.write("futures::executor::block_on(queue.on_submitted_work_done());\n".to_string());

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

		for var_id in output_vars.iter()
		{
			self.variable_tracker.transition_to_queue(* var_id);
			self.variable_tracker.transition_to_on_gpu(* var_id);
			//self.variable_tracker.transition_to_local(* var_id);
		}
	}

	fn flush_submission(&mut self)
	{
		let mut active_submission_encoding_state = None;
		std::mem::swap(&mut self.active_submission_encoding_state, &mut active_submission_encoding_state);

		if let Some(submission_encoding_state) = active_submission_encoding_state
		{
			for command in submission_encoding_state.commands.iter()
			{
				match command
				{
					Command::DispatchCompute{external_function_id, dimension_vars, argument_vars, output_vars} =>
					{
						self.generate_compute_dispatch(* external_function_id, dimension_vars, argument_vars, output_vars);
					}
				}
			}

		}

		//self.active_submission_encoding_state = None;
	}

	pub fn insert_comment(&mut self, comment_string : &str)
	{
		self.code_writer.write(format!("// {}\n", comment_string));
	}

	pub fn begin_pipeline(&mut self, pipeline_name : &str, input_types : &[ir::TypeId], output_types : &[ir::TypeId]) -> Box<[usize]>
	{
		self.reset_pipeline();

		self.active_pipeline_name = Some(String::from(pipeline_name));
		self.code_writer.begin_module(pipeline_name);

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

			let mut tuple_fields = Vec::<ir::TypeId>::new();
			for output_index in 0 .. output_types.len()
			{
				let output_type = output_types[output_index];
				tuple_fields.push(output_type);
			}
			let type_id = self.types.create(ir::Type::Tuple{fields : tuple_fields.into_boxed_slice()});
			self.generate_type_definition(type_id);
			write!(self.code_writer, "pub type {} = super::super::{};\n", pipeline_name, self.get_type_name(type_id));
		}
		self.code_writer.end_module();

		self.code_writer.write(format!("pub trait CpuFunctions\n{{\n"));
		for external_cpu_function in self.external_cpu_functions.iter()
		{
			self.code_writer.write(format!("\tfn {}(&self", external_cpu_function.name));
			for (input_index, input_type) in external_cpu_function.input_types.iter().enumerate()
			{
				self.code_writer.write(format!(", _ : {}", self.get_type_name(*input_type)));
			}
			self.code_writer.write(format!(") -> outputs::{};\n", external_cpu_function.name));
		}
		self.code_writer.write(format!("}}\n"));

		let mut argument_variable_ids = Vec::<usize>::new();
		self.code_writer.write(format!("pub fn run<F>(device : &mut wgpu::Device, queue : &mut wgpu::Queue, cpu_functions : & F"));
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

		self.code_writer.write(format!(" ) -> outputs::{}\n\twhere F : CpuFunctions", pipeline_name));
		self.code_writer.write("\n{\n\tuse std::convert::TryInto;\n".to_string());
		argument_variable_ids.into_boxed_slice()
	}

	pub fn build_return(&mut self, output_var_ids : &[usize])
	{
		self.require_local(output_var_ids);
		self.code_writer.write(format!("return outputs::{} {{", self.active_pipeline_name.as_ref().unwrap().as_str()));
		for (return_index, var_id) in output_var_ids.iter().enumerate()
		{
			self.code_writer.write(format!("field_{} : var_{}, ", return_index, var_id));
		}
		self.code_writer.write(format!("}};"));
	}

	pub fn end_pipeline(&mut self)
	{
		self.code_writer.write("}\n".to_string());
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
				write!(self.type_code_writer, "pub struct type_{}", type_id);
				self.type_code_writer.write_str("{\n");
				for (index, field_type_id) in fields.iter().enumerate()
				{
					let type_name = self.get_type_name(* field_type_id);
					write!(self.type_code_writer, "\tpub field_{} : {},\n", index, type_name);
				}
				self.type_code_writer.write_str("}\n\n");
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
			_ => panic!("Unimplemented")
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
		self.code_writer.write(format!("let var_{} = cpu_functions.{}({});\n", call_result_var, external_cpu_function.name, argument_string));
		let mut output_variables = Vec::<usize>::new();
		for (i, output_type) in external_cpu_function.output_types.iter().enumerate()
		{
			//let var = self.variable_tracker.generate();
			let var = self.variable_tracker.create_local(* output_type);
			output_variables.push(var);
			self.code_writer.write(format!("let var_{} = var_{}.field_{};\n", var, call_result_var, i));
		};
		output_variables.into_boxed_slice()
	}

	fn build_create_buffer(&mut self, type_id : ir::TypeId) -> usize
	{
		let variable_id = self.variable_tracker.generate();

		let type_binding_info = self.get_type_binding_info(type_id); 
		let type_name = self.get_type_name(type_id);
		self.code_writer.write(format!("let mut var_{} = device.create_buffer(& wgpu::BufferDescriptor {{ label : None, size : {}, usage : wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::MAP_WRITE, mapped_at_creation : false}});\n", variable_id, type_binding_info.size));
		variable_id
	}

	fn build_create_buffer_with_data(&mut self, data_var : usize, type_id : ir::TypeId) -> usize
	{
		let variable_id = self.variable_tracker.generate();
		let type_binding_info = self.get_type_binding_info(type_id); 
		let type_name = self.get_type_name(type_id);
		self.code_writer.write(format!("let mut var_{} = device.create_buffer(& wgpu::BufferDescriptor {{ label : None, size : {}, usage : wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::MAP_WRITE, mapped_at_creation : false}});\n", variable_id, type_binding_info.size));
		self.code_writer.write(format!("queue.write_buffer(& var_{}, 0, & var_{}.to_ne_bytes() );\n", variable_id, data_var));
		variable_id
	}

	fn build_create_buffer_with_buffer_data(&mut self, data_var : usize, type_id : ir::TypeId) -> usize
	{
		let variable_id = self.variable_tracker.generate();
		let type_binding_info = self.get_type_binding_info(type_id); 
		let type_name = self.get_type_name(type_id);
		self.code_writer.write(format!("let mut var_{} = device.create_buffer(& wgpu::BufferDescriptor {{ label : None, size : {}, usage : wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::MAP_WRITE, mapped_at_creation : false}});\n", variable_id, type_binding_info.size));
		write!(self.code_writer, "{{\n");
		self.code_writer.write("let mut command_encoder = device.create_command_encoder(& wgpu::CommandEncoderDescriptor {label : None});\n".to_string());
		write!(self.code_writer, "command_encoder.copy_buffer_to_buffer(& var_{}, 0, & var_{}, 0, {});\n", data_var, variable_id, type_binding_info.size);
		self.code_writer.write("let command_buffer = command_encoder.finish();\n".to_string());
		self.code_writer.write("queue.submit([command_buffer]);\n".to_string());
		self.code_writer.write(format!("device.poll(wgpu::Maintain::Wait);\n"));
		self.code_writer.write("futures::executor::block_on(queue.on_submitted_work_done());\n".to_string());
		write!(self.code_writer, "}}\n");
		//self.code_writer.write(format!("queue.write_buffer(& var_{}, 0, & var_{}.to_ne_bytes() );\n", variable_id, data_var));
		variable_id
	}

	pub fn build_compute_dispatch(&mut self, external_function_id : ir::ExternalGpuFunctionId, dimension_vars : &[usize; 3], argument_vars : &[usize]) -> Box<[usize]>
	{
		if let Some(output_vars) = self.enqueue_compute_dispatch(external_function_id, dimension_vars, argument_vars)
		{
			//self.flush_submission();
			return output_vars;
		}

		panic!("Did not use recording!");
	}
}