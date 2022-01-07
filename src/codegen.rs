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
	Retired,
	LocalVariable(usize),
	GpuBuffer(usize),
}

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
	Retired,
	SingleOutput(Value),
	MultipleOutput(Box<[Value]>),
}

// Answers the question: Given a node at the current time, where is my data and what state is it in?
struct NodeResultTracker
{
	node_results : Vec<NodeResult>,
	tasks : Vec<Task>,
	node_task_ids : Vec<Option<TaskId>>,
	node_gpu_buffers : HashMap<usize, usize>,
	node_local_variables : HashMap<usize, usize>,
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
	local_variable_var_ids : Vec<usize>,
	gpu_buffer_var_ids : Vec<usize>
}

impl NodeResultTracker
{
	fn new() -> Self
	{
		Self { node_results : vec![], tasks : vec![], node_task_ids : vec![], node_gpu_buffers : HashMap::<usize, usize>::new(), node_local_variables : HashMap::<usize, usize>::new() }
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

	fn begin_task<'program>(&mut self, code_generator : &mut CodeGenerator<'program>, local_variable_node_ids : &[usize], gpu_buffer_node_ids : &[usize]) -> TaskToken
	{
		self.check_sanity();
		let mut dependencies = BTreeSet::<TaskId>::new();

		// To do: Task tracking isn't yet correct for tasks that depend on a copy created later
		// This is something that submission nodes are supposed to solve
		// Otherwise, we get implicit and invisible dependencies between tasks
		 
		let mut local_variable_var_ids = Vec::<usize>::new();
		for node_id in local_variable_node_ids
		{
			assert!(* node_id < self.node_results.len());
			if let Some(dependency_task_id) = self.node_task_ids[* node_id]
			{
				dependencies.insert(dependency_task_id);
			}

			if let Some(& var_id) = self.node_local_variables.get(node_id)
			{
				local_variable_var_ids.push(var_id);
				continue;
			}

			if let Some(id) = self.node_gpu_buffers.get(node_id)
			{
				if let Some(new_id) = code_generator.make_local_copy(* id)
				{
					local_variable_var_ids.push(new_id);
					self.register_value(* node_id, & Value::LocalVariable(new_id));
				}
				else
				{
					panic!("Couldn't make local copy of data");
					local_variable_var_ids.push(* id);
					self.register_value(* node_id, & Value::LocalVariable(* id));
				}
				continue;
			}

			match &mut self.node_results[* node_id]
			{
				NodeResult::SingleOutput(value) =>
				{
					match value
					{
						_ => panic!("Unexpected value {:?}", value)
					}
				}
				_ => panic!("Node isn't a single output node!")
			}
		}

		let mut gpu_buffer_var_ids = Vec::<usize>::new();
		for node_id in gpu_buffer_node_ids
		{
			assert!(* node_id < self.node_results.len());
			if let Some(dependency_task_id) = self.node_task_ids[* node_id]
			{
				dependencies.insert(dependency_task_id);
			}

			if let Some(& var_id) = self.node_gpu_buffers.get(node_id)
			{
				gpu_buffer_var_ids.push(var_id);
				continue;
			}

			if let Some(id) = self.node_local_variables.get(node_id)
			{
				if let Some(new_id) = code_generator.make_on_gpu_copy(* id)
				{
					gpu_buffer_var_ids.push(new_id);
					self.register_value(* node_id, & Value::GpuBuffer(new_id));
				}
				else
				{
					panic!("Couldn't make gpu copy of data");
					gpu_buffer_var_ids.push(* id);
					self.register_value(* node_id, & Value::GpuBuffer(* id));
				}
				continue;
			}

			match &mut self.node_results[* node_id]
			{
				NodeResult::SingleOutput(value) =>
				{
					match value
					{
						_ => panic!("Unexpected value {:?}", value)
					}
				}
				_ => panic!("Node isn't a single output node!")
			}
		}

		let token = TaskToken{ task_id : TaskId(self.tasks.len()), local_variable_var_ids, gpu_buffer_var_ids };
		let task = Task{ dependencies };
		self.tasks.push(task);
		token
	}

