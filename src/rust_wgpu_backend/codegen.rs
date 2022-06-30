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
use std::collections::BinaryHeap;
use std::cmp::Reverse;

use crate::scheduling_state;
use crate::scheduling_state::{LogicalTimestamp};

#[derive(Debug)]
enum NodeResult
{
	None,
	// Reference to a value function that can be scheduled
	//ValueFunction { root_node : Option<ir::NodeId>, funclet_id : Option<ir::FuncletId>, node_id : Option<ir::NodeId> },
	InlineValue { value_id : scheduling_state::ValueId, type_id : ir::TypeId },
	//InlineValue { value_id : scheduling_state::ValueId, type_id : ir::TypeId },
	RootValueInstance { funclet_id : ir::FuncletId, node_id : ir::NodeId },
	Slot { slot_id : scheduling_state::SlotId },
	Fence { place : ir::Place, timestamp : LogicalTimestamp },
	ExternalGpuCall{ output_slots : Box<[scheduling_state::SlotId]> },
	ExternalCpuCall{ output_slots : Box<[scheduling_state::SlotId]> }
}

// Records the most recent state of a place as known to the local coordinator
#[derive(Debug, Default)]
struct PlaceState
{
	//node_variable_ids : HashMap<ir::NodeId, usize>,
}

#[derive(Debug)]
struct PlacementState
{
	place_states : HashMap<ir::Place, PlaceState>, // as known to the coordinator
	node_results : HashMap<ir::NodeId, NodeResult>,
	scheduling_state : scheduling_state::SchedulingState,
	submission_map : HashMap<scheduling_state::SubmissionId, SubmissionId>,
	slot_variable_ids : HashMap<scheduling_state::SlotId, usize>,
	value_funclet_node_id_pairs : HashMap<scheduling_state::ValueId, (ir::FuncletId, ir::NodeId)>
}

impl PlacementState
{
	fn new() -> Self
	{
		let mut place_states = HashMap::<ir::Place, PlaceState>::new();
		place_states.insert(ir::Place::Gpu, PlaceState{ .. Default::default() });
		place_states.insert(ir::Place::Local, PlaceState{ .. Default::default() });
		Self{ place_states, scheduling_state : scheduling_state::SchedulingState::new(), node_results : Default::default(), submission_map : HashMap::new(), slot_variable_ids : HashMap::new(), value_funclet_node_id_pairs : HashMap::new()}
	}

	fn update_slot_state(&mut self, slot_id : scheduling_state::SlotId, place : ir::Place, stage : ir::ResourceQueueStage, var_id : usize)
	{
		self.slot_variable_ids.insert(slot_id, var_id);
		// need to do place and stage
	}

	/*fn update_node_state(&mut self, node_id : ir::NodeId, place : ir::Place, stage : ir::ResourceQueueStage, var_id : usize)
	{
		let place_state : &mut PlaceState = self.place_states.get_mut(& place).unwrap();
		//place_state.node_queue_stages.insert(node_id, stage);

		place_state.node_variable_ids.insert(node_id, var_id);
		//place_state.node_timestamps.insert(node_id, place_state.timestamp);
		//self.slot_variable_ids.insert(node, );
	}*/

