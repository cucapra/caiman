use crate::ir;
use crate::shadergen;
use crate::arena::Arena;
use std::default::Default;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BTreeSet;
use std::collections::BTreeMap;
use crate::rust_wgpu_backend::code_generator::CodeGenerator;
use crate::rust_wgpu_backend::code_generator::SubmissionId;
use std::fmt::Write;
use crate::node_usage_analysis::*;
use std::collections::BinaryHeap;
use std::cmp::Reverse;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct LogicalTimestamp(usize);

impl LogicalTimestamp
{
	fn new() -> Self
	{
		Self(0)
	}

	fn step(&mut self)
	{
		self.0 += 1;
	}
}

impl Default for LogicalTimestamp
{
	fn default() -> Self
	{
		Self::new()
	}
}

// This is a temporary hack
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum Value
{
	Slot,
	Fence { place : ir::Place, timestamp : LogicalTimestamp }
}

#[derive(Debug)]
enum NodeResult
{
	None,
	SingleOutput(Value),
	MultipleOutput(Box<[Value]>),
}

// Records the most recent state of a place as known to the local coordinator
#[derive(Debug, Default)]
struct PlaceState
{
	timestamp : LogicalTimestamp,
	//pending_submission_timestamps : BinaryHeap<Reverse<LogicalTimestamp>>,
	pending_submissions : BTreeMap<LogicalTimestamp, SubmissionId>,
	node_timestamps : BTreeMap<ir::NodeId, LogicalTimestamp>,
	node_queue_stages : BTreeMap<ir::NodeId, ir::ResourceQueueStage>,
	node_variable_ids : HashMap<ir::NodeId, usize>,
	// A hack to match with the old code generator
	node_call_states : HashMap<ir::NodeId, Box<[usize]>>,
	//last_submission_timestamp_opt : Option<LogicalTimestamp>
}

#[derive(Debug, Default)]
struct PlacementState
{
	//submit_node_submission_ids : HashMap<ir::NodeId, Option<SubmissionId>>,
	//pending_submission_node_ids : BinaryHeap<Reverse<ir::NodeId>>,
	//node_submission_node_ids : HashMap<ir::NodeId, ir::NodeId>,
	//last_submissions : HashMap<ir::Place, SubmissionId>,
	place_states : HashMap<ir::Place, PlaceState>, // as known to the coordinator
	node_results : HashMap<ir::NodeId, NodeResult>
}

impl PlacementState
{
	fn new() -> Self
	{
		let mut place_states = HashMap::<ir::Place, PlaceState>::new();
		place_states.insert(ir::Place::Gpu, PlaceState{ timestamp : LogicalTimestamp(0), .. Default::default() });
		place_states.insert(ir::Place::Local, PlaceState{ timestamp : LogicalTimestamp(0), .. Default::default() });
		Self{ place_states, .. Default::default()}
	}

	fn update_node_state(&mut self, node_id : ir::NodeId, place : ir::Place, stage : ir::ResourceQueueStage, var_id : usize)
	{
		let place_state : &mut PlaceState = self.place_states.get_mut(& place).unwrap();
		place_state.node_queue_stages.insert(node_id, stage);
		place_state.node_variable_ids.insert(node_id, var_id);
		place_state.node_timestamps.insert(node_id, place_state.timestamp);
	}

	fn get_var_ids(&self, node_ids : &[ir::NodeId], place : ir::Place) -> Option<Box<[usize]>>
	{
		let mut var_ids = Vec::<usize>::new();
		let place_state : & PlaceState = self.place_states.get(& place).unwrap();
		for node_id in node_ids.iter()
		{
			if let Some(& var_id) = place_state.node_variable_ids.get(node_id)
			{
				var_ids.push(var_id);
			}
			else
			{
				return None;
			}
		}
		Some(var_ids.into_boxed_slice())
	}

	fn get_local_state_var_ids(&self, node_ids : &[ir::NodeId]) -> Option<Box<[usize]>>
	{
		self.get_var_ids(node_ids, ir::Place::Local)
	}

