use crate::ir;
use crate::shadergen;
use crate::arena::Arena;
use std::default::Default;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BTreeSet;
use std::collections::BTreeMap;
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
	node_explicit_task_ids : Vec<Option<TaskId>>,
	node_gpu_buffers : HashMap<usize, usize>,
	node_local_variables : HashMap<usize, usize>,
	encoding_task_ids : BTreeSet<TaskId>,
	next_submission_id : SubmissionId,
	submission_queues : Vec<SubmissionQueue>,
	active_task_token : Option<TaskToken>,
	active_explicit_task_tokens : BTreeMap<TaskId, TaskToken>
}

// Tasks are logical units of work that represent regions where resources are in a known state
// These mostly correspond to nodes in the IR, but make explicit information that we'd otherwise want to be implicit in the IR

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug)]
struct TaskId(usize);

struct Task
{
	dependencies : BTreeSet<TaskId>,
	node_ids : Vec<usize>,
	task_submission : Option<TaskSubmission> // None if not yet submitted
}

struct TaskSubmission
{
	queue_id : QueueId,
	submission_id : SubmissionId
}

struct TaskToken
{
	explicit_introduction_node_id : Option<usize>,
	task_id : TaskId,
	local_variable_var_ids : Vec<usize>,
	gpu_buffer_var_ids : Vec<usize>,
}

// Submissions represent groups of tasks that are executing in a logical sequence
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug)]
struct SubmissionId(usize);

// A logical queue represents a logical sequence of tasks
// Tasks that in order on the same queue can be logically viewed as if they start and complete in that order
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug)]
struct QueueId(usize);

struct SubmissionQueue
{
	most_recently_synchronized_submission_id : Option<SubmissionId>
}

/*#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug)]
struct ScopeId(usize);

struct Scope
{

}*/

struct TaskAppendOptions
{
	defer_gpu_ordering_checks_to_submit : bool,
	task_internal_dependencies_only : bool,
}

impl NodeResultTracker
{
	fn new() -> Self
	{
		let mut submission_queues = Vec::<SubmissionQueue>::new();
		submission_queues.push(SubmissionQueue{most_recently_synchronized_submission_id : None});
		Self { node_results : vec![], tasks : vec![], node_task_ids : vec![], node_explicit_task_ids : vec![], node_gpu_buffers : HashMap::<usize, usize>::new(), node_local_variables : HashMap::<usize, usize>::new(), encoding_task_ids : BTreeSet::<TaskId>::new(), next_submission_id : SubmissionId(0), submission_queues, active_task_token : None, active_explicit_task_tokens : BTreeMap::<TaskId, TaskToken>::new() }
	}

	fn submission_queue_at_mut(&mut self, queue_id : QueueId) -> &mut SubmissionQueue
	{
		&mut self.submission_queues[queue_id.0]
	}

	fn submission_queue_at(& self, queue_id : QueueId) -> & SubmissionQueue
	{
		& self.submission_queues[queue_id.0]
	}

	fn task_at(& self, task_id : TaskId) -> & Task
	{
		& self.tasks[task_id.0]
	}

	fn task_at_mut(&mut self, task_id : TaskId) -> &mut Task
	{
		&mut self.tasks[task_id.0]
	}

	fn queue_for_submitted_task(& self, task_id : TaskId) -> Option<QueueId>
	{
		if let Some(task_submission) = & self.task_at(task_id).task_submission
		{
			Some(task_submission.queue_id)
		}
		else
		{
			None
		}
	}

	fn task_has_synchronized_local(& self, task_id : TaskId) -> bool
	{
		if let Some(task_submission) = & self.task_at(task_id).task_submission
		{
			assert_eq!(task_submission.queue_id, QueueId(0));
			let queue = self.submission_queue_at(task_submission.queue_id);
			if let Some(last_submission_id) = queue.most_recently_synchronized_submission_id
			{
				return last_submission_id > task_submission.submission_id;
			}
		}
		
		false
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

	fn get_node_local<'program>(&mut self, code_generator : &mut CodeGenerator<'program>, node_id : usize) -> Option<usize>
	{
		if let Some(& var_id) = self.node_local_variables.get(& node_id)
		{
			return Some(var_id);
		}

		None
	}

