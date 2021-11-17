use crate::ir;
use std::default::Default;
use std::collections::HashMap;

enum CpuInstruction
{
	//AdapterRequestDevice { adapter_id : usize,  },
	GetDefaultDevice { device_id : usize },
	DeviceGetDefaultQueue { device_id : usize, queue_id : usize },
	DeviceCreateCommandEncoder { device_id : usize, encoder_id : usize },
	//DeviceCreateBuffer,
	CommandEncoderFinish { encoder_id : usize, command_buffer_id : usize },
	InstantiateCommandBuffer { device_id : usize, command_buffer_id : usize, command_buffer_instance_id : usize },
	QueueSubmit { queue_id : usize, command_buffer_id : usize },
	CallExternal { external_function_id : ir::ExternalCpuFunctionId },
	CommandEncoderBeginComputePass { compute_encoder_id : usize, compute_pass_var : usize },
	ComputePassDispatch { compute_pass_var : usize, dimension_vars : [usize; 3] },
}

enum CommandBufferInstruction
{
	//DispatchCompute { external_function_id : ir::ExternalGpuFunctionId, arguments : Box<[usize]>, dimensions : [usize; 3] }
}

struct CpuFunction
{
	instructions : Box<[CpuInstruction]>
}

struct CommandBuffer
{
	instructions : Box<[CommandBufferInstruction]>
}

struct CompiledProgram
{
	cpu_functions : Box<[CpuFunction]>,
	command_buffers : Box<[CommandBuffer]>
}

impl CompiledProgram
{
	/*fn new() -> Self
	{
		Self { functions : vec![].into_boxed_slice() }
	}*/

	fn generate_string(& self) -> String
	{
		let mut output_string = String::new();

		for function in self.cpu_functions.iter()
		{
			for instruction in function.instructions.iter()
			{
				match instruction
				{
					CpuInstruction::GetDefaultDevice { device_id } =>
					{
						output_string.push_str(format!("let var_{} = self.device_id;", device_id).as_str());
					}
					CpuInstruction::DeviceGetDefaultQueue { device_id, queue_id } =>
					{
						output_string.push_str(format!("let var_{} = wgpu_device_get_default_queue(var_{});", queue_id, device_id).as_str());
					}
					CpuInstruction::DeviceCreateCommandEncoder { device_id, encoder_id } =>
					{
						output_string.push_str(format!("let var_{} = wgpu_device_create_command_encoder(var_{}, None);", encoder_id, device_id).as_str());
					}
					CpuInstruction::CommandEncoderFinish { encoder_id, command_buffer_id } =>
					{
						output_string.push_str(format!("let var_{} = wgpu_command_encoder_finish(var_{}, None);", command_buffer_id, encoder_id).as_str());
					}
					CpuInstruction::QueueSubmit { queue_id, command_buffer_id } =>
					{
						output_string.push_str(format!("wgpu_queue_submit(var_{}, (& var_{} as *wgpu_core::id::CommandBufferId), 1);", queue_id, command_buffer_id).as_str());
					}
					_ => panic!("Unknown instruction")
				}
			}
		}

		/*for command_buffer in self.command_buffers.iter()
		{
			for instruction in command_buffer.instructions.iter()
			{
				match instruction
				{
					CommandBufferInstruction::DispatchCompute { external_function_id, arguments, dimensions } =>
					{

					}
					_ => panic!("Unknown instruction")
				}
			}
		}*/
		
		return output_string;
	}
}

struct CodeWriter
{
	code_string : String
}

impl CodeWriter
{
	fn new() -> Self
	{
		Self { code_string : String::new() }
	}

	fn finish(&mut self) -> String
	{
		self.code_string.clone()
	}

	/*fn begin_pipeline(&mut self, name : &String)
	{

	}

	fn end_pipeline(&mut self)
	{

	}*/

	/*fn write_line(&mut self, line : &String)
	{

	}*/

	fn write(&mut self, text : String)
	{
		self.code_string += text.as_str();
	}

	fn write_str(&mut self, text : &str)
	{
		self.code_string += text;
	}
}

struct VariableTracker
{
	next_id : usize
}

impl VariableTracker
{
	fn new() -> Self
	{
		Self { next_id : 0 }
	}

	fn generate(&mut self) -> usize
	{
		let id = self.next_id;
		self.next_id += 1;
		id
	}
}

//#[derive(Default)]
struct CodeGen<'program>
{
	program : & 'program ir::Program,
	//code_strings : Vec<String>,
	code_writer : CodeWriter
}