	fn get_gpu_state_var_ids(&self, node_ids : &[ir::NodeId]) -> Option<Box<[usize]>>
	{
		self.get_var_ids(node_ids, ir::Place::Gpu)
	}
}

/*#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct PipelineStageKey
{
	funclet_id : ir::FuncletId,
	funclet_stage_id : Option<usize>,
}

// When pipeline stages surface to the outside world (the calling function) entry points don't strictly correspond to funclets (nodes), but instead correspond to the prior stage and the next funclet (paths)
// This can introduce potentially infinite cycles, so it's important that we not try to do placement inference across funclets lest we wake the sleeping halting problem
// For now we dodge this by forcing local on entry and exit, but this will have to change and it's important to be careful when it does
// Ignore the above
struct PipelineStageData
{
	placement_state_opt : Option<PlacementState>,
	//captured_argument_count : usize,
	//prior_stage_id_opt : Option<usize>
}*/

// PipelineContext tracks traversal through the funclet graph
#[derive(Default)]
struct PipelineContext
{
	//pipeline_stages : HashMap<PipelineStageKey, PipelineStageData>
	funclet_placement_states : HashMap<ir::FuncletId, PlacementState>,
	pending_funclet_ids : Vec<ir::FuncletId>
}

impl PipelineContext
{
	fn new() -> Self
	{
		Default::default()
	}
}

pub struct CodeGen<'program>
{
	program : & 'program ir::Program,
	code_generator : CodeGenerator<'program>,
	print_codegen_debug_info : bool
}

