use crate::ir;
use crate::shadergen;
use std::default::Default;
use std::collections::HashMap;
use crate::rust_wgpu_backend::code_writer::CodeWriter;
use std::fmt::Write;

pub struct VariableTracker
{
	next_id : usize
}

impl VariableTracker
{
	pub fn new() -> Self
	{
		Self { next_id : 0 }
	}

	pub fn generate(&mut self) -> usize
	{
		let id = self.next_id;
		self.next_id += 1;
		id
	}
}


//#[derive(Default)]
pub struct CodeGen<'program>
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
	pub fn new(program : & 'program ir::Program) -> Self
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

	fn generate_cpu_function(&mut self, funclet_id : ir::FuncletId, pipeline_name : &str)
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
		let device_var = variable_tracker.generate();
		let queue_var = variable_tracker.generate();

		//self.code_writer.write(format!("pub mod {} {{\n", pipeline_name));
		//self.code_writer.begin_pipeline(pipeline_name);
		self.code_writer.begin_module(pipeline_name);
		
		//self.code_writer.write(format!("pub mod outputs {{\n"));
		self.code_writer.begin_module("outputs");
		{
			for external_cpu_function in self.program.external_cpu_functions.iter()
			{
				//self.code_writer.write(format!("pub struct {}{{ ", external_cpu_function.name));
				self.code_writer.begin_struct(external_cpu_function.name.as_str());
				for (output_index, output_type) in external_cpu_function.output_types.iter().enumerate()
				{
					//self.code_writer.write(format!("pub field_{} : {}, ", output_index, self.get_type_name(*output_type)));
					self.code_writer.write_struct_field(output_index, self.get_type_name(*output_type).as_str());
				}
				//self.code_writer.write(format!("}}\n"));
				self.code_writer.end_struct();
			}

			//self.code_writer.write(format!("pub struct {} {{", pipeline_name));
			self.code_writer.begin_struct(pipeline_name);
			for output_index in 0 .. funclet.output_types.len()
			{
				let output_type = funclet.output_types[output_index];
				//self.code_writer.write(format!("pub field_{} : {}, ", output_index, self.get_type_name(output_type)));
				self.code_writer.write_struct_field(output_index, self.get_type_name(output_type).as_str());
			}
			//self.code_writer.write(format!("}}\n"));
			self.code_writer.end_struct();
		}
		self.code_writer.end_module();

		//self.code_writer.write(format!("}}\n"));

		self.code_writer.write(format!("pub trait CpuFunctions\n{{\n"));
		for external_cpu_function in self.program.external_cpu_functions.iter()
		{
			self.code_writer.write(format!("\tfn {}(&self", external_cpu_function.name));
			for (input_index, input_type) in external_cpu_function.input_types.iter().enumerate()
			{
				self.code_writer.write(format!(", _ : {}", self.get_type_name(*input_type)));
			}
			self.code_writer.write(format!(") -> outputs::{};\n", external_cpu_function.name));
		}
		self.code_writer.write(format!("}}\n"));
		

		self.code_writer.write(format!("pub fn run<F>(device : &mut wgpu::Device, queue : &mut wgpu::Queue, cpu_functions : & F"));
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

		self.code_writer.write(format!(" ) -> outputs::{}\n\twhere F : CpuFunctions", pipeline_name));
		self.code_writer.write("\n{\n\tuse std::convert::TryInto;\n".to_string());

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
					self.code_writer.write(format!("let var_{} = cpu_functions.{}({});\n", call_result_var, external_cpu_function.name, argument_string));
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
					
					let mut shader_module = match & external_gpu_function.shader_module_content
					{
						ir::ShaderModuleContent::Wgsl(text) => shadergen::ShaderModule::new_with_wgsl(text.as_str())
					};

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
					assert_eq!(arguments.len(), external_gpu_function.input_types.len());
					for input_index in 0 .. external_gpu_function.input_types.len()
					{
						let variable_id = variable_tracker.generate();
						input_staging_variables.push(variable_id);
						let type_id = external_gpu_function.input_types[input_index];

						let type_binding_info = self.get_type_binding_info(type_id); 
						let type_name = self.get_type_name(type_id);
						self.code_writer.write(format!("let mut var_{} = device.create_buffer(& wgpu::BufferDescriptor {{ label : None, size : {}, usage : wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::MAP_WRITE, mapped_at_creation : false}});\n", variable_id, type_binding_info.size));
						//self.code_writer.write(format!("queue.write_buffer(& var_{}, 0, unsafe {{ std::mem::transmute::<& {}, & [u8; {}]>(& var_{}) }} );\n", variable_id, type_name, type_binding_info.size, arguments[input_index]));
						self.code_writer.write(format!("queue.write_buffer(& var_{}, 0, & var_{}.to_ne_bytes() );\n", variable_id, arguments[input_index]));
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
							let variable_id = variable_tracker.generate();
							output_staging_variables.push(variable_id);
							let type_id = external_gpu_function.output_types[output_index];

							let type_binding_info = self.get_type_binding_info(type_id); 
							let type_name = self.get_type_name(type_id);
							self.code_writer.write(format!("let mut var_{} = device.create_buffer(& wgpu::BufferDescriptor {{ label : None, size : {}, usage : wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::MAP_WRITE, mapped_at_creation : false}});\n", variable_id, type_binding_info.size));
						}
					};

					let dimension_vars = [
						force_single_output(& node_results[dimensions[0]]),
						force_single_output(& node_results[dimensions[1]]),
						force_single_output(& node_results[dimensions[2]])
					];
					
					let mut output_variables = Vec::<usize>::new();
					self.code_writer.write(format!("let ("));
					for output_index in 0 .. external_gpu_function.output_types.len()
					{
						let var_id = variable_tracker.generate();
						output_variables.push(var_id);
						self.code_writer.write(format!("var_{}, ", var_id));
					}
					self.code_writer.write(format!(") = "));

					self.code_writer.write("{\n".to_string());

					self.code_writer.write("let bind_group_layout_entries = [".to_string());
					for (binding, (_input_opt, output_opt)) in bindings.iter()
					{
						let is_read_only : bool = output_opt.is_none();
						self.code_writer.write("wgpu::BindGroupLayoutEntry { ".to_string());
						self.code_writer.write(format!("binding : {}, visibility : wgpu::ShaderStages::COMPUTE, ty : wgpu::BindingType::Buffer{{ ty : wgpu::BufferBindingType::Storage {{ read_only : {} }}, has_dynamic_offset : false, min_binding_size : None}}, count : None", binding, is_read_only));
						self.code_writer.write(" }, ".to_string());
					}
					self.code_writer.write("];\n".to_string());

					self.code_writer.write_str("let module = device.create_shader_module(& wgpu::ShaderModuleDescriptor { label : None, source : wgpu::ShaderSource::Wgsl(std::borrow::Cow::from(\"");
					/*match & external_gpu_function.shader_module_content
					{
						ir::ShaderModuleContent::Wgsl(text) => self.code_writer.write_str(text.as_str())
					}*/
					self.code_writer.write_str(shader_module.compile_wgsl_text().as_str());
					self.code_writer.write_str("\"))});\n");
					self.code_writer.write("let bind_group_layout = device.create_bind_group_layout(& wgpu::BindGroupLayoutDescriptor { label : None, entries : & bind_group_layout_entries});\n".to_string());
					self.code_writer.write("let pipeline_layout = device.create_pipeline_layout(& wgpu::PipelineLayoutDescriptor { label : None, bind_group_layouts : & [& bind_group_layout], push_constant_ranges : & []});\n".to_string());
					self.code_writer.write("let pipeline = device.create_compute_pipeline(& wgpu::ComputePipelineDescriptor {label : None, layout : Some(& pipeline_layout), module : & module, entry_point : & \"main\"});\n".to_string());
					self.code_writer.write("let mut command_encoder = device.create_command_encoder(& wgpu::CommandEncoderDescriptor {label : None});\n".to_string());
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
						let range_var_id = variable_tracker.generate();
						let output_temp_var_id = variable_tracker.generate();
						let slice_var_id = variable_tracker.generate();
						let future_var_id = variable_tracker.generate();
						output_temp_variables.push(output_temp_var_id);
						let type_binding_info = self.get_type_binding_info(type_id); 
						let type_name = self.get_type_name(type_id);
						//self.code_writer.write_str("{\n");
						self.code_writer.write(format!("let var_{} = var_{}.slice(0..);\n", slice_var_id, staging_var_id));
						self.code_writer.write(format!("let var_{} = var_{}.map_async(wgpu::MapMode::Read);\n", future_var_id, slice_var_id));
						self.code_writer.write(format!("device.poll(wgpu::Maintain::Wait);\n"));
						self.code_writer.write(format!("futures::executor::block_on(var_{});;\n", future_var_id));
						self.code_writer.write(format!("let var_{} = var_{}.get_mapped_range();\n", range_var_id, slice_var_id));
						self.code_writer.write(format!("let var_{} = * unsafe {{ std::mem::transmute::<* const u8, & {}>(var_{}.as_ptr()) }};\n", output_temp_var_id, type_name, range_var_id));
						//self.code_writer.write(format!("let var_{} = unsafe {{ let mut temp = std::mem::zeroed::<{}>(); std::mempcy(std::mem::transmute::<& {}, & [u8; {}]>(& temp), var_{}.as_ptr(), var_{}.len()); temp }};\n", output_var_id, type_name, type_name, type_binding_info.size, range_var_id, range_var_id));
						//self.code_writer.write_str("}\n");
					}

					self.code_writer.write(format!("("));
					for output_temp_var_id in output_temp_variables.iter()
					{
						self.code_writer.write(format!("var_{}, ", output_temp_var_id));
					}
					self.code_writer.write(format!(")"));

					self.code_writer.write("};\n".to_string());

					NodeResult::MultipleOutput(output_variables.into_boxed_slice())
				}
				_ => panic!("Unknown node")
			};
			node_results.push(node_result);
		}

		match & funclet.tail_edge
		{
			ir::TailEdge::Return { return_values } =>
			{
				assert_eq!(return_values.len(), funclet.output_types.len());
				self.code_writer.write(format!("return outputs::{} {{", pipeline_name));
				for (return_index, node_index) in return_values.iter().enumerate()
				{
					self.code_writer.write(format!("field_{} : var_{}, ", return_index, force_single_output(& node_results[* node_index])));
				}
				self.code_writer.write(format!("}};"));
			}
		}

		self.code_writer.write("}\n".to_string());

		//self.code_writer.end_pipeline();
		//self.code_writer.write(format!("}}\n"));
		self.code_writer.end_module();
	}

	pub fn generate<'codegen>(& 'codegen mut self) -> String
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
				self.generate_type_definition(* type_id);
				self.code_writer.write_str("\n");
			}
		}

		for pipeline in self.program.pipelines.iter()
		{
			self.generate_cpu_function(pipeline.entry_funclet, pipeline.name.as_str());
		}

		return self.code_writer.finish();
	}
}

#[cfg(test)]
mod tests
{
	use crate::codegen;
	use crate::ir;
}