struct TypeBindingInfo
{
	size : usize,
	alignment : usize,
}

impl<'program> CodeGen<'program>
{
	fn new(program : & 'program ir::Program) -> Self
	{
		Self { program : & program, code_writer : CodeWriter::new()/*, code_strings : Vec::<String>::new()*/ }
	}

	/*fn generate_command_buffer(&mut self, funclet_id : ir::FuncletId) -> usize
	{
		let funclet = & self.program.funclets[& funclet_id];
		assert_eq!(funclet.execution_scope, Some(ir::Scope::Gpu));

		for (node_id, node) in funclet.nodes.iter().enumerate()
		{
			
		}

		panic!("Unfinished")
	}*/

	fn generate_type_definition(&mut self, type_id : ir::TypeId)
	{
		let typ = & self.program.types[& type_id];
		self.code_writer.write(format!("// Type #{}: {:?}", type_id, typ));
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
			ir::Type::Struct { fields, byte_alignment, byte_size } =>
			{
				self.code_writer.write(format!("struct type_{}", type_id));
				self.code_writer.write_str("{\n");
				for field in fields.iter()
				{
					let type_name = self.get_type_name(type_id);
					self.code_writer.write(format!("\t{} : {},\n", field.name, type_name));
				}
				self.code_writer.write_str("}\n\n");
			}
			_ => panic!("Unimplemented")
		}
	}

