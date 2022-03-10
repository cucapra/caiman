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
use crate::node_usage_analysis::*;

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

/*// Answers the question: Given a node at the current time, where is my data and what state is it in?
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

	fn begin_dyn_scoped_node_task<'program>(&mut self, code_generator : &mut CodeGenerator<'program>, local_variable_node_ids : &[usize], gpu_buffer_node_ids : &[usize]/*, node_usage_analysis : & NodeUsageAnalysis*/) -> TaskToken
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

	fn end_dyn_scoped_node_task<'program>(&mut self, token : TaskToken, code_generator : &mut CodeGenerator<'program>/*, node_usage_analysis : & NodeUsageAnalysis*/)
	{
		assert!(self.active_task_token.is_none());
		assert!(token.explicit_introduction_node_id.is_none());
		self.active_task_token = Some(token);
	}

	/*fn begin_node_task_opening_scope<'program>(&mut self, code_generator : &mut CodeGenerator<'program>, local_variable_node_ids : &[usize], gpu_buffer_node_ids : &[usize], node_usage_analysis : & NodeUsageAnalysis, explicit_introduction_node_id : usize) -> TaskToken
	{
		let token = self.begin_task(code_generator, local_variable_node_ids, gpu_buffer_node_ids, Some(explicit_introduction_node_id));
		self.node_explicit_task_ids.push(Some(token.task_id));
		return token;
	}*/

	fn begin_node_task<'program>(&mut self, code_generator : &mut CodeGenerator<'program>, local_variable_node_ids : &[usize], gpu_buffer_node_ids : &[usize]/*, node_usage_analysis : & NodeUsageAnalysis*/) -> TaskToken
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

	fn end_node_task<'program>(&mut self, token : TaskToken, code_generator : &mut CodeGenerator<'program>/*, node_usage_analysis : & NodeUsageAnalysis*/)
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

	/*fn end_node_task_closing_scope<'program>(&mut self, token : TaskToken, code_generator : &mut CodeGenerator<'program>, node_usage_analysis : & NodeUsageAnalysis)
	{
		assert!(token.explicit_introduction_node_id.is_some());
		assert!(! self.active_explicit_task_tokens.contains_key(& token.task_id));
	}*/

	fn flush_tasks<'program>(&mut self, code_generator : &mut CodeGenerator<'program>/*, node_usage_analysis : & NodeUsageAnalysis*/)
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

	fn end_task<'program>(&mut self, token : TaskToken, code_generator : &mut CodeGenerator<'program>/*, node_usage_analysis : & NodeUsageAnalysis*/)
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

#[derive(Debug, Clone, Copy)]
enum GpuResidencyState
{
	Useable(usize),
	Encoded(usize),
	Submitted(usize)
}

// A hack
fn gpu_residency_state_with_var_replaced(state : &GpuResidencyState, variable_id : usize) -> GpuResidencyState
{
	use GpuResidencyState::*;
	match state
	{
		Useable(_) => Useable(variable_id),
		Encoded(_) => Encoded(variable_id),
		Submitted(_) => Submitted(variable_id),
	}
}

#[derive(Debug, Clone, Copy)]
enum LocalResidencyState
{
	Useable(usize),
}

fn local_residency_state_with_var_replaced(state : &LocalResidencyState, variable_id : usize) -> LocalResidencyState
{
	use LocalResidencyState::*;
	match state
	{
		Useable(_) => Useable(variable_id),
	}
}

#[derive(Debug, Default)]
struct PlacementState
{
	node_gpu_residency_states : HashMap<ir::NodeId, GpuResidencyState>,
	node_local_residency_states : HashMap<ir::NodeId, LocalResidencyState>,
	// A hack to match with the old code generator
	node_call_states : HashMap<ir::NodeId, Box<[usize]>>
}

impl PlacementState
{
	fn new() -> Self
	{
		Default::default()
	}

	fn get_local_state_var_ids(&self, node_ids : &[ir::NodeId]) -> Option<Box<[usize]>>
	{
		let mut var_ids = Vec::<usize>::new();
		for node_id in node_ids.iter()
		{
			let var_id = match self.node_local_residency_states.get(node_id)
			{
				Some(LocalResidencyState::Useable(variable_id)) => *variable_id,
				None => return None
			};
			var_ids.push(var_id);
		}
		Some(var_ids.into_boxed_slice())
	}

	fn get_gpu_state_var_ids(&self, node_ids : &[ir::NodeId]) -> Option<Box<[usize]>>
	{
		let mut var_ids = Vec::<usize>::new();
		for node_id in node_ids.iter()
		{
			let var_id = match self.node_gpu_residency_states.get(node_id)
			{
				Some(GpuResidencyState::Useable(variable_id)) => *variable_id,
				Some(GpuResidencyState::Encoded(variable_id)) => *variable_id,
				Some(GpuResidencyState::Submitted(variable_id)) => *variable_id,
				None => return None
			};
			var_ids.push(var_id);
		}
		Some(var_ids.into_boxed_slice())
	}

	/*fn encode_gpu(&mut self, node_ids : &[ir::NodeId], funclet_builder : &mut ir_builders::FuncletBuilder)
	{
		self.transition_gpu(node_ids, funclet_builder, GpuResidencyState::Encoded);
	}

	fn submit_gpu(&mut self, node_ids : &[ir::NodeId], funclet_builder : &mut ir_builders::FuncletBuilder)
	{
		self.transition_gpu(node_ids, funclet_builder, GpuResidencyState::Submitted);
	}

	fn sync_local(&mut self, node_ids : &[ir::NodeId], funclet_builder : &mut ir_builders::FuncletBuilder)
	{

	}*/
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

