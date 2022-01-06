use crate::ir;
use crate::shadergen;
use crate::arena::Arena;
use std::default::Default;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BTreeSet;
use crate::rust_wgpu_backend::code_generator::CodeGenerator;
use std::fmt::Write;

// This is a temporary hack
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum Value
{
	LocalVariable(usize),
	GpuBuffer(usize),
	Unknown(usize), // Temporary while resources are pulled out of the code generator
}

/*#[derive(Default)]
struct VariableTracker
{
	id_generator : IdGenerator,
	variable_values : HashMap<usize, VariableValue>,
	variable_types : HashMap<usize, ir::TypeId>
}

impl VariableTracker
{
	fn new() -> Self
	{
		Self { id_generator : IdGenerator::new(), variable_states : HashMap::<usize, VariableState>::new(), variable_types : HashMap::<usize, ir::TypeId>::new() }
	}

	fn create(&mut self, state : VariableState, type_id : ir::TypeId) -> usize
	{
		let id = self.id_generator.generate();
		self.variable_states.insert(id, state);
		id
	}
}*/

enum Location
{
	Constant,
	Local,
	Gpu,
}

/*
enum SubmissionState
{
	Encoding,
	Submitted,
	Done
}
*/

enum NodeResult
{
	Error,
	SingleOutput(Value),
	MultipleOutput(Box<[Value]>),
}

// Answers the question: Given a node at the current time, where is my data and what state is it in?
struct NodeResultTracker
{
	node_results : Vec<NodeResult>,
	tasks : Vec<Task>,
	node_task_ids : Vec<Option<TaskId>>,
}

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
struct TaskId(usize);

struct Task
{
	dependencies : BTreeSet<TaskId>,
}

struct TaskToken
{
	task_id : TaskId,
}

impl NodeResultTracker
{
	fn new() -> Self
	{
		Self { node_results : vec![], tasks : vec![], node_task_ids : vec![] }
	}

	fn get_node_result(&self, node_id : usize) -> &NodeResult
	{
		& self.node_results[node_id]
	}

	fn force_single_output(result : & NodeResult) -> Value
	{
		if let NodeResult::SingleOutput(output) = result
		{
			return *output;
		}
		panic!("Not a single output node result")
	}

	fn get_node_output_subvalue(& self, node_id : usize, index : usize) -> Value
	{
		if let NodeResult::MultipleOutput(output) = & self.get_node_result(node_id)
		{
			output[index]
		}
		else
		{
			panic!("Not a multiple output node result");
		}
	}

	fn get_node_output_value(& self, node_id : usize) -> Value
	{
		if let NodeResult::SingleOutput(output) = & self.get_node_result(node_id)
		{
			* output
		}
		else
		{
			panic!("Not a single output node result");
		}
	}

	fn begin_task(&mut self, local_variable_node_ids : &[usize], gpu_buffer_node_ids : &[usize]) -> TaskToken
	{
		// Doesn't account for nodes that need to be in two states

		for node_id in local_variable_node_ids
		{
			match &mut self.node_results[* node_id]
			{
				NodeResult::SingleOutput(value) =>
				{
					match value
					{
						Value::LocalVariable(_) => (),
						Value::GpuBuffer(id) => * value = Value::LocalVariable(* id),
						Value::Unknown(id) => * value = Value::LocalVariable(* id),
					}
				}
				_ => panic!("Node isn't a single output node!")
			}
		}

		for node_id in gpu_buffer_node_ids
		{
			match &mut self.node_results[* node_id]
			{
				NodeResult::SingleOutput(value) =>
				{
					match value
					{
						Value::LocalVariable(id) => * value = Value::GpuBuffer(* id),
						Value::GpuBuffer(_) => (),
						Value::Unknown(id) => * value = Value::GpuBuffer(* id),
					}
				}
				_ => panic!("Node isn't a single output node!")
			}
		}

		let token = TaskToken{ task_id : TaskId(self.tasks.len()) };
		let task = Task{ dependencies : BTreeSet::<TaskId>::new() };
		self.tasks.push(task);
		token
	}

	fn end_task(&mut self, token : TaskToken)
	{
		// To do: Support other patterns
		assert_eq!(token.task_id.0 + 1, self.tasks.len());
	}

	fn store_node_result(&mut self, node_id : usize, node_result : NodeResult, active_task : Option<&TaskToken>)
	{
		assert_eq!(node_id, self.node_results.len());
		self.node_results.push(node_result);

		let task_id_opt = if let Some(task_token) = active_task
		{
			Some(task_token.task_id)
		}
		else
		{
			None
		};

		self.node_task_ids.push(task_id_opt);
	}
}