	fn get_type_name(& self, type_id : ir::TypeId) -> String
	{
		match & self.program.types[& type_id]
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
		match & self.program.types[& type_id]
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

	fn generate_cpu_function(&mut self, funclet_id : ir::FuncletId)
	{
		let funclet = & self.program.funclets[& funclet_id];
		assert_eq!(funclet.execution_scope, Some(ir::Scope::Cpu));

		enum NodeResult
		{
			Error,
			SingleOutput(usize),
			MultipleOutput(Box<[usize]>),
		}

		fn force_single_output(result : & NodeResult) -> usize
		{
			if let NodeResult::SingleOutput(output) = result
			{
				return *output;
			}
			panic!("Not a single output node result")
		}

		let mut variable_tracker = VariableTracker::new();

		let mut argument_variable_ids = Vec::<usize>::new();
		let mut node_results = Vec::<NodeResult>::new();
		//let mut instructions = Vec::<CpuInstruction>::new();
		//let mut code_strings = Vec::<String>::new();
		let device_var = variable_tracker.generate();
		let queue_var = variable_tracker.generate();
		//instructions.push(CpuInstruction::DeviceGetDefaultQueue{ device_id : device_variable_id, queue_id : queue_variable_id });
		//code_strings.push(format!("let var_{}", device_var, queue_var));

		self.code_writer.write(format!("fn pipeline_{}(device : &mut wgpu::Device, queue : &mut wgpu::Queue", funclet_id));
		//self.code_strings.push("(".to_string());
		for (input_index, input_type) in funclet.input_types.iter().enumerate()
		{
			self.code_writer.write(", ".to_string());

			let variable_id = variable_tracker.generate();
			argument_variable_ids.push(variable_id);
			let type_name = self.get_type_name(*input_type);
			self.code_writer.write(format!("var_{} : {}", variable_id, type_name));

			/*if input_index + 1 < funclet.input_types.len()
			{
				self.code_strings.push(", ".to_string());
			}*/
		}
		self.code_writer.write(" )\n{\n".to_string());

		for (node_id, node) in funclet.nodes.iter().enumerate()
		{
			self.code_writer.write(format!("// node #{}: {:?}\n", node_id, node));
			let node_result = match node
			{
				ir::Node::Phi {index} => NodeResult::SingleOutput(argument_variable_ids[*index as usize]),
				ir::Node::ExtractResult { node_id, index } =>
				{
					if let NodeResult::MultipleOutput(output) = &node_results[*node_id]
					{
						NodeResult::SingleOutput(output[*index])
					}
					else
					{
						panic!("Not a multiple output node result");
						NodeResult::Error
					}
				}
				ir::Node::ConstantInteger(value, type_id) =>
				{
					let variable_id = variable_tracker.generate();
					self.code_writer.write(format!("let var_{} : {} = {};\n", variable_id, self.get_type_name(* type_id), value));
					NodeResult::SingleOutput(variable_id)
				}
				ir::Node::ConstantUnsignedInteger(value, type_id) =>
				{
					let variable_id = variable_tracker.generate();
					self.code_writer.write(format!("let var_{} : {} = {};\n", variable_id, self.get_type_name(* type_id), value));
					NodeResult::SingleOutput(variable_id)
				}
				ir::Node::CallExternalCpu { external_function_id, arguments } =>
				{
					let external_cpu_function = & self.program.external_cpu_functions[* external_function_id];
					let call_result_var = variable_tracker.generate();
					let mut argument_string = String::new();
					for (index, argument) in arguments.iter().enumerate()
					{
						argument_string += format!("var_{}", force_single_output(& node_results[* argument])).as_str();
						if index + 1 < arguments.len()
						{
							argument_string += ", ";
						}
					}
					self.code_writer.write(format!("let var_{} = {}({});\n", call_result_var, external_cpu_function.name, argument_string));
					let mut output_variables = Vec::<usize>::new();
					for i in 0 .. external_cpu_function.output_types.len()
					{
						let var = variable_tracker.generate();
						output_variables.push(var);
						self.code_writer.write(format!("let var_{} = var_{}.field_{};\n", var, call_result_var, i));
					};
					//instructions.push(CpuInstruction::CallExternal{external_function_id : *external_function_id});
					NodeResult::MultipleOutput(output_variables.into_boxed_slice())
				}
				ir::Node::CallExternalGpuCompute {external_function_id, arguments, dimensions} =>
				{
					let external_gpu_function = & self.program.external_gpu_functions[* external_function_id];
					assert_eq!(external_gpu_function.input_types.len(), arguments.len());
					
					let mut output_staging_variables = Vec::<usize>::new();
					for output_index in 0 .. external_gpu_function.output_types.len()
					{
						let variable_id = variable_tracker.generate();
						output_staging_variables.push(variable_id);
						let type_id = external_gpu_function.output_types[output_index];

						let type_binding_info = self.get_type_binding_info(type_id); 
						let type_name = self.get_type_name(type_id);
						self.code_writer.write(format!("let mut var_{} = device.create_buffer(& wgpu::BufferDescriptor {{ label : None, size : {}, usage : wgpu::BufferUsages::all(), mapped_at_creation : false}});\n", variable_id, type_binding_info.size));
					};

					let mut input_staging_variables = Vec::<usize>::new();
					assert_eq!(arguments.len(), external_gpu_function.input_types.len());
					for input_index in 0 .. external_gpu_function.input_types.len()
					{
						let variable_id = variable_tracker.generate();
						input_staging_variables.push(variable_id);
						let type_id = external_gpu_function.input_types[input_index];

						let type_binding_info = self.get_type_binding_info(type_id); 
						let type_name = self.get_type_name(type_id);
						self.code_writer.write(format!("let mut var_{} = device.create_buffer(& wgpu::BufferDescriptor {{ label : None, size : {}, usage : wgpu::BufferUsages::all(), mapped_at_creation : false}});\n", variable_id, type_binding_info.size));
						self.code_writer.write(format!("queue.write_buffer(& var_{}, 0, unsafe {{ std::mem::transmute::<& {}, & [u8; {}]>(& var_{}) }} );\n", variable_id, type_name, type_binding_info.size, arguments[input_index]));
					}

					let dimension_vars = [
						force_single_output(& node_results[dimensions[0]]),
						force_single_output(& node_results[dimensions[1]]),
						force_single_output(& node_results[dimensions[2]])
					];
					
					self.code_writer.write("{\n".to_string());
					self.code_writer.write("let bind_group_layout_entries = [".to_string());
					let mut binding = 0usize;
					for input_type in external_gpu_function.input_types.iter()
					{
						self.code_writer.write("wgpu::BindGroupLayoutEntry { ".to_string());
						self.code_writer.write(format!("binding : {}, visibility : wgpu::ShaderStages::all(), ty : wgpu::BindingType::Buffer{{ ty : wgpu::BufferBindingType::Storage {{ read_only : true }}, has_dynamic_offset : false, min_binding_size : None}}, count : None", binding));
						self.code_writer.write(" }, ".to_string());
						binding += 1;
					}
					let output_binding_start = binding;
					for output_type in external_gpu_function.output_types.iter()
					{
						self.code_writer.write("wgpu::BindGroupLayoutEntry { ".to_string());
						self.code_writer.write(format!("binding : {}, visibility : wgpu::ShaderStages::all(), ty : wgpu::BindingType::Buffer{{ ty : wgpu::BufferBindingType::Storage {{ read_only : true }}, has_dynamic_offset : false, min_binding_size : None}}, count : None", binding));
						self.code_writer.write(" }, ".to_string());
						binding += 1;
					}
					self.code_writer.write("];\n".to_string());
					self.code_writer.write_str("let module = device.create_shader_module(& wgpu::ShaderModuleDescriptor { label : None, source : wgpu::ShaderSource::Wgsl(std::borrow::Cow::from(\"");
					self.code_writer.write_str(external_gpu_function.shader_text.as_str());
					self.code_writer.write_str("\n\n");
					{
						let mut binding = 0usize;
						let binding_count = external_gpu_function.input_types.len() + external_gpu_function.output_types.len();

						for input_index in 0 .. external_gpu_function.input_types.len()
						{
							let input_type = external_gpu_function.input_types[input_index];
							self.code_writer.write(format!("[[group(0), binding({})]] var<storage, read> input_{} : {};\n", binding, input_index, self.get_type_name(input_type)));
							binding += 1;
						}

						for output_index in 0 .. external_gpu_function.output_types.len()
						{
							let output_type = external_gpu_function.output_types[output_index];
							self.code_writer.write(format!("[[group(0), binding({})]] var<storage, read> output_{} : {};\n", binding, output_index, self.get_type_name(output_type)));
							binding += 1;
						}

						self.code_writer.write_str("[[stage(compute)]] fn main(");
						self.code_writer.write_str(")\n{\n");
						self.code_writer.write_str(external_gpu_function.name.as_str());
						self.code_writer.write_str("(");
						for input_index in 0 .. external_gpu_function.input_types.len()
						{
							let input_type = external_gpu_function.input_types[input_index];
							self.code_writer.write(format!("input_{}", input_index));
							self.code_writer.write_str(",");
						}
						for output_index in 0 .. external_gpu_function.output_types.len()
						{
							let output_type = external_gpu_function.output_types[output_index];
							self.code_writer.write(format!("output_{}", output_index));
							self.code_writer.write_str(",");
						}
						self.code_writer.write_str(");\n");
						self.code_writer.write_str("}\n");
					}
					self.code_writer.write_str("\"))});\n");
					//self.code_writer.write("let module = device.create_shader_module(& wgpu::ShaderModuleDescriptor { label : None, source : wgpu::ShaderSource::Wgsl(std::borrow::Cow::from(\"\"))});\n");
					self.code_writer.write("let bind_group_layout = device.create_bind_group_layout(& wgpu::BindGroupLayoutDescriptor { label : None, entries : & bind_group_layout_entries});\n".to_string());
					self.code_writer.write("let pipeline_layout = device.create_pipeline_layout(& wgpu::PipelineLayoutDescriptor { label : None, bind_group_layouts : & [& bind_group_layout], push_constant_ranges : & []});\n".to_string());
					self.code_writer.write("let pipeline = device.create_compute_pipeline(& wgpu::ComputePipelineDescriptor {label : None, layout : Some(& pipeline_layout), module : & module, entry_point : & \"main\"});\n".to_string());
					self.code_writer.write("let mut command_encoder = device.create_command_encoder(& wgpu::CommandEncoderDescriptor {label : None});\n".to_string());
					self.code_writer.write("let entries = [".to_string());
					binding = 0usize;
					for input_index in 0 .. arguments.len()
					{
						self.code_writer.write(format!("wgpu::BindGroupEntry {{binding : {}, resource : wgpu::BindingResource::Buffer(wgpu::BufferBinding{{buffer : & var_{}, offset : 0, size : None}}) }}", binding, input_staging_variables[input_index]));
						binding += 1;
					}
					for output_index in external_gpu_function.output_types.iter()
					{

						binding += 1;
					}
					self.code_writer.write("];\n".to_string());
					self.code_writer.write("let bind_group = device.create_bind_group(& wgpu::BindGroupDescriptor {label : None, layout : & bind_group_layout, entries : & entries});\n".to_string());
					
					self.code_writer.write_str("{\n");
					self.code_writer.write("let mut compute_pass = command_encoder.begin_compute_pass(& wgpu::ComputePassDescriptor {label : None});\n".to_string());
					self.code_writer.write("compute_pass.set_pipeline(& pipeline);\n".to_string());
					self.code_writer.write("compute_pass.set_bind_group(0, & bind_group, & []);\n".to_string());
					self.code_writer.write(format!("compute_pass.dispatch(var_{}.try_into().unwrap(), var_{}.try_into().unwrap(), var_{}.try_into().unwrap());\n", dimension_vars[0], dimension_vars[1], dimension_vars[2]));
					self.code_writer.write_str("}\n");

					self.code_writer.write("let command_buffer = command_encoder.finish();\n".to_string());
					self.code_writer.write("queue.submit([command_buffer]);\n".to_string());
					self.code_writer.write("futures::executor::block_on(queue.on_submitted_work_done());\n".to_string());

					let mut output_variables = Vec::<usize>::new();
					for output_index in 0 .. external_gpu_function.output_types.len()
					{
						let staging_var_id = output_staging_variables[output_index];
						let type_id = external_gpu_function.output_types[output_index];
						let range_var_id = variable_tracker.generate();
						let output_var_id = variable_tracker.generate();
						output_variables.push(output_var_id);
						let type_binding_info = self.get_type_binding_info(type_id); 
						let type_name = self.get_type_name(type_id);
						self.code_writer.write_str("{\n");
						self.code_writer.write(format!("let var_{} = var_{}.slice(0..).get_mapped_range();\n", range_var_id, staging_var_id));
						self.code_writer.write(format!("let var_{} = * unsafe {{ std::mem::transmute::<* const u8, & {}>(var_{}.as_ptr()) }};\n", output_var_id, type_name, range_var_id));
						//self.code_writer.write(format!("let var_{} = unsafe {{ let mut temp = std::mem::zeroed::<{}>(); std::mempcy(std::mem::transmute::<& {}, & [u8; {}]>(& temp), var_{}.as_ptr(), var_{}.len()); temp }};\n", output_var_id, type_name, type_name, type_binding_info.size, range_var_id, range_var_id));
						self.code_writer.write_str("}\n");
					}

					self.code_writer.write("}\n".to_string());

					NodeResult::MultipleOutput(output_variables.into_boxed_slice())
				}
				/*ir::Node::CallGpuCoordinator { funclet_id : callee_funclet_id, arguments } =>
				{
					let callee_funclet = & self.program.funclets[& callee_funclet_id];
					assert_eq!(callee_funclet.execution_scope, Some(ir::Scope::Gpu));

					let command_buffer_id = variable_tracker.generate();
					//let command_buffer_instance_id = self.generate_command_buffer(*callee_funclet_id);
					let mut output_variables = Vec::<usize>::new();
					for _ in 0 .. callee_funclet.output_types.len()
					{
						//force_single_output()
						output_variables.push(variable_tracker.generate());
					}
					{
						let gpu_funclet = & self.program.funclets[callee_funclet_id];
						assert_eq!(gpu_funclet.execution_scope, Some(ir::Scope::Gpu));

						let mut gpu_node_results = Vec::<NodeResult>::new();

						for (gpu_node_id, gpu_node) in gpu_funclet.nodes.iter().enumerate()
						{
							let gpu_node_result = match gpu_node
							{
								ir::Node::Phi {index} => node_results[*index],
								ir::Node::CallExternalGpuCompute {external_function_id, arguments, dimensions} =>
								{
									let dimension_vars = [
										force_single_output(gpu_node_results[dimensions[0]]),
										force_single_output(gpu_node_results[dimensions[1]]),
										force_single_output(gpu_node_results[dimensions[2]])
									];

									let compute_pass_var = variable_tracker.generate();
									instructions.push(CpuInstruction::DeviceCreateCommandEncoder { , compute_pass_var });
									instructions.push(CpuInstruction::CommandEncoderBeginComputePass { command_encoder_id, compute_pass_var });
									instructions.push(CpuInstruction::DeviceCreateComputePipeline );
									instructions.push(CpuInstruction::ComputePassSetPipeline );
									instructions.push(CpuInstruction::DeviceCreateBindGroup );
									instructions.push(CpuInstruction::ComputePassSetBindGroup );
									instructions.push(CpuInstruction::ComputePassDispatch { compute_pass_var, dimension_vars });
									instructions.push(CpuInstruction::CommandEncoderFinish );
								}
								_ => panic!("Unknown node")
							};
							gpu_node_results.push(gpu_node_result);
						};
						0usize
					}
					//instructions.push(CpuInstruction::InstantiateCommandBuffer { device_id : device_variable_id, command_buffer_id, command_buffer_instance_id });
					instructions.push(CpuInstruction::QueueSubmit { queue_id : queue_variable_id, command_buffer_id });
					// Still need to unpack outputs
					NodeResult::MultipleOutput(output_variables.into_boxed_slice())
				}*/
				_ => panic!("Unknown node")
			};
			node_results.push(node_result);

			//let line = format!("let var_{} = {};", variable_id, );
			//self.code_writer.write_line(& line);
		}

		self.code_writer.write("}\n".to_string());
	}

	/*fn generate(&mut self) -> CompiledProgram
	{
		//code_string.push("fn ");
		for pipeline in self.program.pipelines.iter()
		{
			self.code_writer.begin_pipeline(& self.name);
			self.generate_funclet(pipeline.entry_funclet);
			self.code_writer.end_pipeline();
		}

		return self.code_writer.finish();
	}*/

	fn generate<'codegen>(& 'codegen mut self) -> String
	{
		{
			let mut type_ids = Vec::<ir::TypeId>::new();

			for (type_id, _) in self.program.types.iter()
			{
				type_ids.push(* type_id);
			}

			type_ids.sort();

			for type_id in type_ids.iter()
			{
				//self.program.types[type_id]
				//self.code_writer.write();
				self.generate_type_definition(* type_id);
				self.code_writer.write_str("\n");
			}
		}

		for pipeline in self.program.pipelines.iter()
		{
			self.generate_cpu_function(pipeline.entry_funclet);
		}


		/*let mut final_output = String::new();
		for code_string in self.code_strings.iter()
		{
			final_output += code_string.as_str();
		}

		return final_output;*/

		return self.code_writer.finish();
	}
}