impl<'program> CodeGen<'program>
{
	pub fn new(program : & 'program ir::Program) -> Self
	{
		Self { program : & program, code_generator : CodeGenerator::new(program.types.clone(), program.external_cpu_functions.as_slice(), program.external_gpu_functions.as_slice()), print_codegen_debug_info : false }
	}

	pub fn set_print_codgen_debug_info(&mut self, to : bool)
	{
		self.print_codegen_debug_info = to;
	}

	fn advance_local_time(&mut self, placement_state : &mut PlacementState) -> LogicalTimestamp
	{
		placement_state.place_states.get_mut(& ir::Place::Local).unwrap().timestamp.step();
		let local_timestamp = placement_state.place_states.get(& ir::Place::Local).unwrap().timestamp;
		local_timestamp
	}

	fn advance_known_place_time(&mut self, placement_state : &mut PlacementState, place : ir::Place, known_timestamp : LogicalTimestamp) -> Option<LogicalTimestamp>
	{
		assert!(place != ir::Place::Local);
		let local_timestamp = placement_state.place_states[& ir::Place::Local].timestamp;
		// The local coordinator is always the latest time because all events are caused by the coordinator
		assert!(known_timestamp <= local_timestamp);

		let place_state : &mut PlaceState = placement_state.place_states.get_mut(& place).unwrap();

		// Return if we already know of this or a later time

		if place_state.timestamp >= known_timestamp
		{
			return Some(place_state.timestamp);
		}
		
		place_state.timestamp = known_timestamp;

		// Update submissions for this place
		let mut last_submission_id_opt : Option<SubmissionId> = None;
		let mut expired_timestamps = Vec::<LogicalTimestamp>::new();
		for (& timestamp, & submission_id) in place_state.pending_submissions.iter()
		{
			if timestamp <= known_timestamp
			{
				expired_timestamps.push(timestamp);
				//self.sync_submission(submission_id);
				// Relies on iteration order of a BTreeMap
				last_submission_id_opt = Some(submission_id);
			}
			else
			{
				// Also relies on iteration order
				break
			}
		}

		for & timestamp in expired_timestamps.iter()
		{
			place_state.pending_submissions.remove(& timestamp);
		}

		if let Some(submission_id) = last_submission_id_opt
		{
			self.code_generator.sync_submission(submission_id);
		}

		// Transition resource stages

		let place_state = placement_state.place_states.get_mut(& place).unwrap();
		for (node_id, stage) in place_state.node_queue_stages.iter_mut()
		{
			match stage
			{
				ir::ResourceQueueStage::Submitted if place_state.node_timestamps[node_id] <= known_timestamp =>
				{
					* stage = ir::ResourceQueueStage::Ready;
					place_state.node_timestamps.insert(* node_id, local_timestamp);
				}
				_ => ()
			}
		}

		None
	}

	/*fn transition_resource_stages(&mut self, placement_state : &mut PlacementState, place : ir::Place, from_stage : ir::ResourceQueueStage, to_stage : ir::ResourceQueueStage, timestamp : LogicalTimestamp)
	{
		let place_state = placement_state.place_states.get_mut(& place).unwrap();
		for (node_id, stage) in place_state.node_queue_stages.iter_mut()
		{
			if * stage == from_stage
			{
				* stage = to_stage;
				place_state.node_timestamps.insert(* node_id, timestamp);
			}
		}
	}*/

	fn compile_funclet(&mut self, funclet_id : ir::FuncletId, argument_variable_ids : &[usize], pipeline_context : &mut PipelineContext)
	{
		let mut placement_state = PlacementState::new();

		let funclet = & self.program.funclets[& funclet_id];
		assert_eq!(funclet.kind, ir::FuncletKind::MixedExplicit);

		if self.print_codegen_debug_info
		{
			println!("Compiling Funclet #{}...\n{:?}\n", funclet_id, funclet);
		}

		for (current_node_id, node) in funclet.nodes.iter().enumerate()
		{
			self.code_generator.insert_comment(format!(" node #{}: {:?}", current_node_id, node).as_str());

			if self.print_codegen_debug_info
			{
				println!("#{} {:?} : {:?}", current_node_id, node, placement_state);
			}

			match node
			{
				ir::Node::None => (),
				ir::Node::Phi {index} =>
				{
					placement_state.update_node_state(current_node_id, ir::Place::Local, ir::ResourceQueueStage::Ready, argument_variable_ids[*index as usize]);
				}
				ir::Node::ExtractResult { node_id, index } =>
				{
					let mut exists = false;

					for place in [ir::Place::Local, ir::Place::Gpu]
					{
						let place_state : &mut PlaceState = placement_state.place_states.get_mut(& place).unwrap();
						if let Some(node_call_state) = place_state.node_call_states.get(node_id)
						{
							exists = true;
							place_state.node_queue_stages.insert(current_node_id, place_state.node_queue_stages[node_id]);
							place_state.node_variable_ids.insert(current_node_id, node_call_state[*index as usize]);
							place_state.node_timestamps.insert(current_node_id, place_state.node_timestamps[node_id]);
						}
					}
					
					assert!(exists, "Funclet #{} at node #{} {:?}: Node #{} is not the result of a call {:?}", funclet_id, current_node_id, node, node_id, placement_state);
				}
				ir::Node::ConstantInteger{value, type_id} =>
				{
					let variable_id = self.code_generator.build_constant_integer(* value, * type_id);
					placement_state.update_node_state(current_node_id, ir::Place::Local, ir::ResourceQueueStage::Ready, variable_id);
				}
				ir::Node::ConstantUnsignedInteger{value, type_id} =>
				{
					let variable_id = self.code_generator.build_constant_unsigned_integer(* value, * type_id);
					placement_state.update_node_state(current_node_id, ir::Place::Local, ir::ResourceQueueStage::Ready, variable_id);
				}
				ir::Node::CallValueFunction { function_id, arguments } =>
				{
					panic!("Not yet implemented");
					let function = & self.program.value_functions[function_id];
					assert!(function.default_funclet_id.is_some(), "Codegen doesn't know how to handle value functions with no default binding yet");
					let default_funclet_id = function.default_funclet_id.unwrap();
					
					
				}
				ir::Node::CallExternalCpu { external_function_id, arguments } =>
				{
					let argument_var_ids_opt = placement_state.get_local_state_var_ids(arguments);
					assert!(argument_var_ids_opt.is_some(), "#{} {:?}: Not all arguments are local {:?} {:?}", current_node_id, node, arguments, placement_state);
					let argument_var_ids = argument_var_ids_opt.unwrap();
					let raw_outputs = self.code_generator.build_external_cpu_function_call(* external_function_id, & argument_var_ids);
					//placement_state.node_local_residency_states.insert(current_node_id, LocalResidencyState::Useable(usize::MAX));
					//placement_state.node_call_states.insert(current_node_id, raw_outputs);

					let place_state : &mut PlaceState = placement_state.place_states.get_mut(& ir::Place::Local).unwrap();
					place_state.node_queue_stages.insert(current_node_id, ir::ResourceQueueStage::Ready);
					place_state.node_call_states.insert(current_node_id, raw_outputs);
					place_state.node_timestamps.insert(current_node_id, place_state.timestamp);
				}
				ir::Node::CallExternalGpuCompute {external_function_id, arguments, dimensions} =>
				{
					use std::convert::TryInto;
					//use core::slice::SlicePattern;
					let dimension_var_ids_opt = placement_state.get_local_state_var_ids(dimensions);
					let argument_var_ids_opt = placement_state.get_gpu_state_var_ids(arguments);
					assert!(dimension_var_ids_opt.is_some(), "#{} {:?}: Not all dimensions are local {:?} {:?}", current_node_id, node, dimensions, placement_state);
					assert!(argument_var_ids_opt.is_some(), "#{} {:?}: Not all arguments are gpu {:?} {:?}", current_node_id, node, arguments, placement_state);

					let dimension_var_ids = dimension_var_ids_opt.unwrap();
					let argument_var_ids = argument_var_ids_opt.unwrap();
					let dimensions_slice : &[usize] = & dimension_var_ids;
					let raw_outputs = self.code_generator.build_compute_dispatch(* external_function_id, dimensions_slice.try_into().expect("Expected 3 elements for dimensions"), & argument_var_ids);

					let place_state : &mut PlaceState = placement_state.place_states.get_mut(& ir::Place::Gpu).unwrap();
					place_state.node_queue_stages.insert(current_node_id, ir::ResourceQueueStage::Encoded);
					place_state.node_call_states.insert(current_node_id, raw_outputs);
					place_state.node_timestamps.insert(current_node_id, place_state.timestamp);
				}
				ir::Node::EncodeGpu{values} =>
				{
					for node_id in values.iter()
					{
						if let Some(variable_id) = placement_state.place_states[& ir::Place::Local].node_variable_ids.get(node_id).map(|x| *x)
						{
							let new_variable_id = self.code_generator.make_on_gpu_copy(variable_id).unwrap();
							placement_state.update_node_state(* node_id, ir::Place::Gpu, ir::ResourceQueueStage::Encoded, new_variable_id);
						}
						else
						{
							panic!("Encoded node is not locally resident");
						}
					}
				}
				ir::Node::SyncLocal{values} =>
				{
					let local_timestamp = self.advance_local_time(&mut placement_state);
					let mut latest_timestamp = LogicalTimestamp(0);
					{
						let gpu_place_state = placement_state.place_states.get_mut(& ir::Place::Gpu).unwrap();
						for node_id in values.iter()
						{
							latest_timestamp = latest_timestamp.max(gpu_place_state.node_timestamps[node_id]);
						}
					}

					assert!(latest_timestamp <= local_timestamp);

					if let Some(newer_timestamp) = self.advance_known_place_time(&mut placement_state, ir::Place::Gpu, latest_timestamp)
					{
						assert!(newer_timestamp <= local_timestamp);
						// nothing
					}

					for node_id in values.iter()
					{
						let new_variable_id = 
						{
							let gpu_place_state = placement_state.place_states.get_mut(& ir::Place::Gpu).unwrap();
							self.code_generator.make_local_copy(gpu_place_state.node_variable_ids[node_id]).unwrap()
						};

						placement_state.update_node_state(* node_id, ir::Place::Local, ir::ResourceQueueStage::Ready, new_variable_id);
					}


					/*let mut latest_submission_node_id_opt = None;
					for node_id in values.iter()
					{
						let submission_node_id = *placement_state.node_submission_node_ids.get(node_id).unwrap();
						while placement_state.pending_submission_node_ids.len() > 0
						{
							if let Some(Reverse(pending_submission_node_id)) = placement_state.pending_submission_node_ids.peek().map(|x| *x)
							{
								if pending_submission_node_id < submission_node_id
								{
									placement_state.pending_submission_node_ids.pop();
								}
								else if pending_submission_node_id == submission_node_id
								{
									placement_state.pending_submission_node_ids.pop();
									latest_submission_node_id_opt = Some(pending_submission_node_id);
									break
								}
							}
							else
							{
								break
							}
						}
					}
					
					if let Some(submission_node_id) = latest_submission_node_id_opt
					{
						if let Some(submission_id) = placement_state.submit_node_submission_ids[& submission_node_id]
						{
							self.code_generator.sync_submission(submission_id);
						}
						placement_state.submit_node_submission_ids.insert(submission_node_id, None);
					}

					for node_id in values.iter()
					{
						if let Some(GpuResidencyState::Submitted(variable_id, _)) = placement_state.node_gpu_residency_states.get(node_id).map(|x| *x)
						{
							// This is a wart with how this code is designed...
							// It should eventually get cleaned up once the scheduling language implementationm is reworked
							assert!(variable_id != usize::MAX, "Cannot synchronize directly on a gpu call because there is no value.  This is a wart resulting from the old scheduling language being value-centric.  This will get fixed.");
							placement_state.update_node_state(* node_id, ir::Place::Gpu, ir::ResourceQueueStage::Ready, variable_id);
							let new_variable_id = self.code_generator.make_local_copy(variable_id).unwrap();
							placement_state.update_node_state(* node_id, ir::Place::Local, ir::ResourceQueueStage::Ready, new_variable_id);
							assert!(old.is_none());
						}
						else
						{
							panic!("Locally synced node is not gpu submitted");
						}
					}*/
				}

				// New scheduling nodes
				ir::Node::Submit { place } =>
				{
					let local_timestamp = self.advance_local_time(&mut placement_state);
					match place
					{
						ir::Place::Gpu =>
						{
							let place_state = placement_state.place_states.get_mut(place).unwrap();
							for (node_id, stage) in place_state.node_queue_stages.iter_mut()
							{
								match stage
								{
									ir::ResourceQueueStage::Encoded =>
									{
										* stage = ir::ResourceQueueStage::Submitted;
										place_state.node_timestamps.insert(* node_id, local_timestamp);
									}
									_ => ()
								}
							}

							let submission_id = self.code_generator.flush_submission();
							place_state.pending_submissions.insert(local_timestamp, submission_id);
						}
						_ => panic!("Unimplemented")
					}
				}
				ir::Node::EncodeFence { place } =>
				{
					let local_timestamp = self.advance_local_time(&mut placement_state);
					let fence_value = Value::Fence { place : * place, timestamp : local_timestamp };
					placement_state.node_results.insert(current_node_id, NodeResult::SingleOutput(fence_value));
				}
				ir::Node::SyncFence { place : synced_place, fence } =>
				{
					let local_timestamp = self.advance_local_time(&mut placement_state);
					// Only implemented for the local queue for now
					assert_eq!(* synced_place, ir::Place::Local);
					// To do: Need to update nodes
					let value_opt = match placement_state.node_results.get(fence)
					{
						Some(NodeResult::SingleOutput(Value::Fence{place, timestamp})) =>
						{
							Some(Value::Fence{place : * place, timestamp : * timestamp})
						}
						_ => panic!("Expected fence")
					};

					if let Some(Value::Fence{place : fenced_place, timestamp}) = value_opt
					{
						assert_eq!(fenced_place, ir::Place::Gpu);
						if let Some(newer_timestamp) = self.advance_known_place_time(&mut placement_state, fenced_place, timestamp)
						{
							panic!("Have already synced to a later time")
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
			ir::TailEdge::Yield { funclet_ids, captured_arguments, return_values } =>
			{
				let captured_argument_var_ids = placement_state.get_local_state_var_ids(captured_arguments).unwrap();
				let return_var_ids = placement_state.get_local_state_var_ids(return_values).unwrap();

				let mut next_funclet_input_types = Vec::<Box<[ir::TypeId]>>::new();
				for & next_funclet_id in funclet_ids.iter()
				{
					pipeline_context.pending_funclet_ids.push(next_funclet_id);
					/*if ! pipeline_context.funclet_placement_states.contains_key(& funclet_id)
					{
					}*/
					let input_types = self.program.funclets[& next_funclet_id].input_types.to_vec();
					//let input_types = Vec::<ir::TypeId>::new();
					next_funclet_input_types.push(input_types.into_boxed_slice());
				}
				// Proper codegen is a lot more complicated than this
				// self.code_generator.build_yield(& captured_argument_var_ids, & return_var_ids);
				// This is disgusting
				self.code_generator.build_yield(funclet_ids, next_funclet_input_types.into_boxed_slice(), & captured_argument_var_ids, & return_var_ids);
			}
		}

		let old = pipeline_context.funclet_placement_states.insert(funclet_id, placement_state);
		assert!(old.is_none());
	}

	/*fn generate_pipeline_stage(&mut self, pipeline_context : &mut PipelineContext, parent_stage_id_opt : Option<usize>) -> usize
	{

	}*/

	fn generate_cpu_function(&mut self, entry_funclet_id : ir::FuncletId, pipeline_name : &str)
	{
		let entry_funclet = & self.program.funclets[& entry_funclet_id];
		assert_eq!(entry_funclet.kind, ir::FuncletKind::MixedExplicit);

		let mut pipeline_context = PipelineContext::new();
		pipeline_context.pending_funclet_ids.push(entry_funclet_id);

		self.code_generator.begin_pipeline(pipeline_name);

		while let Some(funclet_id) = pipeline_context.pending_funclet_ids.pop()
		{
			if ! pipeline_context.funclet_placement_states.contains_key(& funclet_id)
			{
				let funclet = & self.program.funclets[& funclet_id];
				assert_eq!(funclet.kind, ir::FuncletKind::MixedExplicit);

				let argument_variable_ids = self.code_generator.begin_funclet(funclet_id, &funclet.input_types, &funclet.output_types);
				self.compile_funclet(funclet_id, & argument_variable_ids, &mut pipeline_context);
				self.code_generator.end_funclet();
			}
		}

		/*match & entry_funclet.tail_edge
		{
			ir::TailEdge::Return {return_values : _} =>
			{
				let argument_variable_ids = self.code_generator.begin_oneshot_entry_funclet(&entry_funclet.input_types, &entry_funclet.output_types);
				self.compile_funclet(entry_funclet_id, & argument_variable_ids, &mut pipeline_context);
				self.code_generator.end_funclet();
			}

			ir::TailEdge::Yield {funclet_ids : _, captured_arguments : _, return_values : _} => 
			{
				()
			}
			//self.code_generator.begin_corecursive_base_funclet(pipeline_name, &entry_funclet.input_types, &entry_funclet.output_types),
		};*/

		self.code_generator.emit_pipeline_entry_point(entry_funclet_id, &entry_funclet.input_types, &entry_funclet.output_types);
		
		match & entry_funclet.tail_edge
		{
			ir::TailEdge::Return {return_values : _} =>
			{
				self.code_generator.emit_oneshot_pipeline_entry_point(entry_funclet_id, &entry_funclet.input_types, &entry_funclet.output_types);
			}

			ir::TailEdge::Yield {funclet_ids : _, captured_arguments : _, return_values : _} => 
			{
				()
			}
		};

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
	use super::*;
	use crate::ir;
}