	fn end_task(&mut self, token : TaskToken)
	{
		// To do: Support other patterns
		assert_eq!(token.task_id.0 + 1, self.tasks.len());
	}

	fn register_value(&mut self, node_id : usize, value : & Value)
	{
		match value
		{
			Value::Retired => panic!("Should not register a retired value"),
			Value::LocalVariable(id) =>
			{
				self.node_local_variables.insert(node_id, *id);
			}
			Value::GpuBuffer(id) =>
			{
				self.node_gpu_buffers.insert(node_id, *id);
			}
			//Value::Unknown(id) => ()
		}
	}

	fn store_node_result(&mut self, node_id : usize, node_result : NodeResult, active_task : Option<&TaskToken>)
	{
		assert_eq!(node_id, self.node_results.len());
		match & node_result
		{
			NodeResult::SingleOutput(value) => self.register_value(node_id, value),
			_ => ()
		}
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

		self.check_sanity();
	}

	fn retire_node(&mut self, node_id : usize)
	{
		self.node_local_variables.remove(& node_id);
		self.node_gpu_buffers.remove(& node_id);
		self.node_results[node_id] = NodeResult::Retired;
	}

	fn check_sanity(&self)
	{
		for (node_id, node_result) in self.node_results.iter().enumerate()
		{
			match node_result
			{
				NodeResult::SingleOutput(value) =>
				{
					match value
					{
						Value::Retired =>
						{
							assert!(self.node_local_variables.get(& node_id).is_none(), "Should not have local copy of {}", node_id);
							assert!(self.node_gpu_buffers.get(& node_id).is_none(), "Should not have gpu copy of {}", node_id);
						}
						Value::LocalVariable(_) =>
						{
							assert!(self.node_local_variables.get(& node_id).is_some(), "Does not have node id {}", node_id);
						}
						Value::GpuBuffer(_) =>
						{
							assert!(self.node_gpu_buffers.get(& node_id).is_some(), "Does not have node id {}", node_id);
						}
						_ => panic!("Should not have this case")
					}
				}
				_ =>
				{
					assert!(self.node_local_variables.get(& node_id).is_none(), "Should not have local copy of {}", node_id);
					assert!(self.node_gpu_buffers.get(& node_id).is_none(), "Should not have gpu copy of {}", node_id);
				}
			}
		}
	}
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum Usage
{
	LocalVariable,
	GpuBuffer,
	Extraction(usize),
}

struct SubnodeUsage
{
	//user_node_ids : BTreeSet<usize>,
	total_usage_count : usize,
	usages : HashMap<Usage, usize>
}

#[derive(Clone, Copy, PartialOrd, Ord, Hash, PartialEq, Eq)]
struct SubnodePath(usize, Option<usize>);

// Answers the question: For a given node, how will it be used in the future?
struct NodeUsageAnalysis
{
	subnode_usages : HashMap<SubnodePath, SubnodeUsage>
}

impl NodeUsageAnalysis
{
	fn new() -> Self
	{
		Self { subnode_usages : HashMap::<SubnodePath, SubnodeUsage>::new() }
	}

	fn use_subnode_raw(&mut self, used_subnode_path : SubnodePath, usage : Usage)
	{
		if ! self.subnode_usages.contains_key(& used_subnode_path)
		{
			let node_usage = SubnodeUsage { total_usage_count : 0, usages : HashMap::<Usage, usize>::new() };
			self.subnode_usages.insert(used_subnode_path, node_usage);
		}

		let subnode_usage = self.subnode_usages.get_mut(& used_subnode_path).unwrap();
		subnode_usage.total_usage_count += 1;
		if let Some(usage_count) = subnode_usage.usages.get_mut(& usage)
		{
			* usage_count += 1;
		}
		else
		{
			subnode_usage.usages.insert(usage, 1);
		}
	}