	fn get_var_ids(&self, node_ids : &[ir::NodeId], place : ir::Place) -> Option<Box<[usize]>>
	{
		let mut var_ids = Vec::<usize>::new();
		/*let place_state : & PlaceState = self.place_states.get(& place).unwrap();
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
		}*/
		for node_id in node_ids.iter()
		{
			match self.node_results[node_id]
			{
				NodeResult::Slot{slot_id} =>
				{
					if self.scheduling_state.get_slot_queue_place(slot_id) != place
					{
						return None;
					}
					
					var_ids.push(self.slot_variable_ids[& slot_id])
				}
				_ => return None
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

	fn get_node_value_id(&self, node_id : ir::NodeId) -> Option<scheduling_state::ValueId>
	{
		match & self.node_results[& node_id]
		{
			NodeResult::InlineValue{value_id, ..} => Some(* value_id),
			_ => None
		}
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
		/*placement_state.place_states.get_mut(& ir::Place::Local).unwrap().timestamp.step();
		let local_timestamp = placement_state.place_states.get(& ir::Place::Local).unwrap().timestamp;
		local_timestamp*/
		placement_state.scheduling_state.advance_local_time()
	}

	fn advance_known_place_time(&mut self, placement_state : &mut PlacementState, place : ir::Place, known_timestamp : LogicalTimestamp) -> Option<LogicalTimestamp>
	{
		use scheduling_state::SchedulingEvent;

		let mut submission_ids = Vec::<scheduling_state::SubmissionId>::new();

		let time_opt = placement_state.scheduling_state.advance_known_place_time
		(
			place, known_timestamp,
			&mut |scheduling_state, event|
			match event
			{
				SchedulingEvent::SyncSubmission{ submission_id } => { submission_ids.push(* submission_id); }
			}
		);

		for submission_id in submission_ids
		{
			self.code_generator.sync_submission(placement_state.submission_map[& submission_id])
		}

		return time_opt;
		/*assert!(place != ir::Place::Local);
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

		None*/
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

	fn encode_do_node_gpu(&mut self, placement_state : &mut PlacementState, node : & ir::Node, input_slot_ids : &[scheduling_state::SlotId], output_slot_ids : &[scheduling_state::SlotId])
	{
		match node
		{
			ir::Node::CallExternalGpuCompute {external_function_id, arguments, dimensions} =>
			{
				let function = & self.program.external_gpu_functions[* external_function_id];

				assert_eq!(input_slot_ids.len(), arguments.len());
				assert_eq!(output_slot_ids.len(), function.output_types.len());

				use std::convert::TryInto;
				//use core::slice::SlicePattern;
				let dimension_var_ids_opt = placement_state.get_local_state_var_ids(dimensions);
				let argument_var_ids_opt = placement_state.get_gpu_state_var_ids(arguments);
				assert!(dimension_var_ids_opt.is_some(), "{:?}: Not all dimensions are local {:?} {:?}", node, dimensions, placement_state);
				assert!(argument_var_ids_opt.is_some(), "{:?}: Not all arguments are gpu {:?} {:?}", node, arguments, placement_state);

				let dimension_var_ids = dimension_var_ids_opt.unwrap();
				let argument_var_ids = argument_var_ids_opt.unwrap();
				let dimensions_slice : &[usize] = & dimension_var_ids;
				let raw_outputs = self.code_generator.build_compute_dispatch(* external_function_id, dimensions_slice.try_into().expect("Expected 3 elements for dimensions"), & argument_var_ids);

				for (index, output_type_id) in function.output_types.iter().enumerate()
				{
					let slot_id = output_slot_ids[index];
					placement_state.update_slot_state(slot_id, ir::Place::Gpu, ir::ResourceQueueStage::Encoded, raw_outputs[index]);
				}
			}
			/*{
				use std::convert::TryInto;
				//use core::slice::SlicePattern;
				let dimension_var_ids_opt = placement_state.get_local_state_var_ids(dimensions);
				let argument_var_ids_opt = placement_state.get_gpu_state_var_ids(arguments);
				assert!(dimension_var_ids_opt.is_some(), "#{} {:?}: Not all dimensions are local {:?} {:?}", current_node_id, node, dimensions, placement_state);
				assert!(argument_var_ids_opt.is_some(), "#{} {:?}: Not all arguments are gpu {:?} {:?}", current_node_id, node, arguments, placement_state);

				let dimension_var_ids = dimension_var_ids_opt.unwrap();
				let argument_var_ids = argument_var_ids_opt.unwrap();
				let dimensions_slice : &[usize] = & dimension_var_ids;

				let raw_outputs = 

				self.code_generator.build_compute_dispatch_with_outputs(* external_function_id, dimensions_slice.try_into().expect("Expected 3 elements for dimensions"), & argument_var_ids);

				let place_state : &mut PlaceState = placement_state.place_states.get_mut(& ir::Place::Gpu).unwrap();
				place_state.node_queue_stages.insert(current_node_id, ir::ResourceQueueStage::Encoded);
				place_state.node_timestamps.insert(current_node_id, place_state.timestamp);

				placement_state.node_results.insert(current_node_id, NodeResult::ExternalGpuCall{outputs : raw_outputs.into_boxed_slice()});
			}*/
			_ => panic!("Node cannot be encoded to the gpu")
		}
	}

	fn encode_do_node_local(&mut self, placement_state : &mut PlacementState, node : & ir::Node, input_slot_ids : &[scheduling_state::SlotId], output_slot_ids : &[scheduling_state::SlotId])
	{
		match node
		{
			ir::Node::ConstantInteger{value, type_id} =>
			{
				assert_eq!(input_slot_ids.len(), 0);
				assert_eq!(output_slot_ids.len(), 1);

				let slot_id = output_slot_ids[0];
				let variable_id = self.code_generator.build_constant_integer(* value, * type_id);
				placement_state.update_slot_state(slot_id, ir::Place::Local, ir::ResourceQueueStage::Ready, variable_id);
			}
			ir::Node::ConstantUnsignedInteger{value, type_id} =>
			{
				assert_eq!(input_slot_ids.len(), 0);
				assert_eq!(output_slot_ids.len(), 1);

				let slot_id = output_slot_ids[0];
				let variable_id = self.code_generator.build_constant_unsigned_integer(* value, * type_id);
				placement_state.update_slot_state(slot_id, ir::Place::Local, ir::ResourceQueueStage::Ready, variable_id);
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
				let function = & self.program.external_cpu_functions[* external_function_id];

				assert_eq!(input_slot_ids.len(), arguments.len());
				assert_eq!(output_slot_ids.len(), function.output_types.len());

				let argument_var_ids_opt = placement_state.get_local_state_var_ids(arguments);
				assert!(argument_var_ids_opt.is_some(), "{:?}: Not all arguments are local {:?} {:?}", node, arguments, placement_state);
				let argument_var_ids = argument_var_ids_opt.unwrap();
				let raw_outputs = self.code_generator.build_external_cpu_function_call(* external_function_id, & argument_var_ids);

				for (index, output_type_id) in function.output_types.iter().enumerate()
				{
					let slot_id = output_slot_ids[index];
					placement_state.update_slot_state(slot_id, ir::Place::Local, ir::ResourceQueueStage::Ready, raw_outputs[index]);
				}
			}
			_ => panic!("Cannot be scheduled local")
		}
	}

	fn compile_funclet(&mut self, funclet_id : ir::FuncletId, argument_variable_ids : &[usize], pipeline_context : &mut PipelineContext)
	{
		let mut placement_state = PlacementState::new();

		let funclet = & self.program.funclets[& funclet_id];
		assert_eq!(funclet.kind, ir::FuncletKind::ScheduleExplicit);

		let mut argument_slot_ids = Vec::<scheduling_state::SlotId>::new();
		let mut argument_value_ids = Vec::<scheduling_state::ValueId>::new();
		
		for (index, input_type_id) in funclet.input_types.iter().enumerate()
		{
			let slot_id = placement_state.scheduling_state.insert_hacked_slot(* input_type_id, ir::Place::Local, ir::ResourceQueueStage::Ready);
			argument_slot_ids.push(slot_id);
			placement_state.slot_variable_ids.insert(slot_id, argument_variable_ids[index]);

			let value_id = placement_state.scheduling_state.insert_value(None, &[]);
			argument_value_ids.push(value_id);

			placement_state.scheduling_state.bind_slot_value(slot_id, value_id);
		}

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
					let slot_id = argument_slot_ids[* index as usize];
					let value_id = argument_value_ids[* index as usize];
					placement_state.node_results.insert(current_node_id, NodeResult::Slot{slot_id});
					placement_state.value_funclet_node_id_pairs.insert(value_id, (funclet_id, current_node_id));
				}
				ir::Node::ExtractResult { node_id, index } =>
				{
					match & placement_state.node_results[node_id]
					{
						NodeResult::ExternalCpuCall{output_slots} =>
						{
							let place_state : &mut PlaceState = placement_state.place_states.get_mut(& ir::Place::Local).unwrap();
							placement_state.node_results.insert(current_node_id, NodeResult::Slot{slot_id : output_slots[*index as usize]});
						}
						NodeResult::ExternalGpuCall{output_slots} =>
						{
							let place_state : &mut PlaceState = placement_state.place_states.get_mut(& ir::Place::Gpu).unwrap();
							placement_state.node_results.insert(current_node_id, NodeResult::Slot{slot_id : output_slots[*index as usize]});
						}
						_ => panic!("Funclet #{} at node #{} {:?}: Node #{} is not the result of a call {:?}", funclet_id, current_node_id, node, node_id, placement_state)
					}
				}
				ir::Node::ConstantInteger { type_id, .. } =>
				{
					let value_id = placement_state.scheduling_state.insert_value(Some(* type_id), &[]);
					placement_state.node_results.insert(current_node_id, NodeResult::InlineValue{value_id, type_id : * type_id});
				}
				ir::Node::ConstantUnsignedInteger { type_id, .. } =>
				{
					let value_id = placement_state.scheduling_state.insert_value(Some(* type_id), &[]);
					placement_state.node_results.insert(current_node_id, NodeResult::InlineValue{value_id, type_id : * type_id});
				}
				//ir::Node::CallValueFunction { .. } => (),
				ir::Node::CallExternalCpu { external_function_id, arguments } =>
				{
					let function = & self.program.external_cpu_functions[* external_function_id];
					let mut dependencies = Vec::<scheduling_state::ValueId>::new();

					for & argument_node_id in arguments.iter()
					{
						dependencies.push(placement_state.get_node_value_id(argument_node_id).unwrap());
					}

					let function_value_id = placement_state.scheduling_state.insert_value(None, dependencies.as_slice());
					placement_state.value_funclet_node_id_pairs.insert(function_value_id, (funclet_id, current_node_id));

					let mut output_value_ids = Vec::<scheduling_state::ValueId>::new();
					for & output_type_id in function.output_types.iter()
					{
						let value_id = placement_state.scheduling_state.insert_value(Some(output_type_id), &[function_value_id]);
					}
					
					//placement_state.node_results.insert(current_node_id, NodeResult::InlineValue{value_id, }); ?
				}
				/*ir::Node::ConstantInteger{value, type_id} =>
				{
					let variable_id = self.code_generator.build_constant_integer(* value, * type_id);
					let place_state : &mut PlaceState = placement_state.place_states.get_mut(& ir::Place::Gpu).unwrap();
					let slot_id = placement_state.scheduling_state.insert_hacked_slot(* type_id, ir::Place::Local, ir::ResourceQueueStage::Ready);
					placement_state.slot_variable_ids.insert(slot_id, variable_id);
					placement_state.node_results.insert(current_node_id, NodeResult::Slot{slot_id});
				}
				ir::Node::ConstantUnsignedInteger{value, type_id} =>
				{
					let variable_id = self.code_generator.build_constant_unsigned_integer(* value, * type_id);
					let place_state : &mut PlaceState = placement_state.place_states.get_mut(& ir::Place::Gpu).unwrap();
					let slot_id = placement_state.scheduling_state.insert_hacked_slot(* type_id, ir::Place::Local, ir::ResourceQueueStage::Ready);
					placement_state.slot_variable_ids.insert(slot_id, variable_id);
					placement_state.node_results.insert(current_node_id, NodeResult::Slot{slot_id});
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

					let place_state : &mut PlaceState = placement_state.place_states.get_mut(& ir::Place::Local).unwrap();

					let mut output_slots = Vec::<scheduling_state::SlotId>::new();
					for (index, output_type_id) in self.program.funclets[external_function_id].output_types.iter().enumerate()
					{
						let slot_id = placement_state.scheduling_state.insert_hacked_slot(* output_type_id, ir::Place::Local, ir::ResourceQueueStage::Encoded);
						placement_state.slot_variable_ids.insert(slot_id, raw_outputs[index]);
						output_slots.push(slot_id);
					}

					placement_state.node_results.insert(current_node_id, NodeResult::ExternalCpuCall{output_slots : output_slots.into_boxed_slice()});
				}*/
				ir::Node::CallExternalGpuCompute { .. } => (),
				/*{
					placement_state.node_results.insert(current_node_id, NodeResult::InlineValue);
				}*/
				/*{
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
					//place_state.node_queue_stages.insert(current_node_id, ir::ResourceQueueStage::Encoded);
					//place_state.node_timestamps.insert(current_node_id, place_state.timestamp);

					let mut output_slots = Vec::<scheduling_state::SlotId>::new();
					for (index, output_type_id) in self.program.funclets[external_function_id].output_types.iter().enumerate()
					{
						let slot_id = placement_state.scheduling_state.insert_hacked_slot(* output_type_id, ir::Place::Local, ir::ResourceQueueStage::Ready);
						placement_state.slot_variable_ids.insert(slot_id, raw_outputs[index]);
						output_slots.push(slot_id);
					}

					placement_state.node_results.insert(current_node_id, NodeResult::ExternalGpuCall{output_slots : output_slots.into_boxed_slice()});
				}*/

				// 
				/*ir::Node::EncodeGpu{values} =>
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
					for node_id in values.iter()
					{
						let new_variable_id = 
						{
							let slot_id = match & placement_state.node_results[node_id]
							{
								NodeResult::Slot{slot_id} => * slot_id,
								_ => panic!("Node is not a slot")
							};
							let gpu_place_state = placement_state.place_states.get_mut(& ir::Place::Gpu).unwrap();
							assert_eq!(placement_state.scheduling_state.get_slot_queue_stage(slot_id), ir::ResourceQueueStage::Ready);
							self.code_generator.make_local_copy(gpu_place_state.node_variable_ids[node_id]).unwrap()
						};

						placement_state.update_node_state(* node_id, ir::Place::Local, ir::ResourceQueueStage::Ready, new_variable_id);
					}
				}*/

				ir::Node::AllocTemporary{place, value} =>
				{
					let type_id = 0;
					let slot_id = placement_state.scheduling_state.insert_hacked_slot(type_id, * place, ir::ResourceQueueStage::None);
					//let slot_id = placement_state.scheduling_state.insert_hacked_slot(type_id, * place);
					placement_state.node_results.insert(current_node_id, NodeResult::Slot{slot_id});

					// To do: Allocate buffers for GPU/CPU

					/*match place
					{
						ir::Place::Local =>
						{
							placement_state.update_node_state(* node_id, ir::Place::Local, ir::ResourceQueueStage::Ready, new_variable_id);
						}
					}*/
				}

				// Syncs to earliest legal point that is available
				/*ir::Node::SyncEarliest{to_place, from_place, nodes} =>
				{
					// Only implemented for the local queue for now
					assert_eq!(* to_place, ir::Place::Local);

					// This requires sophistication, but it's sophistication the backend will have for a while?

					let local_timestamp = self.advance_local_time(&mut placement_state);
					let mut latest_timestamp = LogicalTimestamp::new();
					{
						let gpu_place_state = placement_state.place_states.get_mut(from_place).unwrap();
						for node_id in nodes.iter()
						{
							latest_timestamp = latest_timestamp.max(gpu_place_state.node_timestamps[node_id]);
						}
					}

					assert!(latest_timestamp <= local_timestamp);

					if let Some(newer_timestamp) = self.advance_known_place_time(&mut placement_state, * from_place, latest_timestamp)
					{
						assert!(newer_timestamp <= local_timestamp);
						// nothing
					}
				}*/
				// New scheduling nodes
				ir::Node::EncodeDo { place, value, inputs, outputs } =>
				{
					let mut input_slots = Vec::<scheduling_state::SlotId>::new();
					let mut output_slots = Vec::<scheduling_state::SlotId>::new();
					for i in 0 .. 2
					{

					}
					//let slot_id = placement_state.scheduling_state.insert_hacked_slot(* type_id, ir::Place::Local, ir::ResourceQueueStage::Ready);
					//placement_state.slot_variable_ids.insert(slot_id, variable_id);
					//placement_state.node_results.insert(current_node_id, NodeResult::Slot{slot_id});
					match place
					{
						ir::Place::Local =>
						{
							self.encode_do_node_local(&mut placement_state, node, input_slots.as_slice(), output_slots.as_slice());
						}
						ir::Place::Gpu =>
						{
							self.encode_do_node_gpu(&mut placement_state, node, input_slots.as_slice(), output_slots.as_slice());
						}
						ir::Place::Cpu => (),
					}
				}
				ir::Node::Submit { place } =>
				{
					/*let local_timestamp = self.advance_local_time(&mut placement_state);
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
					}*/
					let submission_id = placement_state.scheduling_state.insert_submission
					(
						* place,
						&mut |scheduling_state, event| ()
						/*match event
						{

						}*/
					);

					placement_state.submission_map.insert(submission_id, self.code_generator.flush_submission());
				}
				ir::Node::EncodeFence { place } =>
				{
					let local_timestamp = self.advance_local_time(&mut placement_state);
					placement_state.node_results.insert(current_node_id, NodeResult::Fence { place : * place, timestamp : local_timestamp });
				}
				ir::Node::SyncFence { place : synced_place, fence } =>
				{
					let local_timestamp = self.advance_local_time(&mut placement_state);
					// Only implemented for the local queue for now
					assert_eq!(* synced_place, ir::Place::Local);
					// To do: Need to update nodes
					let value_opt = match placement_state.node_results.get(fence)
					{
						Some(NodeResult::Fence{place, timestamp}) =>
						{
							Some(NodeResult::Fence{place : * place, timestamp : * timestamp})
						}
						_ => panic!("Expected fence")
					};

					if let Some(NodeResult::Fence{place : fenced_place, timestamp}) = value_opt
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