// Answers the question: For a given node, how will it be used in the future?
struct NodeUsagePredictor
{

}

pub struct CodeGen<'program>
{
	program : & 'program ir::Program,
	code_generator : CodeGenerator<'program>
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

		let mut node_result_tracker = NodeResultTracker::new();


		fn force_var(value : Value) -> usize
		{
			match value
			{
				Value::Unknown(id) => id,
				Value::LocalVariable(id) => id,
				Value::GpuBuffer(id) => id,
				_ => panic!("Wrong type")
			}
		}

		let argument_variable_ids = self.code_generator.begin_pipeline(pipeline_name, &funclet.input_types, &funclet.output_types);		

		for (current_node_id, node) in funclet.nodes.iter().enumerate()
		{
			self.code_generator.insert_comment(format!(" node #{}: {:?}", current_node_id, node).as_str());
			match node
			{
				ir::Node::Phi {index} =>
				{
					let node_result = NodeResult::SingleOutput(Value::Unknown(argument_variable_ids[*index as usize]));
					node_result_tracker.store_node_result(current_node_id, node_result, None);
				}
				ir::Node::ExtractResult { node_id, index } =>
				{
					let node_result = NodeResult::SingleOutput(node_result_tracker.get_node_output_subvalue(* node_id, * index));
					node_result_tracker.store_node_result(current_node_id, node_result, None);
				}
				ir::Node::ConstantInteger(value, type_id) =>
				{
					let variable_id = self.code_generator.build_constant_integer(* value, * type_id);
					let node_result = NodeResult::SingleOutput(Value::LocalVariable(variable_id));
					node_result_tracker.store_node_result(current_node_id, node_result, None);
				}
				ir::Node::ConstantUnsignedInteger(value, type_id) =>
				{
					let variable_id = self.code_generator.build_constant_unsigned_integer(* value, * type_id);
					let node_result = NodeResult::SingleOutput(Value::LocalVariable(variable_id));
					node_result_tracker.store_node_result(current_node_id, node_result, None);
				}
				ir::Node::CallExternalCpu { external_function_id, arguments } =>
				{
					let token = node_result_tracker.begin_task(arguments, &[]);

					let mut argument_vars = Vec::<usize>::new();
					for (index, argument) in arguments.iter().enumerate()
					{
						argument_vars.push(force_var(node_result_tracker.get_node_output_value(* argument)));
					}
					let raw_outputs = self.code_generator.build_external_cpu_function_call(* external_function_id, argument_vars.as_slice());
					let mut outputs = Vec::<Value>::new();
					for output in raw_outputs.iter()
					{
						outputs.push(Value::LocalVariable(* output));
					}

					let node_result = NodeResult::MultipleOutput(outputs.into_boxed_slice());
					node_result_tracker.store_node_result(current_node_id, node_result, Some(&token));
					node_result_tracker.end_task(token);
				}
				ir::Node::CallExternalGpuCompute {external_function_id, arguments, dimensions} =>
				{
					let token = node_result_tracker.begin_task(dimensions, arguments);

					let dimension_vars = [
						force_var(node_result_tracker.get_node_output_value(dimensions[0])),
						force_var(node_result_tracker.get_node_output_value(dimensions[1])),
						force_var(node_result_tracker.get_node_output_value(dimensions[2]))
					];

					let mut argument_vars = Vec::<usize>::new();
					for argument in arguments.iter()
					{
						argument_vars.push(force_var(node_result_tracker.get_node_output_value(* argument)));
					}

					let raw_outputs = self.code_generator.build_compute_dispatch(* external_function_id, & dimension_vars, argument_vars.as_slice());
					let mut outputs = Vec::<Value>::new();
					for output in raw_outputs.iter()
					{
						outputs.push(Value::GpuBuffer(* output));
					}

					let node_result = NodeResult::MultipleOutput(outputs.into_boxed_slice());
					node_result_tracker.store_node_result(current_node_id, node_result, Some(&token));

					node_result_tracker.end_task(token);
				}
				_ => panic!("Unknown node")
			};
		}

		match & funclet.tail_edge
		{
			ir::TailEdge::Return { return_values } =>
			{
				let token = node_result_tracker.begin_task(return_values, &[]);

				assert_eq!(return_values.len(), funclet.output_types.len());
				let mut output_var_ids = Vec::<usize>::new();
				for (return_index, node_index) in return_values.iter().enumerate()
				{
					output_var_ids.push(force_var(node_result_tracker.get_node_output_value(* node_index)));
				}
				self.code_generator.build_return(output_var_ids.as_slice());

				node_result_tracker.end_task(token);
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
