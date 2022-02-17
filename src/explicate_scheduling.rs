use crate::ir;
use crate::ir_builders;

use crate::shadergen;
use crate::arena::Arena;
use std::default::Default;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BTreeSet;
use std::collections::BTreeMap;
use crate::rust_wgpu_backend::code_generator::CodeGenerator;
use std::fmt::Write;
use crate::node_usage_analysis::*;

/*
// This is a temporary hack
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum Value
{
	Retired,
	LocalVariable(usize),
	GpuBuffer(usize),
}

enum NodeResult
{
	Error,
	Retired,
	None,
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

	fn begin_dyn_scoped_node_task<'program>(&mut self, code_generator : &mut CodeGenerator<'program>, local_variable_node_ids : &[usize], gpu_buffer_node_ids : &[usize]) -> TaskToken
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
				self.end_task(active_task_token, code_generator/*, node_usage_analysis*/);
			}
		}

		return self.begin_task(code_generator, local_variable_node_ids, gpu_buffer_node_ids, None);
	}

	fn end_dyn_scoped_node_task<'program>(&mut self, token : TaskToken, code_generator : &mut CodeGenerator<'program>)
	{
		assert!(self.active_task_token.is_none());
		assert!(token.explicit_introduction_node_id.is_none());
		self.active_task_token = Some(token);
	}

	fn begin_node_task<'program>(&mut self, code_generator : &mut CodeGenerator<'program>, local_variable_node_ids : &[usize], gpu_buffer_node_ids : &[usize]) -> TaskToken
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
				return self.begin_dyn_scoped_node_task(code_generator, local_variable_node_ids, gpu_buffer_node_ids/*, node_usage_analysis*/);
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

	fn end_node_task<'program>(&mut self, token : TaskToken, code_generator : &mut CodeGenerator<'program>)
	{
		//let explicit_task_id_opt = self.node_explicit_task_ids.last().unwrap();
		match & token.explicit_introduction_node_id
		{
			None => self.end_dyn_scoped_node_task(token, code_generator/*, node_usage_analysis*/),
			Some(_) =>
			{
				assert!(token.explicit_introduction_node_id.is_some());
				assert!(! self.active_explicit_task_tokens.contains_key(& token.task_id));
				self.active_explicit_task_tokens.insert(token.task_id, token);
			}
		}
	}

	fn flush_tasks<'program>(&mut self, code_generator : &mut CodeGenerator<'program>)
	{
		let mut active_task_token_opt = None;
		std::mem::swap(&mut active_task_token_opt, &mut self.active_task_token);

		if let Some(active_task_token) = active_task_token_opt
		{
			self.end_task(active_task_token, code_generator/*, node_usage_analysis*/);
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

	fn end_task<'program>(&mut self, token : TaskToken, code_generator : &mut CodeGenerator<'program>)
	{
		// To do: Support other patterns
		assert_eq!(token.task_id.0 + 1, self.tasks.len());

		for & dependency_task_id in self.task_at(token.task_id).dependencies.iter()
		{
			assert_eq!(self.queue_for_submitted_task(dependency_task_id), Some(QueueId(0)));
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

	fn submit_gpu<'program>(&mut self, code_generator : &mut CodeGenerator<'program>, node_ids : &[usize])
	{
		for & node_id in node_ids.iter()
		{
			if let Some(_var_id) = self.make_node_on_gpu(code_generator, node_id)
			{
			}
			else
			{
				panic!("Failed to make the data gpu resident");
			}
		}
	}

	fn sync_local<'program>(&mut self, code_generator : &mut CodeGenerator<'program>, node_ids : &[usize])
	{
		for & node_id in node_ids.iter()
		{
			if let Some(_var_id) = self.make_node_local(code_generator, node_id)
			{
			}
			else
			{
				panic!("Failed to make the data local");
			}
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
}*/

#[derive(Debug)]
enum GpuResidencyState
{
	Useable,
	Encoded,
	Submitted
}

#[derive(Debug, Default)]
struct NodeResourceTracker
{
	registered_node_set : HashSet<ir::NodeId>,
	active_encoding_node_set : BTreeSet::<ir::NodeId>,
	//submitted_node_map : HashMap::<ir::NodeId, ir::NodeId>,
	node_gpu_residency_state : HashMap<ir::NodeId, GpuResidencyState>,
	locally_resident_node_set : HashSet<ir::NodeId>
}

impl NodeResourceTracker
{
	fn new() -> Self
	{
		Default::default()
	}

	fn register_local_nodes(&mut self, node_ids : &[ir::NodeId])
	{
		for & node_id in node_ids.iter()
		{
			let was_newly_registered = self.registered_node_set.insert(node_id);
			assert!(was_newly_registered);
			let was_newly_local = self.locally_resident_node_set.insert(node_id);
			assert!(was_newly_local);
		}
	}

	fn register_gpu_encoded_nodes(&mut self, node_ids : &[ir::NodeId])
	{
		for & node_id in node_ids.iter()
		{
			let was_newly_registered = self.registered_node_set.insert(node_id);
			assert!(was_newly_registered);
			let was_newly_encoded = self.active_encoding_node_set.insert(node_id);
			assert!(was_newly_encoded);
			/*let was_newly_gpu = self.gpu_resident_node_set.insert(node_id);
			assert!(was_newly_gpu);*/
			let was_newly_gpu_resident = self.node_gpu_residency_state.insert(node_id, GpuResidencyState::Encoded);
			assert!(was_newly_gpu_resident.is_none());
		}
	}

	fn transition_gpu(&mut self, node_ids : &[ir::NodeId], funclet_builder : &mut ir_builders::FuncletBuilder, min_required_state : GpuResidencyState)
	{
		let mut should_submit = false;
		let mut should_encode = false;
		let mut should_sync = false;
		match min_required_state
		{
			GpuResidencyState::Useable =>
			{
				should_submit = true;
				should_encode = true;
				should_sync = true;
			}
			GpuResidencyState::Submitted =>
			{
				should_submit = true;
				should_encode = true;
				should_sync = false;
			}
			GpuResidencyState::Encoded =>
			{
				should_submit = false;
				should_encode = true;
				should_sync = false;
			}
		}

		let mut encoded_node_depedencies = Vec::<ir::NodeId>::new();
		let mut local_node_depedencies = Vec::<ir::NodeId>::new();
		let mut submitted_node_dependencies = Vec::<ir::NodeId>::new();
		//let mut collateral_encoded_node_dependencies = Vec::<ir::NodeId>::new();
		let mut node_dependency_set = HashSet::<ir::NodeId>::new();
		let mut sync_node_dependencies = Vec::<ir::NodeId>::new();

		for & node_id in node_ids.iter()
		{
			if node_dependency_set.contains(& node_id)
			{
				continue;
			}

			node_dependency_set.insert(node_id);

			assert!(self.registered_node_set.contains(& node_id));
			let is_locally_resident = self.locally_resident_node_set.contains(& node_id);
			/*let is_gpu_resident = self.gpu_resident_node_set.contains(& node_id);
			let is_locally_resident = self.locally_resident_node_set.contains(& node_id);
			assert!(is_gpu_resident || is_locally_resident); // This will probably change eventually
			if ! is_gpu_resident
			{
				assert!(is_locally_resident);
				local_node_depedencies.push(node_id);
			}
			if self.encoding_node_set.contains(& node_id)
			{
				encoded_node_depedencies.push(node_id);
			}*/

			let gpu_residency_state = & self.node_gpu_residency_state.get(& node_id);
			match gpu_residency_state
			{
				None =>
				{
					assert!(is_locally_resident);
					local_node_depedencies.push(node_id);
					encoded_node_depedencies.push(node_id);
				}
				Some(GpuResidencyState::Useable) =>
				{
					// Nothing to do!
				}
				Some(GpuResidencyState::Encoded) =>
				{
					assert!(self.active_encoding_node_set.contains(& node_id));
					encoded_node_depedencies.push(node_id);
					sync_node_dependencies.push(node_id);
				}
				Some(GpuResidencyState::Submitted) =>
				{
					submitted_node_dependencies.push(node_id);
					sync_node_dependencies.push(node_id);
				}
			}
		}

		if should_encode && local_node_depedencies.len() > 0
		{
			for & node_id in local_node_depedencies.iter()
			{
				self.node_gpu_residency_state.insert(node_id, GpuResidencyState::Encoded);
			}

			funclet_builder.add_node(ir::Node::EncodeGpu{values : local_node_depedencies.into_boxed_slice()});
		}

		let mut has_collateral_encoded_nodes = false;
		if encoded_node_depedencies.len() > 0
		{
			has_collateral_encoded_nodes = true;
			for & node_id in self.active_encoding_node_set.iter()
			{
				if node_dependency_set.contains(& node_id)
				{
					continue;
				}

				//collateral_encoded_node_dependencies.push(node_id);
				encoded_node_depedencies.push(node_id);
			}
		}

		if should_submit && encoded_node_depedencies.len() > 0
		{
			for & node_id in encoded_node_depedencies.iter()
			{
				self.node_gpu_residency_state.insert(node_id, GpuResidencyState::Submitted);
			}

			if has_collateral_encoded_nodes
			{
				self.active_encoding_node_set.clear();
			}
			else
			{
				for & node_id in encoded_node_depedencies.iter()
				{
					self.active_encoding_node_set.remove(& node_id);
				}
			}

			funclet_builder.add_node(ir::Node::SubmitGpu{values : encoded_node_depedencies.into_boxed_slice()});
		}

		if should_sync && sync_node_dependencies.len() > 0
		{
			for & node_id in sync_node_dependencies.iter()
			{
				self.node_gpu_residency_state.insert(node_id, GpuResidencyState::Useable);
			}
			funclet_builder.add_node(ir::Node::SyncLocal{values : sync_node_dependencies.into_boxed_slice()});
		}
	}

	fn encode_gpu(&mut self, node_ids : &[ir::NodeId], funclet_builder : &mut ir_builders::FuncletBuilder)
	{
		self.transition_gpu(node_ids, funclet_builder, GpuResidencyState::Encoded);
	}

	fn submit_gpu(&mut self, node_ids : &[ir::NodeId], funclet_builder : &mut ir_builders::FuncletBuilder)
	{
		self.transition_gpu(node_ids, funclet_builder, GpuResidencyState::Submitted);
	}

	fn sync_local(&mut self, node_ids : &[ir::NodeId], funclet_builder : &mut ir_builders::FuncletBuilder)
	{
		// This isn't right yet
		self.transition_gpu(node_ids, funclet_builder, GpuResidencyState::Useable);
		for & node_id in node_ids.iter()
		{
			self.locally_resident_node_set.insert(node_id);
		}
	}
}

struct Explicator<'program>
{
	program : &'program mut ir::Program
}

fn remap_nodes(funclet_builder : & ir_builders::FuncletBuilder, node_ids : &[ir::NodeId]) -> Box<[ir::NodeId]>
{
	let mut remapped_node_ids = Vec::<ir::NodeId>::new();
	for & node_id in node_ids.iter()
	{
		remapped_node_ids.push(funclet_builder.get_remapped_node_id(node_id).unwrap());
	}
	return remapped_node_ids.into_boxed_slice();
}

impl<'program> Explicator<'program>
{
	pub fn new(program : &'program mut ir::Program) -> Self
	{
		Self {program}
	}

	pub fn run(&mut self)
	{
		for (funclet_id, funclet) in self.program.funclets.iter_mut()
		{
			*funclet = Self::explicate_funclet(funclet);
		}
	}

	fn explicate_funclet(original_funclet : & ir::Funclet) -> ir::Funclet
	{
		// funclet_id : ir::FuncletId
		{
			//let original_funclet = & self.program.funclets[& funclet_id];
			//let original_funclet = & program.funclets[& funclet_id];

			let mut funclet_builder = match original_funclet.execution_scope
			{
				Some(scope) => ir_builders::FuncletBuilder::new_with_execution_scope(scope),
				None => ir_builders::FuncletBuilder::new()
			};

			for input_type in original_funclet.input_types.iter()
			{
				funclet_builder.add_input(* input_type);
			}

			let mut node_resource_tracker = NodeResourceTracker::new();
			let node_usage_analysis = NodeUsageAnalysis::from_funclet(original_funclet);
			//let node_remapping = HashMap::<ir::NodeId, ir::NodeId>::new();

			//let encoded_node_set = HashSet::<ir::NodeId>::new();
			//let gpu_resident_node_set = HashSet::<ir::NodeId>::new();
			//let local_resident_node_set = HashSet::<ir::NodeId>::new();
	
			for (current_node_id, node) in original_funclet.nodes.iter().enumerate()
			{
				if ! node_usage_analysis.is_node_used(current_node_id)
				{
					//node_result_tracker.store_node_result(current_node_id, NodeResult::Retired, None);
					continue;
				}
	
				match node
				{
					ir::Node::Phi {index} =>
					{
						let new_node_id = funclet_builder.add_node_from_old(current_node_id, & node);
						node_resource_tracker.register_local_nodes(&[new_node_id]);
					}
					ir::Node::ExtractResult { node_id, index } =>
					{
						// This isn't right
						node_resource_tracker.sync_local(& remap_nodes(& funclet_builder, &[* node_id]), &mut funclet_builder);
						let new_node_id = funclet_builder.add_node_from_old(current_node_id, & node);
						node_resource_tracker.register_local_nodes(&[new_node_id]);
					}
					ir::Node::ConstantInteger{value, type_id} =>
					{
						let new_node_id = funclet_builder.add_node_from_old(current_node_id, & node);
						node_resource_tracker.register_local_nodes(&[new_node_id]);
					}
					ir::Node::ConstantUnsignedInteger{value, type_id} =>
					{
						let new_node_id = funclet_builder.add_node_from_old(current_node_id, & node);
						node_resource_tracker.register_local_nodes(&[new_node_id]);
					}
					ir::Node::CallExternalCpu { external_function_id, arguments } =>
					{
						node_resource_tracker.sync_local(& remap_nodes(& funclet_builder, arguments), &mut funclet_builder);
						let new_node_id = funclet_builder.add_node_from_old(current_node_id, & node);
						node_resource_tracker.register_local_nodes(&[new_node_id]);
					}
					ir::Node::CallExternalGpuCompute {external_function_id, arguments, dimensions} =>
					{
						node_resource_tracker.encode_gpu(& remap_nodes(& funclet_builder, arguments), &mut funclet_builder);
						node_resource_tracker.sync_local(& remap_nodes(& funclet_builder, dimensions), &mut funclet_builder);
						let new_node_id = funclet_builder.add_node_from_old(current_node_id, & node);
						node_resource_tracker.register_gpu_encoded_nodes(&[new_node_id]);
					}
					ir::Node::EncodeGpu{values} =>
					{
						let new_node_id = funclet_builder.add_node_from_old(current_node_id, & node);
					}
					ir::Node::SubmitGpu{values} =>
					{
						node_resource_tracker.encode_gpu(& remap_nodes(& funclet_builder, values), &mut funclet_builder);
						let new_node_id = funclet_builder.add_node_from_old(current_node_id, & node);
					}
					ir::Node::SyncLocal{values} =>
					{
						node_resource_tracker.sync_local(& remap_nodes(& funclet_builder, values), &mut funclet_builder);
						let new_node_id = funclet_builder.add_node_from_old(current_node_id, & node);
					}
					_ => panic!("Unknown node")
				};
			}
	
			match & original_funclet.tail_edge
			{
				ir::TailEdge::Return { return_values } =>
				{
					funclet_builder.set_tail_edge_from_old(& original_funclet.tail_edge)
				}
			}

			return funclet_builder.build();
		}
	}
}

// Converts the dataflow to control flow and makes the implicit scheduling explicit using a canonical interpretation when no hints are given
// Transitions from a language where scheduling is optional to one where it is required
pub fn explicate_scheduling(program : &mut ir::Program)
{
	let mut explicator = Explicator::new(program);
	explicator.run();
	//explicator.explicate_funclet();
}