		//let node_usage_analysis = NodeUsageAnalysis::from_funclet(funclet);
		//let mut node_result_tracker = NodeResultTracker::new();

		// Placement state
		let mut placement_state = PlacementState::new();

		let argument_variable_ids = self.code_generator.begin_pipeline(pipeline_name, &funclet.input_types, &funclet.output_types);

		for (current_node_id, node) in funclet.nodes.iter().enumerate()
		{
			self.code_generator.insert_comment(format!(" node #{}: {:?}", current_node_id, node).as_str());

			match node
			{
				ir::Node::Phi {index} =>
				{
					placement_state.node_local_residency_states.insert(current_node_id, LocalResidencyState::Useable(argument_variable_ids[*index as usize]));
				}
				ir::Node::ExtractResult { node_id, index } =>
				{
					let node_call_state = placement_state.node_call_states.get(node_id).unwrap();

					if let Some(local_residency_state) = placement_state.node_local_residency_states.get(node_id).map(|x| *x)
					{
						placement_state.node_local_residency_states.insert(current_node_id, local_residency_state_with_var_replaced(& local_residency_state, node_call_state[* index as usize]));
					}

					if let Some(gpu_residency_state) = placement_state.node_gpu_residency_states.get(node_id).map(|x| *x)
					{
						placement_state.node_gpu_residency_states.insert(current_node_id, gpu_residency_state_with_var_replaced(& gpu_residency_state, node_call_state[* index as usize]));
					}
				}
				ir::Node::ConstantInteger{value, type_id} =>
				{
					let variable_id = self.code_generator.build_constant_integer(* value, * type_id);
					placement_state.node_local_residency_states.insert(current_node_id, LocalResidencyState::Useable(variable_id));
				}
				ir::Node::ConstantUnsignedInteger{value, type_id} =>
				{
					let variable_id = self.code_generator.build_constant_unsigned_integer(* value, * type_id);
					placement_state.node_local_residency_states.insert(current_node_id, LocalResidencyState::Useable(variable_id));
				}
				ir::Node::CallExternalCpu { external_function_id, arguments } =>
				{
					let argument_var_ids = placement_state.get_local_state_var_ids(arguments).unwrap();
					let raw_outputs = self.code_generator.build_external_cpu_function_call(* external_function_id, & argument_var_ids);
					placement_state.node_local_residency_states.insert(current_node_id, LocalResidencyState::Useable(usize::MAX));
					placement_state.node_call_states.insert(current_node_id, raw_outputs);
				}
				ir::Node::CallExternalGpuCompute {external_function_id, arguments, dimensions} =>
				{
					use std::convert::TryInto;
					//use core::slice::SlicePattern;

					let dimension_var_ids = placement_state.get_local_state_var_ids(dimensions).unwrap();
					let argument_var_ids = placement_state.get_gpu_state_var_ids(arguments).unwrap();
					let dimensions_slice : &[usize] = & dimension_var_ids;
					let raw_outputs = self.code_generator.build_compute_dispatch(* external_function_id, dimensions_slice.try_into().expect("Expected 3 elements for dimensions"), & argument_var_ids);

					placement_state.node_local_residency_states.insert(current_node_id, LocalResidencyState::Useable(usize::MAX));
					placement_state.node_call_states.insert(current_node_id, raw_outputs);
				}
				ir::Node::EncodeGpu{values} =>
				{
					for node_id in values.iter()
					{
						if let Some(LocalResidencyState::Useable(variable_id)) = placement_state.node_local_residency_states.get(node_id).map(|x| *x)
						{
							let new_variable_id = self.code_generator.make_on_gpu_copy(variable_id).unwrap();
							let old = placement_state.node_gpu_residency_states.insert(current_node_id, GpuResidencyState::Encoded(new_variable_id));
							assert!(old.is_none());
						}
						else
						{
							panic!("Encoded node is not locally resident");
						}
					}
				}
				ir::Node::SubmitGpu{values} =>
				{
					for node_id in values.iter()
					{
						if let Some(GpuResidencyState::Encoded(variable_id)) = placement_state.node_gpu_residency_states.get(node_id).map(|x| *x)
						{
							placement_state.node_gpu_residency_states.insert(current_node_id, GpuResidencyState::Encoded(variable_id));
						}
						else
						{
							panic!("Submitted node is not gpu encoded");
						}
					}
				}
				ir::Node::SyncLocal{values} =>
				{
					for node_id in values.iter()
					{
						if let Some(GpuResidencyState::Submitted(variable_id)) = placement_state.node_gpu_residency_states.get(node_id).map(|x| *x)
						{
							placement_state.node_gpu_residency_states.insert(current_node_id, GpuResidencyState::Useable(variable_id));
							let new_variable_id = self.code_generator.make_local_copy(variable_id).unwrap();
							let old = placement_state.node_local_residency_states.insert(current_node_id, LocalResidencyState::Useable(new_variable_id));
							assert!(old.is_none());
						}
						else
						{
							panic!("Locally synced node is not gpu submitted");
						}
					}
				}
				_ => panic!("Unknown node")
			};
		}

		match & funclet.tail_edge
		{
			ir::TailEdge::Return { return_values } =>
			{
				let return_var_ids = placement_state.get_local_state_var_ids(return_values).unwrap();
				self.code_generator.build_return(& return_var_ids);
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