	fn use_subnode(&mut self, used_subnode_path : SubnodePath, usage : Usage)
	{
		let SubnodePath(node_id, subnode_id_opt) = used_subnode_path;
		self.use_subnode_raw(SubnodePath(node_id, None), usage);
		if subnode_id_opt.is_some()
		{
			self.use_subnode_raw(SubnodePath(node_id, subnode_id_opt), usage);
		}
	}

	fn use_node(&mut self, node_id : usize, usage : Usage)
	{
		self.use_subnode(SubnodePath(node_id, None), usage)
	}

	fn is_subnode_used(& self, used_subnode_path : SubnodePath) -> bool
	{
		if let Some(subnode_usage) = self.subnode_usages.get(& used_subnode_path)
		{
			subnode_usage.total_usage_count > 0
		}
		else
		{
			false
		}
	}

	fn is_node_used(& self, node_id : usize) -> bool
	{
		self.is_subnode_used(SubnodePath(node_id, None))
	}

	fn from_funclet(funclet : & ir::Funclet) -> Self
	{
		let mut analysis = Self::new();
		assert_eq!(funclet.execution_scope, Some(ir::Scope::Cpu));

		match & funclet.tail_edge
		{
			ir::TailEdge::Return { return_values } =>
			{
				for & node_id in return_values.iter()
				{
					analysis.use_node(node_id, Usage::LocalVariable);
				}
			}
		}

		for (current_node_id, node) in funclet.nodes.iter().enumerate().rev()
		{
			if ! analysis.is_node_used(current_node_id)
			{
				continue;
			}
			
			match node
			{
				ir::Node::Phi {index} => (),
				ir::Node::ExtractResult { node_id, index } =>
				{
					analysis.use_node(* node_id, Usage::Extraction(current_node_id));
				}
				ir::Node::ConstantInteger(value, type_id) => (),
				ir::Node::ConstantUnsignedInteger(value, type_id) => (),
				ir::Node::CallExternalCpu { external_function_id, arguments } =>
				{
					for & node_id in arguments.iter()
					{
						analysis.use_node(node_id, Usage::LocalVariable)
					}
				}
				ir::Node::CallExternalGpuCompute {external_function_id, arguments, dimensions} =>
				{
					for & node_id in dimensions.iter()
					{
						analysis.use_node(node_id, Usage::LocalVariable)
					}

					for & node_id in arguments.iter()
					{
						analysis.use_node(node_id, Usage::GpuBuffer)
					}
				}
				_ => panic!("Unimplemented node")
			}
		}
		
		analysis
	}
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

		let node_usage_analysis = NodeUsageAnalysis::from_funclet(funclet);
		let mut node_result_tracker = NodeResultTracker::new();

		let argument_variable_ids = self.code_generator.begin_pipeline(pipeline_name, &funclet.input_types, &funclet.output_types);		

		for (current_node_id, node) in funclet.nodes.iter().enumerate()
		{
			self.code_generator.insert_comment(format!(" node #{}: {:?}", current_node_id, node).as_str());

			if ! node_usage_analysis.is_node_used(current_node_id)
			{
				self.code_generator.insert_comment(" unused");
				node_result_tracker.store_node_result(current_node_id, NodeResult::Retired, None);
				continue;
			}

			match node
			{
				ir::Node::Phi {index} =>
				{
					let node_result = NodeResult::SingleOutput(Value::LocalVariable(argument_variable_ids[*index as usize]));
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
					let token = node_result_tracker.begin_task(&mut self.code_generator, arguments, &[]);

					let raw_outputs = self.code_generator.build_external_cpu_function_call(* external_function_id, token.local_variable_var_ids.as_slice());
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
					use std::convert::TryInto;

					let token = node_result_tracker.begin_task(&mut self.code_generator, dimensions, arguments);

					let raw_outputs = self.code_generator.build_compute_dispatch(* external_function_id, token.local_variable_var_ids.as_slice().try_into().expect("Expected 3 elements for dimensions"), token.gpu_buffer_var_ids.as_slice());
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
				let token = node_result_tracker.begin_task(&mut self.code_generator, return_values, &[]);

				self.code_generator.build_return(token.local_variable_var_ids.as_slice());

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