	fn get_node_on_gpu<'program>(&mut self, code_generator : &mut CodeGenerator<'program>, node_id : usize) -> Option<usize>
	{
		if let Some(& var_id) = self.node_gpu_buffers.get(& node_id)
		{
			return Some(var_id);
		}

		None
	}

	fn make_node_local<'program>(&mut self, code_generator : &mut CodeGenerator<'program>, node_id : usize) -> Option<usize>
	{
		if let Some(& var_id) = self.node_local_variables.get(& node_id)
		{
			return Some(var_id);
		}

		if let Some(id) = self.node_gpu_buffers.get(& node_id)
		{
			if let Some(new_id) = code_generator.make_local_copy(* id)
			{
				self.register_value(node_id, & Value::LocalVariable(new_id));
				return Some(new_id);
			}
			else
			{
				panic!("Couldn't make local copy of data");
				return None;
			}
		}

		match &mut self.node_results[node_id]
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

		None
	}

	fn make_node_on_gpu<'program>(&mut self, code_generator : &mut CodeGenerator<'program>, node_id : usize) -> Option<usize>
	{
		if let Some(& var_id) = self.node_gpu_buffers.get(& node_id)
		{
			return Some(var_id);
		}

		if let Some(id) = self.node_local_variables.get(& node_id)
		{
			if let Some(new_id) = code_generator.make_on_gpu_copy(* id)
			{
				self.register_value(node_id, & Value::GpuBuffer(new_id));
				return Some(new_id);
			}
			else
			{
				panic!("Couldn't make gpu copy of data");
				return None;
			}
		}

		match &mut self.node_results[node_id]
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

		None
	}

	fn begin_dyn_scoped_node_task<'program>(&mut self, code_generator : &mut CodeGenerator<'program>, local_variable_node_ids : &[usize], gpu_buffer_node_ids : &[usize], node_usage_analysis : & NodeUsageAnalysis) -> TaskToken
	{
		let mut active_task_token_opt = None;
		std::mem::swap(&mut active_task_token_opt, &mut self.active_task_token);

		if let Some(mut active_task_token) = active_task_token_opt
		{
			assert!(active_task_token.explicit_introduction_node_id.is_none());
			let options = TaskAppendOptions{defer_gpu_ordering_checks_to_submit : false, task_internal_dependencies_only : false};
			if self.append_task(&mut active_task_token, code_generator, local_variable_node_ids, gpu_buffer_node_ids, options)
			{
				return active_task_token;
			}
			else
			{
				self.end_task(active_task_token, code_generator, node_usage_analysis);
			}
		}

		return self.begin_task(code_generator, local_variable_node_ids, gpu_buffer_node_ids, None);
	}

	fn end_dyn_scoped_node_task<'program>(&mut self, token : TaskToken, code_generator : &mut CodeGenerator<'program>, node_usage_analysis : & NodeUsageAnalysis)
	{
		assert!(self.active_task_token.is_none());
		assert!(token.explicit_introduction_node_id.is_none());
		self.active_task_token = Some(token);
	}

	fn begin_node_task_opening_scope<'program>(&mut self, code_generator : &mut CodeGenerator<'program>, local_variable_node_ids : &[usize], gpu_buffer_node_ids : &[usize], node_usage_analysis : & NodeUsageAnalysis, explicit_introduction_node_id : usize) -> TaskToken
	{
		let token = self.begin_task(code_generator, local_variable_node_ids, gpu_buffer_node_ids, Some(explicit_introduction_node_id));
		self.node_explicit_task_ids.push(Some(token.task_id));
		return token;
	}

	fn begin_node_task<'program>(&mut self, code_generator : &mut CodeGenerator<'program>, local_variable_node_ids : &[usize], gpu_buffer_node_ids : &[usize], node_usage_analysis : & NodeUsageAnalysis) -> TaskToken
	{
		// Detect if there are active static scopes
		let scope_inference_result = self.infer_single_explicit_scope(None, local_variable_node_ids).and_then(|base| self.infer_single_explicit_scope(base, local_variable_node_ids));

		match & scope_inference_result
		{
			Err(()) => panic!("Inferred multiple explicit scopes"),
			Ok(None) =>
			{
				// Handle unscoped nodes
				self.node_explicit_task_ids.push(None);
				return self.begin_dyn_scoped_node_task(code_generator, local_variable_node_ids, gpu_buffer_node_ids, node_usage_analysis);
			}
			Ok(Some(task_id)) =>
			{
				self.node_explicit_task_ids.push(Some(* task_id));
				if let Some(mut token) = self.active_explicit_task_tokens.remove(task_id)
				{
					assert!(token.explicit_introduction_node_id.is_some());
					let options = TaskAppendOptions{defer_gpu_ordering_checks_to_submit : false, task_internal_dependencies_only : true};
					let succeeded = self.append_task(&mut token, code_generator, local_variable_node_ids, gpu_buffer_node_ids, options);
					assert!(succeeded);
					return token;
				}
				else
				{
					panic!("Token was not available")
				}
			}
		}
	}

	fn end_node_task<'program>(&mut self, token : TaskToken, code_generator : &mut CodeGenerator<'program>, node_usage_analysis : & NodeUsageAnalysis)
	{
		//let explicit_task_id_opt = self.node_explicit_task_ids.last().unwrap();
		match & token.explicit_introduction_node_id
		{
			None => self.end_dyn_scoped_node_task(token, code_generator, node_usage_analysis),
			Some(_) =>
			{
				assert!(token.explicit_introduction_node_id.is_some());
				assert!(! self.active_explicit_task_tokens.contains_key(& token.task_id));
				self.active_explicit_task_tokens.insert(token.task_id, token);
			}
		}
	}

	fn end_node_task_closing_scope<'program>(&mut self, token : TaskToken, code_generator : &mut CodeGenerator<'program>, node_usage_analysis : & NodeUsageAnalysis)
	{
		assert!(token.explicit_introduction_node_id.is_some());
		assert!(! self.active_explicit_task_tokens.contains_key(& token.task_id));
	}

	fn flush_tasks<'program>(&mut self, code_generator : &mut CodeGenerator<'program>, node_usage_analysis : & NodeUsageAnalysis)
	{
		let mut active_task_token_opt = None;
		std::mem::swap(&mut active_task_token_opt, &mut self.active_task_token);

		if let Some(active_task_token) = active_task_token_opt
		{
			self.end_task(active_task_token, code_generator, node_usage_analysis);
		}
	}

	fn begin_task<'program>(&mut self, code_generator : &mut CodeGenerator<'program>, local_variable_node_ids : &[usize], gpu_buffer_node_ids : &[usize], explicit_introduction_node_id : Option<usize>) -> TaskToken
	{
		self.check_sanity();
		let mut dependencies = BTreeSet::<TaskId>::new();

		// To do: Task tracking isn't yet correct for tasks that depend on a copy created later
		// This is something that submission nodes are supposed to solve
		// Otherwise, we get implicit and invisible dependencies between tasks
		 
		// To do: Reuse tasks still in encoding if possible.  Use commit number on queue?

		let mut local_variable_var_ids = Vec::<usize>::new();
		for node_id in local_variable_node_ids
		{
			assert!(* node_id < self.node_results.len());
			if let Some(dependency_task_id) = self.node_task_ids[* node_id]
			{
				dependencies.insert(dependency_task_id);
			}

			if let Some(var_id) = self.make_node_local(code_generator, * node_id)
			{
				local_variable_var_ids.push(var_id);
			}
			else
			{
				panic!("Failed to make the data local");
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

			if let Some(var_id) = self.make_node_on_gpu(code_generator, * node_id)
			{
				gpu_buffer_var_ids.push(var_id);
			}
			else
			{
				panic!("Failed to make the data gpu resident");
			}
		}

		let token = TaskToken{ explicit_introduction_node_id, task_id : TaskId(self.tasks.len()), local_variable_var_ids, gpu_buffer_var_ids };
		let task = Task{ dependencies, node_ids : vec![], task_submission : None };
		self.tasks.push(task);
		token
	}

	// Is there a single scope this set of dependencies can belong to?
	// With the current type system, this means that all dependencies come from the same task
	fn infer_single_explicit_scope(&self, mut previous_task_id : Option<TaskId>, node_ids : &[usize]) -> Result<Option<TaskId>, ()>
	{
		for node_id in node_ids
		{
			assert!(* node_id < self.node_results.len());
			let node_task_id = self.node_explicit_task_ids[* node_id];

			if previous_task_id.is_none()
			{
				previous_task_id = node_task_id;
			}
			else if previous_task_id != node_task_id
			{
				return Err(());
			}
		}
		Ok(previous_task_id)
	}

	// Append is not allowed to add new coordinator synchronization (gpu -> local resource transitions)
	// It's also not allowed to introduce local -> gpu ordering issues on submit
	// This means that all resource dependencies must be within the current task
	// This disallows some important patterns
	fn append_task<'program>(&mut self, token : &mut TaskToken, code_generator : &mut CodeGenerator<'program>, local_variable_node_ids : &[usize], gpu_buffer_node_ids : &[usize], options : TaskAppendOptions) -> bool
	{
		for node_id in local_variable_node_ids
		{
			assert!(* node_id < self.node_results.len());

			if self.get_node_local(code_generator, * node_id).is_none()
			{
				return false;
			}

			if let Some(dependency_task_id) = self.node_task_ids[* node_id]
			{
				if options.task_internal_dependencies_only && dependency_task_id != token.task_id
				{
					return false;
				}
			}
		}

		for node_id in gpu_buffer_node_ids
		{
			assert!(* node_id < self.node_results.len());

			if self.get_node_on_gpu(code_generator, * node_id).is_none()
			{
				return false;
			}

			if let Some(dependency_task_id) = self.node_task_ids[* node_id]
			{
				if options.task_internal_dependencies_only && dependency_task_id != token.task_id
				{
					return false;
				}

				// This is more aggressive than necessary
				// The append can happen if gpu dependencies are within the current task or on already submitted work
				// The real requirement is that the submit for this this task needs to happen after the submit for the other task
				// This might be hard to verify in append

				// This gets more complicated with more queues
				assert_eq!(self.submission_queues.len(), 1);
				let queue_id_opt = self.queue_for_submitted_task(dependency_task_id);
				let known_to_submit_after = dependency_task_id == token.task_id || queue_id_opt.is_some();
				
				if ! known_to_submit_after && ! options.defer_gpu_ordering_checks_to_submit
				{
					return false;
				}
			}
		}

		// To do: Need to be careful about cyclic task dependencies when more than one task token can be active

		let mut local_variable_var_ids = Vec::<usize>::new();
		for node_id in local_variable_node_ids
		{
			assert!(* node_id < self.node_results.len());

			let var_id = self.get_node_local(code_generator, * node_id).unwrap();
			local_variable_var_ids.push(var_id);

			if let Some(dependency_task_id) = self.node_task_ids[* node_id]
			{
				if dependency_task_id != token.task_id
				{
					self.task_at_mut(dependency_task_id).dependencies.insert(dependency_task_id);
				}
			}
		}

		let mut gpu_buffer_var_ids = Vec::<usize>::new();
		for node_id in gpu_buffer_node_ids
		{
			assert!(* node_id < self.node_results.len());

			let var_id = self.get_node_on_gpu(code_generator, * node_id).unwrap();

			gpu_buffer_var_ids.push(var_id);

			if let Some(dependency_task_id) = self.node_task_ids[* node_id]
			{
				if dependency_task_id != token.task_id
				{
					self.task_at_mut(dependency_task_id).dependencies.insert(dependency_task_id);
				}
			}
		}

		token.local_variable_var_ids = local_variable_var_ids;
		token.gpu_buffer_var_ids = gpu_buffer_var_ids;

		true
	}

	fn end_task<'program>(&mut self, token : TaskToken, code_generator : &mut CodeGenerator<'program>, node_usage_analysis : & NodeUsageAnalysis)
	{
		// To do: Support other patterns
		assert_eq!(token.task_id.0 + 1, self.tasks.len());

		for & dependency_task_id in self.task_at(token.task_id).dependencies.iter()
		{
			assert_eq!(self.queue_for_submitted_task(dependency_task_id), Some(QueueId(0)));
		}

		// The frustrations of rust not being able to partition mutable ownership at a granularity finer than a struct
		let node_ids = self.tasks[token.task_id.0].node_ids.clone();
		for & node_id in node_ids.iter()
		{
			for & (usage, usage_count) in node_usage_analysis.get_node_usages(node_id).iter()
			{
				match usage
				{
					Usage::LocalVariable =>
					{
						if let Some(_var_id) = self.make_node_local(code_generator, node_id)
						{
						}
						else
						{
							panic!("Failed to make the data local");
						}
					}
					Usage::GpuBuffer =>
					{
						if let Some(_var_id) = self.make_node_on_gpu(code_generator, node_id)
						{
						}
						else
						{
							panic!("Failed to make the data gpu resident");
						}
					}
					Usage::Extraction(_extraction_node_id) => (),
					Usage::TaskSubmission(_introduction_node_id) => (),
				}
			}
		}

		// Not doing anything sophisticated here yet
		let submission_id = self.next_submission_id;
		self.next_submission_id.0 += 1;
		{
			let task = &mut self.tasks[token.task_id.0];
			let task_submission = TaskSubmission{queue_id : QueueId(0), submission_id : submission_id};
			task.task_submission = Some(task_submission);
		}
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
			self.tasks[task_token.task_id.0].node_ids.push(node_id);
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
	TaskSubmission(usize)
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

	fn get_node_usages(& self, node_id : usize) -> Box<[(Usage, usize)]>
	{
		let mut usages = Vec::<(Usage, usize)>::new();

		if let Some(subnode_usage) = self.subnode_usages.get(& SubnodePath(node_id, None))
		{
			for (usage, usage_count) in subnode_usage.usages.iter()
			{
				usages.push((* usage, * usage_count));
			}
		}
		
		usages.into_boxed_slice()
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

				// Task nodes are a bit weird in that they use nodes, but will take them in any state
				// They just decide what state they are in afterwards
				// The forward pass gets to decide which state they are in initially
				// As such, these rules aren't really correct since they propagate the synchronization requirement to outside
				ir::Node::GpuTaskStart{local_variable_node_ids, gpu_resident_node_ids} =>
				{
					for & node_id in local_variable_node_ids.iter()
					{
						analysis.use_node(node_id, Usage::LocalVariable)
					}

					for & node_id in gpu_resident_node_ids.iter()
					{
						analysis.use_node(node_id, Usage::GpuBuffer)
					}
				}
				ir::Node::GpuTaskEnd{task_node_id, local_variable_node_ids, gpu_resident_node_ids} =>
				{
					analysis.use_node(* task_node_id, Usage::TaskSubmission(current_node_id));

					for & node_id in local_variable_node_ids.iter()
					{
						analysis.use_node(node_id, Usage::LocalVariable)
					}

					for & node_id in gpu_resident_node_ids.iter()
					{
						analysis.use_node(node_id, Usage::GpuBuffer)
					}
				}

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
					let token = node_result_tracker.begin_node_task(&mut self.code_generator, &[], &[], & node_usage_analysis);
					let node_result = NodeResult::SingleOutput(Value::LocalVariable(argument_variable_ids[*index as usize]));
					node_result_tracker.store_node_result(current_node_id, node_result, Some(&token));
					node_result_tracker.end_node_task(token, &mut self.code_generator, & node_usage_analysis);
				}
				ir::Node::ExtractResult { node_id, index } =>
				{
					let token = node_result_tracker.begin_node_task(&mut self.code_generator, &[], &[], & node_usage_analysis);
					let node_result = NodeResult::SingleOutput(node_result_tracker.get_node_output_subvalue(* node_id, * index));
					node_result_tracker.store_node_result(current_node_id, node_result, Some(&token));
					node_result_tracker.end_node_task(token, &mut self.code_generator, & node_usage_analysis);
				}
				ir::Node::ConstantInteger(value, type_id) =>
				{
					let token = node_result_tracker.begin_node_task(&mut self.code_generator, &[], &[], & node_usage_analysis);
					let variable_id = self.code_generator.build_constant_integer(* value, * type_id);
					let node_result = NodeResult::SingleOutput(Value::LocalVariable(variable_id));
					node_result_tracker.store_node_result(current_node_id, node_result, Some(&token));
					node_result_tracker.end_node_task(token, &mut self.code_generator, & node_usage_analysis);
				}
				ir::Node::ConstantUnsignedInteger(value, type_id) =>
				{
					let token = node_result_tracker.begin_node_task(&mut self.code_generator, &[], &[], & node_usage_analysis);
					let variable_id = self.code_generator.build_constant_unsigned_integer(* value, * type_id);
					let node_result = NodeResult::SingleOutput(Value::LocalVariable(variable_id));
					node_result_tracker.store_node_result(current_node_id, node_result, Some(&token));
					node_result_tracker.end_node_task(token, &mut self.code_generator, & node_usage_analysis);
				}
				ir::Node::GpuTaskStart{ local_variable_node_ids, gpu_resident_node_ids } =>
				{
					let token = node_result_tracker.begin_node_task_opening_scope(&mut self.code_generator, local_variable_node_ids, gpu_resident_node_ids, & node_usage_analysis, current_node_id);
					let mut outputs = Vec::<Value>::new();
					
					for (index, node_id) in local_variable_node_ids.iter().enumerate()
					{
						outputs.push(Value::LocalVariable(token.local_variable_var_ids[index]));
					}

					for (index, node_id) in gpu_resident_node_ids.iter().enumerate()
					{
						outputs.push(Value::GpuBuffer(token.gpu_buffer_var_ids[index]));
					}

					let node_result = NodeResult::MultipleOutput(outputs.into_boxed_slice());
					node_result_tracker.store_node_result(current_node_id, node_result, Some(&token));
					node_result_tracker.end_node_task(token, &mut self.code_generator, & node_usage_analysis);
				}
				ir::Node::GpuTaskEnd{ task_node_id, local_variable_node_ids, gpu_resident_node_ids } =>
				{
					// This doesn't work quite the way one would expect
					let token = node_result_tracker.begin_node_task(&mut self.code_generator, local_variable_node_ids, gpu_resident_node_ids, & node_usage_analysis);
					let mut outputs = Vec::<Value>::new();
					
					for (index, node_id) in local_variable_node_ids.iter().enumerate()
					{
						outputs.push(Value::LocalVariable(token.local_variable_var_ids[index]));
					}

					for (index, node_id) in gpu_resident_node_ids.iter().enumerate()
					{
						outputs.push(Value::GpuBuffer(token.gpu_buffer_var_ids[index]));
					}

					let node_result = NodeResult::MultipleOutput(outputs.into_boxed_slice());
					node_result_tracker.store_node_result(current_node_id, node_result, Some(&token));
					node_result_tracker.end_node_task_closing_scope(token, &mut self.code_generator, & node_usage_analysis);
				}
				ir::Node::CallExternalCpu { external_function_id, arguments } =>
				{
					let token = node_result_tracker.begin_node_task(&mut self.code_generator, arguments, &[], & node_usage_analysis);

					let raw_outputs = self.code_generator.build_external_cpu_function_call(* external_function_id, token.local_variable_var_ids.as_slice());
					let mut outputs = Vec::<Value>::new();
					for output in raw_outputs.iter()
					{
						outputs.push(Value::LocalVariable(* output));
					}

					let node_result = NodeResult::MultipleOutput(outputs.into_boxed_slice());
					node_result_tracker.store_node_result(current_node_id, node_result, Some(&token));
					node_result_tracker.end_node_task(token, &mut self.code_generator, & node_usage_analysis);
				}
				ir::Node::CallExternalGpuCompute {external_function_id, arguments, dimensions} =>
				{
					use std::convert::TryInto;

					let token = node_result_tracker.begin_node_task(&mut self.code_generator, dimensions, arguments, & node_usage_analysis);

					let raw_outputs = self.code_generator.build_compute_dispatch(* external_function_id, token.local_variable_var_ids.as_slice().try_into().expect("Expected 3 elements for dimensions"), token.gpu_buffer_var_ids.as_slice());
					let mut outputs = Vec::<Value>::new();
					for output in raw_outputs.iter()
					{
						outputs.push(Value::GpuBuffer(* output));
					}

					let node_result = NodeResult::MultipleOutput(outputs.into_boxed_slice());
					node_result_tracker.store_node_result(current_node_id, node_result, Some(&token));

					node_result_tracker.end_node_task(token, &mut self.code_generator, & node_usage_analysis);
				}
				_ => panic!("Unknown node")
			};
		}

		match & funclet.tail_edge
		{
			ir::TailEdge::Return { return_values } =>
			{
				let token = node_result_tracker.begin_node_task(&mut self.code_generator, return_values, &[], & node_usage_analysis);

				self.code_generator.build_return(token.local_variable_var_ids.as_slice());

				node_result_tracker.end_node_task(token, &mut self.code_generator, & node_usage_analysis);
			}
		}

		node_result_tracker.flush_tasks(&mut self.code_generator, & node_usage_analysis);

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