#[cfg(test)]
mod tests
{
	use crate::codegen;
	use crate::ir;

	/*#[test]
	fn test_compiled_program()
	{
		let compiled_program = codegen::CompiledProgram
		{
			cpu_functions : Box::new
			([
				codegen::CpuFunction
				{
					instructions : Box::new
					([
						codegen::CpuInstruction::GetDefaultDevice { device_id : 0 },
						codegen::CpuInstruction::DeviceGetDefaultQueue { device_id : 0, queue_id : 1 },
						codegen::CpuInstruction::DeviceCreateCommandEncoder { device_id : 0, encoder_id : 2 },
						codegen::CpuInstruction::CommandEncoderFinish { encoder_id : 2, command_buffer_id : 3 },
						codegen::CpuInstruction::QueueSubmit { queue_id : 1, command_buffer_id : 3}
					])
				}
			]),
			command_buffers : Box::new([])
		};
		let mut output_string = compiled_program.generate_string();
		//println!("Test output: {}", output_string);
		assert_eq!(output_string, "", "");
	}*/

	#[test]
	fn test_1()
	{
		let mut program = ir::Program::new();

		program.types.insert(0, ir::Type::I32);
		program.external_gpu_functions.push(ir::ExternalGpuFunction { name : "do_thing_on_gpu".to_string(), input_types : vec![0usize].into_boxed_slice(), output_types : vec![0usize].into_boxed_slice(), shader_text : "fn do_thing_on_gpu(a : i32, b : i32)\n{\n\n}\n".to_string() });
		program.external_cpu_functions.push(ir::ExternalCpuFunction { name : "do_thing_on_cpu".to_string(), input_types : vec![0usize].into_boxed_slice(), output_types : vec![0usize].into_boxed_slice() });

		let funclet = ir::Funclet 
		{
			input_types : vec![0usize].into_boxed_slice(),
			execution_scope : Some(ir::Scope::Cpu),
			output_types : vec![0usize].into_boxed_slice(),
			nodes : vec!
			[
				ir::Node::Phi { index : 0 },
				ir::Node::ConstantInteger(1, 0),
				ir::Node::CallExternalCpu { external_function_id : 0, arguments : vec![0usize].into_boxed_slice() },
				ir::Node::CallExternalGpuCompute { external_function_id : 0, arguments : vec![2usize].into_boxed_slice(), dimensions : [1, 1, 1] }
			].into_boxed_slice(),
			tail_edge : ir::TailEdge::Return { return_values : vec![].into_boxed_slice() }
		};
		program.funclets.insert(0, funclet);
		program.pipelines.push(ir::Pipeline { name : "pipeline_1".to_string(), entry_funclet : 0 });
		let mut codegen = codegen::CodeGen::new(& program);
		let output_string = codegen.generate();
		println!("{}", output_string);
		//assert_eq!(output_string, "", "");
	}
}
