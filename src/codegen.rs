use crate::ir;
use crate::shadergen;
use crate::arena::Arena;
use std::default::Default;
use std::collections::HashMap;
use std::collections::HashSet;
use crate::rust_wgpu_backend::code_writer::CodeWriter;
use crate::rust_wgpu_backend::code_generator::CodeGenerator;
use std::fmt::Write;
use crate::id_generator::IdGenerator;

enum VariableState
{
	Dead,
}

#[derive(Default)]
struct VariableTracker
{
	id_generator : IdGenerator
}

impl VariableTracker
{
	fn new() -> Self
	{
		Self { id_generator : IdGenerator::new() }
	}

	fn generate(&mut self) -> usize
	{
		self.id_generator.generate()
	}
}

pub struct CodeGen<'program>
{
	program : & 'program ir::Program,
	code_generator : CodeGenerator<'program>,
}

impl<'program> CodeGen<'program>
{
	pub fn new(program : & 'program ir::Program) -> Self
	{
		Self { program : & program, code_generator : CodeGenerator::new(program.types.clone(), program.external_cpu_functions.as_slice(), program.external_gpu_functions.as_slice()) }
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

		let mut node_results = Vec::<NodeResult>::new();

		let argument_variable_ids = self.code_generator.begin_pipeline(pipeline_name, &funclet.input_types, &funclet.output_types);		

		for (node_id, node) in funclet.nodes.iter().enumerate()
		{
			self.code_generator.insert_comment(format!(" node #{}: {:?}", node_id, node).as_str());
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
					let variable_id = self.code_generator.build_constant_integer(* value, * type_id);
					NodeResult::SingleOutput(variable_id)
				}
				ir::Node::ConstantUnsignedInteger(value, type_id) =>
				{
					let variable_id = self.code_generator.build_constant_unsigned_integer(* value, * type_id);
					NodeResult::SingleOutput(variable_id)
				}
				ir::Node::CallExternalCpu { external_function_id, arguments } =>
				{
					let mut argument_vars = Vec::<usize>::new();
					for (index, argument) in arguments.iter().enumerate()
					{
						argument_vars.push(force_single_output(& node_results[* argument]));
					}
					let output_variables = self.code_generator.build_external_cpu_function_call(* external_function_id, argument_vars.as_slice());
					NodeResult::MultipleOutput(output_variables)
				}
				ir::Node::CallExternalGpuCompute {external_function_id, arguments, dimensions} =>
				{
					let dimension_vars = [
						force_single_output(& node_results[dimensions[0]]),
						force_single_output(& node_results[dimensions[1]]),
						force_single_output(& node_results[dimensions[2]])
					];

					let mut argument_vars = Vec::<usize>::new();
					for argument in arguments.iter()
					{
						argument_vars.push(force_single_output(& node_results[* argument]));
					}

					NodeResult::MultipleOutput(self.code_generator.build_compute_dispatch(* external_function_id, & dimension_vars, argument_vars.as_slice()))
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
				let mut output_var_ids = Vec::<usize>::new();
				for (return_index, node_index) in return_values.iter().enumerate()
				{
					output_var_ids.push(force_single_output(& node_results[* node_index]));
				}
				self.code_generator.build_return(output_var_ids.as_slice());
			}
		}

		self.code_generator.end_pipeline();
	}

	pub fn generate<'codegen>(& 'codegen mut self) -> String
	{
		for pipeline in self.program.pipelines.iter()
		{
			self.generate_cpu_function(pipeline.entry_funclet, pipeline.name.as_str());
		}

		return self.code_generator.finish();
	}
}

#[cfg(test)]
mod tests
{
	use crate::codegen;
	use crate::ir;
}
