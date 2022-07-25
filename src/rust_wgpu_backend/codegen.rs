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

#[derive(Debug, Clone)]
enum NodeResult
{
	None,
	// Reference to a value function that can be scheduled
	//ValueFunction { root_node : Option<ir::NodeId>, funclet_id : Option<ir::FuncletId>, node_id : Option<ir::NodeId> },
	//InlineValue { value_id : scheduling_state::ValueId, type_id : ir::TypeId },
	//InlineValue { value_id : scheduling_state::ValueId, type_id : ir::TypeId },
	//Value { value_id : scheduling_state::ValueId },
	Slot { slot_id : scheduling_state::SlotId },
	Fence { place : ir::Place, timestamp : LogicalTimestamp },
	//ValueInstance { value_instance_id : scheduling_state::ValueInstanceId }
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
	//value_tags : HashMap<scheduling_state::ValueId, ir::ValueTag>,
}

impl PlacementState
{
	fn new() -> Self
	{
		let mut place_states = HashMap::<ir::Place, PlaceState>::new();
		place_states.insert(ir::Place::Gpu, PlaceState{ .. Default::default() });
		place_states.insert(ir::Place::Local, PlaceState{ .. Default::default() });
		Self{ place_states, scheduling_state : scheduling_state::SchedulingState::new(), node_results : Default::default(), submission_map : HashMap::new(), slot_variable_ids : HashMap::new()/*, value_tags : HashMap::new()*/}
	}

	fn reset_funclet(&mut self, active_funclet_id_opt : Option<ir::FuncletId>)
	{
		self.node_results.clear();
	}

	fn update_slot_state(&mut self, slot_id : scheduling_state::SlotId, stage : ir::ResourceQueueStage, var_id : usize)
	{
		self.slot_variable_ids.insert(slot_id, var_id);
		// need to do place and stage
		self.scheduling_state.advance_queue_stage(slot_id, stage);
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

	fn get_slot_var_ids(&self, slot_ids : &[scheduling_state::SlotId], place : ir::Place) -> Option<Box<[usize]>>
	{
		let mut var_ids = Vec::<usize>::new();
		for & slot_id in slot_ids.iter()
		{
			if self.scheduling_state.get_slot_queue_place(slot_id) != place
			{
				return None;
			}
			
			var_ids.push(self.slot_variable_ids[& slot_id])
		}
		Some(var_ids.into_boxed_slice())
	}

	/*fn get_node_value_id(&self, node_id : ir::NodeId) -> Option<scheduling_state::ValueId>
	{
		match & self.node_results[& node_id]
		{
			NodeResult::InlineValue{value_id, ..} => Some(* value_id),
			_ => None
		}
	}*/

	fn get_node_slot_id(&self, node_id : ir::NodeId) -> Option<scheduling_state::SlotId>
	{
		if let NodeResult::Slot{slot_id} = & self.node_results[& node_id]
		{
			Some(* slot_id)
		}
		else
		{
			None
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
	//funclet_placement_states : HashMap<ir::FuncletId, PlacementState>,
	pending_funclet_ids : Vec<ir::FuncletId>,
	is_entry_point : bool
}

impl PipelineContext
{
	fn new() -> Self
	{
		Self { pending_funclet_ids : Default::default(), is_entry_point : true }
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
	}

	fn encode_do_node_gpu(&mut self, placement_state : &mut PlacementState, node : & ir::Node, input_slot_ids : & [scheduling_state::SlotId], output_slot_ids : & [scheduling_state::SlotId])
	{
		match node
		{
			ir::Node::CallExternalGpuCompute {external_function_id, arguments, dimensions} =>
			{
				let function = & self.program.external_gpu_functions[* external_function_id];

				assert_eq!(input_slot_ids.len(), dimensions.len() + arguments.len());
				assert_eq!(output_slot_ids.len(), function.output_types.len());

				use std::convert::TryInto;
				use std::iter::FromIterator;
				//use core::slice::SlicePattern;
				let dimension_map = |(index, x)| 
				{
					let slot_id = input_slot_ids[index];
					// Need to check that this is int
					//assert_eq!(placement_state.scheduling_state.get_slot_type_id(slot_id), function.input_types[index]);
					assert_eq!(placement_state.scheduling_state.get_slot_queue_place(slot_id), ir::Place::Local);
					assert!(placement_state.scheduling_state.get_slot_queue_stage(slot_id) >= ir::ResourceQueueStage::Encoded);
					placement_state.slot_variable_ids[& slot_id]
				};
				let argument_map = |(index, x)|
				{
					let slot_id = input_slot_ids[dimensions.len() + index];
					assert_eq!(placement_state.scheduling_state.get_slot_type_id(slot_id), function.input_types[index]);
					assert_eq!(placement_state.scheduling_state.get_slot_queue_place(slot_id), ir::Place::Gpu);
					assert!(placement_state.scheduling_state.get_slot_queue_stage(slot_id) >= ir::ResourceQueueStage::Encoded);
					placement_state.slot_variable_ids[& slot_id]
				};
				let dimension_var_ids = Vec::from_iter(dimensions.iter().enumerate().map(dimension_map)).into_boxed_slice();
				let argument_var_ids = Vec::from_iter(arguments.iter().enumerate().map(argument_map)).into_boxed_slice();

				let dimensions_slice : &[usize] = & dimension_var_ids;
				let raw_outputs = self.code_generator.build_compute_dispatch(* external_function_id, dimensions_slice.try_into().expect("Expected 3 elements for dimensions"), & argument_var_ids);

				for (index, output_type_id) in function.output_types.iter().enumerate()
				{
					let slot_id = output_slot_ids[index];
					// To do: Do something about the value
					assert_eq!(placement_state.scheduling_state.get_slot_type_id(slot_id), * output_type_id);
					assert_eq!(placement_state.scheduling_state.get_slot_queue_stage(slot_id), ir::ResourceQueueStage::None);
					assert_eq!(placement_state.scheduling_state.get_slot_queue_place(slot_id), ir::Place::Gpu);
					placement_state.update_slot_state(slot_id, ir::ResourceQueueStage::Encoded, raw_outputs[index]);
				}
			}
			_ => panic!("Node cannot be encoded to the gpu")
		}
	}

	fn encode_do_node_local(&mut self, placement_state : &mut PlacementState, node : & ir::Node, input_slot_ids : & [scheduling_state::SlotId], output_slot_ids : &[scheduling_state::SlotId])
	{
		// To do: Do something about the value
		match node
		{
			ir::Node::ConstantInteger{value, type_id} =>
			{
				assert_eq!(input_slot_ids.len(), 0);
				assert_eq!(output_slot_ids.len(), 1);

				let slot_id = output_slot_ids[0];
				let variable_id = self.code_generator.build_constant_integer(* value, * type_id);

				assert_eq!(placement_state.scheduling_state.get_slot_type_id(slot_id), * type_id);
				assert_eq!(placement_state.scheduling_state.get_slot_queue_stage(slot_id), ir::ResourceQueueStage::None);
				assert_eq!(placement_state.scheduling_state.get_slot_queue_place(slot_id), ir::Place::Local);

				placement_state.update_slot_state(slot_id, ir::ResourceQueueStage::Ready, variable_id);
			}
			ir::Node::ConstantUnsignedInteger{value, type_id} =>
			{
				assert_eq!(input_slot_ids.len(), 0);
				assert_eq!(output_slot_ids.len(), 1);

				let slot_id = output_slot_ids[0];
				let variable_id = self.code_generator.build_constant_unsigned_integer(* value, * type_id);

				assert_eq!(placement_state.scheduling_state.get_slot_type_id(slot_id), * type_id);
				assert_eq!(placement_state.scheduling_state.get_slot_queue_stage(slot_id), ir::ResourceQueueStage::None);
				assert_eq!(placement_state.scheduling_state.get_slot_queue_place(slot_id), ir::Place::Local);

				placement_state.update_slot_state(slot_id, ir::ResourceQueueStage::Ready, variable_id);
			}
			/*ir::Node::CallValueFunction { function_id, arguments } =>
			{
				panic!("Not yet implemented");
				let function = & self.program.value_functions[function_id];
				assert!(function.default_funclet_id.is_some(), "Codegen doesn't know how to handle value functions yet");
				let default_funclet_id = function.default_funclet_id.unwrap();
			}*/
			ir::Node::CallExternalCpu { external_function_id, arguments } =>
			{
				let function = & self.program.external_cpu_functions[* external_function_id];

				assert_eq!(output_slot_ids.len(), function.output_types.len());

				use std::iter::FromIterator;

				let argument_var_ids = Vec::from_iter(arguments.iter().enumerate().map(|(index, x)| { let slot_id = input_slot_ids[index]; assert_eq!(placement_state.scheduling_state.get_slot_type_id(slot_id), function.input_types[index]); assert_eq!(placement_state.scheduling_state.get_slot_queue_place(slot_id), ir::Place::Local); assert!(placement_state.scheduling_state.get_slot_queue_stage(slot_id) >= ir::ResourceQueueStage::Encoded); placement_state.slot_variable_ids[& slot_id] })).into_boxed_slice();
				let raw_outputs = self.code_generator.build_external_cpu_function_call(* external_function_id, & argument_var_ids);

				for (index, output_type_id) in function.output_types.iter().enumerate()
				{
					let slot_id = output_slot_ids[index];
					assert_eq!(placement_state.scheduling_state.get_slot_type_id(slot_id), * output_type_id);
					assert_eq!(placement_state.scheduling_state.get_slot_queue_stage(slot_id), ir::ResourceQueueStage::None);
					assert_eq!(placement_state.scheduling_state.get_slot_queue_place(slot_id), ir::Place::Local);
					placement_state.update_slot_state(slot_id, ir::ResourceQueueStage::Ready, raw_outputs[index]);
				}
			}
			_ => panic!("Cannot be scheduled local")
		}
	}

	fn compile_externally_visible_scheduling_funclet(&mut self, funclet_id : ir::FuncletId, pipeline_context : &mut PipelineContext)
	{
		let funclet = & self.program.funclets[& funclet_id];
		assert_eq!(funclet.kind, ir::FuncletKind::ScheduleExplicit);

		let argument_variable_ids = self.code_generator.begin_funclet(funclet_id, &funclet.input_types, &funclet.output_types);

		let mut placement_state = PlacementState::new();
		let mut argument_slot_ids = Vec::<scheduling_state::SlotId>::new();
		
		for (index, input_type_id) in funclet.input_types.iter().enumerate()
		{	
			let result = 
			{
				use ir::Type;
				
				match & self.program.types[input_type_id]
				{
					ir::Type::Slot { value_type, value_tag_opt, /*value_instance_id_opt, local_resource_id,*/ queue_stage, queue_place, fence_id } =>
					{
						if let Some(value_tag) = value_tag_opt
						{
							/*let value_id = if let ir::LocalMetaVariable::ValueTag(value_tag) = & funclet.local_meta_variables[value_tag_id]
							{
								// I'm too lazy to get the type of a value_tag for now
								let actual_value_type_id_opt = None;
								let value_id = placement_state.scheduling_state.insert_value(actual_value_type_id_opt);
								placement_state.value_tags.insert(value_id, value_tag.clone());

								//assert_eq!(actual_value_type_id_opt.unwrap(), value_type);
							}
							else
							{
								panic!("Not a value tag: {}", value_tag_id);
							};*/
						}
						let slot_id = placement_state.scheduling_state.insert_hacked_slot(* value_type, * queue_place, * queue_stage);
						placement_state.slot_variable_ids.insert(slot_id, argument_variable_ids[index]);
						argument_slot_ids.push(slot_id);
					}
					_ => panic!("Unimplemented")
				}
			};
		}

		let output_slot_ids = self.compile_scheduling_funclet(funclet_id, & argument_slot_ids, pipeline_context, &mut placement_state);
		let return_var_ids = placement_state.get_slot_var_ids(& output_slot_ids, ir::Place::Local).unwrap();
		self.code_generator.build_return(& return_var_ids);
		self.code_generator.end_funclet();
	}

	fn compile_scheduling_funclet(&mut self, funclet_id : ir::FuncletId, argument_slot_ids : &[scheduling_state::SlotId], pipeline_context : &mut PipelineContext, placement_state : &mut PlacementState) -> Box<[scheduling_state::SlotId]>
	{
		placement_state.reset_funclet(Some(funclet_id));
		let is_entry_point = pipeline_context.is_entry_point;
		pipeline_context.is_entry_point = false;

		// ValueTag is only meaningful locally
		// If we were to make this global, we'd need a key for disambiguation of different call instances and a way to define equivalence classes of valuetags (for example, between a phi and the node used as input for that phi)
		// Because of this, we need to recreate this for each funclet instance
		let mut slot_value_tags = HashMap::<scheduling_state::SlotId, Option<ir::ValueTag>>::new();

		let funclet = & self.program.funclets[& funclet_id];
		assert_eq!(funclet.kind, ir::FuncletKind::ScheduleExplicit);
		let funclet_scheduling_extra = & self.program.scheduling_funclet_extras[& funclet_id];
		//let scheduled_value_funclet = & self.program.value_funclets[& scheduling_funclet.value_funclet_id];

		// Ugly hack for now... in a pile of even worse hacks
		let mut argument_node_results = Vec::<NodeResult>::new();
		for (index, input_type_id) in funclet.input_types.iter().enumerate()
		{
			let is_valid = match & funclet.nodes[index]
			{
				ir::Node::None => true,
				ir::Node::Phi { .. } => true,
				_ => false
			};
			assert!(is_valid);
			
			let slot_id = argument_slot_ids[index];

			match & self.program.types[input_type_id]
			{
				ir::Type::Slot { value_type, value_tag_opt, queue_stage, queue_place, fence_id } =>
				{
					let tag = match * value_tag_opt
					{
						None => None,
						Some(ir::ValueTag::Operation{remote_node_id}) => Some(ir::ValueTag::Operation{remote_node_id}),
						Some(ir::ValueTag::ConcreteInput{funclet_id, index}) => Some(ir::ValueTag::Operation{remote_node_id : ir::RemoteNodeId{funclet_id, node_id : index}}),
						_ => panic!("Unimplemented")
					};
					slot_value_tags.insert(slot_id, tag);
				}
				_ => panic!("Unimplemented")
			}
			
			let result = NodeResult::Slot{slot_id};
			argument_node_results.push(result);
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
				ir::Node::Phi { index } =>
				{
					// Phis must appear at the start of a scheduling funclet (so that node order reflects scheduling order)
					assert_eq!(current_node_id, * index as usize);

					placement_state.node_results.insert(current_node_id, argument_node_results[* index as usize].clone());
				}
				ir::Node::ExtractResult { node_id, index } =>
				{
					// Extracts must appear directly after the call (so that node order reflects scheduling order)
					assert_eq!(current_node_id, * node_id + (* index as usize));

					match & placement_state.node_results[node_id]
					{
						_ => panic!("Funclet #{} at node #{} {:?}: Node #{} does not have multiple returns {:?}", funclet_id, current_node_id, node, node_id, placement_state)
					}
				}
				ir::Node::AllocTemporary{ place, type_id, operation } =>
				{
					assert_eq!(funclet_scheduling_extra.value_funclet_id, operation.funclet_id);

					let slot_id = placement_state.scheduling_state.insert_hacked_slot(* type_id, * place, ir::ResourceQueueStage::None);
					placement_state.node_results.insert(current_node_id, NodeResult::Slot{slot_id});

					slot_value_tags.insert(slot_id, Some(ir::ValueTag::Operation{remote_node_id : * operation}));

					// To do: Allocate from buffers for GPU/CPU and assign variable
				}
				ir::Node::EncodeDo { place, operation, inputs, outputs } =>
				{
					assert_eq!(funclet_scheduling_extra.value_funclet_id, operation.funclet_id);

					let mut input_slot_ids = Vec::<scheduling_state::SlotId>::new();
					let mut output_slot_ids = Vec::<scheduling_state::SlotId>::new();

					let encoded_funclet = & self.program.funclets[& operation.funclet_id];
					let encoded_node = & encoded_funclet.nodes[operation.node_id];

					//let mut last_value_instance_id_opt : Option<scheduling_state::ValueInstanceId> = None;

					for & input_node_id in inputs.iter()
					{
						if let Some(slot_id) = placement_state.get_node_slot_id(input_node_id)
						{
							input_slot_ids.push(slot_id);
						}
						else
						{
							panic!("Node #{} (content: {:?}) is not a slot", input_node_id, placement_state.node_results[& input_node_id]);
						}
					}

					for & output_node_id in outputs.iter()
					{
						if let Some(slot_id) = placement_state.get_node_slot_id(output_node_id)
						{
							output_slot_ids.push(slot_id);
						}
						else
						{
							panic!("Node #{} (content: {:?}) is not a slot", output_node_id, placement_state.node_results[& output_node_id]);
						}
					}

					/*for input
					{
						if let Some(value_tag) = slot_value_tags[& slot_id]
						{
							match value_tag
							{
								ir::ValueTag::Input{function_id, index} => panic!("{:?} is not a concrete value", value_tag),
								ir::ValueTag::Output{function_id, index} => panic!("{:?} is not a concrete value", value_tag),
								ir::ValueTag::Operation{remote_node_id} =>
								{
									// To do: Need a flattening of node dependencies for encoded_node
									assert_eq!(operation.funclet_id, remote_node_id.funclet_id);
									if let ir::Node::ExtractResult { node_id, index } = & encoded_funclet.nodes[remote_node_id.node_id]
									{
										assert_eq!(slot_index, * index);
										assert_eq!(operation.node_id, * node_id);
									}
								}
								ir::ValueTag::ConcreteInput{funclet_id, index} => panic!("{:?} can only appear in interface of funclet", value_tag),
								ir::ValueTag::ConcreteOutput{funclet_id, index} => panic!("{:?} can only appear in interface of funclet", value_tag),
							}
						}
					}*/

					/*match encoded_node
					{
						ir::Node::CallExternalCpu{..} =>
						{

						}
					}*/

					/*for & slot_id in input_slot_ids.iter().chain(output_slot_ids.iter())
					{
						if let Some(value_instance_id) = placement_state.scheduling_state.get_slot_value_instance_id(slot_id)
						{
							if let Some(last) = last_value_instance_id_opt
							{
								assert_eq!(last, value_instance_id);
							}

							last_value_instance_id_opt = Some(value_instance_id);
						}
					}*/


					/*if let Some(value_tag) = placement_state.scheduling_state.get_slot_value_tag(slot_id)
					{
						// To do: Check that all inputs have the same instance (if they have one)
						// To do: Check if inputs are associated to a value function
						// To do: Phis should be encodeable with the inputs as all function inputs? Or maybe need a convert operation
						//let value_tag = & placement_state.value_tags[& value_id];
						//assert_eq!(value_tag.function_id, );
						match value_tag
						{
							ir::ValueTag::Input{function_id, index} => panic!("{:?} is not a concrete value", value_tag),
							ir::ValueTag::Output{function_id, index} => panic!("{:?} is not a concrete value", value_tag),
							ir::ValueTag::Operation{remote_node_id} =>
							{
								assert_eq!(operation.funclet_id, remote_node_id.funclet_id);
								// To do: Check that node matches expected input
								return Some()
							}
						}
					}*/

					let is_tuple = match encoded_node
					{
						// Single return nodes
						ir::Node::ConstantInteger { .. } => false,
						ir::Node::ConstantUnsignedInteger { .. } => false,
						// Multiple return nodes
						ir::Node::CallExternalCpu { .. } => true,
						ir::Node::CallExternalGpuCompute { .. } => true,
						_ => panic!("Cannot encode {:?}", encoded_node)
					};

					if is_tuple
					{
						for (slot_index, slot_id) in output_slot_ids.iter().enumerate()
						{
							if let Some(value_tag) = slot_value_tags[slot_id]
							{
								match value_tag
								{
									ir::ValueTag::Input{function_id, index} => panic!("{:?} is not a concrete value", value_tag),
									ir::ValueTag::Output{function_id, index} => panic!("{:?} is not a concrete value", value_tag),
									ir::ValueTag::Operation{remote_node_id} =>
									{
										assert_eq!(operation.funclet_id, remote_node_id.funclet_id);
										if let ir::Node::ExtractResult { node_id, index } = & encoded_funclet.nodes[remote_node_id.node_id]
										{
											assert_eq!(slot_index, * index);
											assert_eq!(operation.node_id, * node_id);
										}
									}
									ir::ValueTag::ConcreteInput{funclet_id, index} => panic!("{:?} can only appear in interface of funclet", value_tag),
									ir::ValueTag::ConcreteOutput{funclet_id, index} => panic!("{:?} can only appear in interface of funclet", value_tag),
								}
							}
						}
					}
					else
					{
						assert_eq!(output_slot_ids.len(), 1);
						if let Some(value_tag) = slot_value_tags[& output_slot_ids[0]]
						{
							match value_tag
							{
								ir::ValueTag::Input{function_id, index} => panic!("{:?} is not a concrete value", value_tag),
								ir::ValueTag::Output{function_id, index} => panic!("{:?} is not a concrete value", value_tag),
								ir::ValueTag::Operation{remote_node_id} =>
								{
									assert_eq!(operation.funclet_id, remote_node_id.funclet_id);
									assert_eq!(operation.node_id, remote_node_id.node_id);
								}
								ir::ValueTag::ConcreteInput{funclet_id, index} => panic!("{:?} can only appear in interface of funclet", value_tag),
								ir::ValueTag::ConcreteOutput{funclet_id, index} => panic!("{:?} can only appear in interface of funclet", value_tag),
							}
						}
					}

					// To do: Check value compatibility
					/*for node_id in inputs.iter()
					{
						if 
						{

						}
					}*/

					/*let value_id = if let NodeResult::Value{value_id} = placement_state.node_results[value] { value_id } else { panic!("Not a value") };
					let value_tag = & placement_state.value_tags[& value_id];

					let encoded_node = match value_tag.subvalue_tag
					{
						ir::SubvalueTag::Operation{funclet_id, node_id} => & self.program.funclets[& funclet_id].nodes[node_id],
						_ => panic!("Can only encode concrete operations")
					};*/

					// To do: Lots of value compatibility checks

					match place
					{
						ir::Place::Local =>
						{
							self.encode_do_node_local(placement_state, encoded_node, input_slot_ids.as_slice(), output_slot_ids.as_slice());
						}
						ir::Place::Gpu =>
						{
							self.encode_do_node_gpu(placement_state, encoded_node, input_slot_ids.as_slice(), output_slot_ids.as_slice());
						}
						ir::Place::Cpu => (),
					}
				}
				ir::Node::EncodeCopy { place, input, output } =>
				{
					let src_slot_id = placement_state.get_node_slot_id(* input).unwrap();
					let dst_slot_id = placement_state.get_node_slot_id(* output).unwrap();

					// This is a VERY temporary assumption due to how code_generator currently works (there is no CPU place)
					assert_eq!(placement_state.scheduling_state.get_slot_queue_place(dst_slot_id), * place);

					assert_eq!(placement_state.scheduling_state.get_slot_type_id(src_slot_id), placement_state.scheduling_state.get_slot_type_id(dst_slot_id));
					assert!(placement_state.scheduling_state.get_slot_queue_stage(src_slot_id) > ir::ResourceQueueStage::None);
					assert!(placement_state.scheduling_state.get_slot_queue_stage(src_slot_id) < ir::ResourceQueueStage::Dead);
					assert_eq!(placement_state.scheduling_state.get_slot_queue_stage(dst_slot_id), ir::ResourceQueueStage::None);

					// To do: Check value compatibility

					// This is wrong, but we need to do it to work with code_generator
					match placement_state.scheduling_state.get_slot_queue_place(dst_slot_id)
					{
						ir::Place::Local =>
						{
							let var_id = self.code_generator.make_local_copy(placement_state.slot_variable_ids[& src_slot_id]).unwrap();
							placement_state.update_slot_state(dst_slot_id, ir::ResourceQueueStage::Ready, var_id);
						}
						ir::Place::Gpu =>
						{
							let var_id = self.code_generator.make_on_gpu_copy(placement_state.slot_variable_ids[& src_slot_id]).unwrap();
							placement_state.update_slot_state(dst_slot_id, ir::ResourceQueueStage::Ready, var_id);
						}
						ir::Place::Cpu => (),
					}
				}
				ir::Node::Submit { place } =>
				{
					let submission_id = placement_state.scheduling_state.insert_submission
					(
						* place,
						&mut |scheduling_state, event| ()
					);

					placement_state.submission_map.insert(submission_id, self.code_generator.flush_submission());
				}
				ir::Node::EncodeFence { place } =>
				{
					let local_timestamp = self.advance_local_time(placement_state);
					placement_state.node_results.insert(current_node_id, NodeResult::Fence { place : * place, timestamp : local_timestamp });
				}
				ir::Node::SyncFence { place : synced_place, fence } =>
				{
					let local_timestamp = self.advance_local_time(placement_state);
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
						if let Some(newer_timestamp) = self.advance_known_place_time(placement_state, fenced_place, timestamp)
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
				let encoded_value_funclet_id = funclet_scheduling_extra.value_funclet_id;
				let encoded_value_funclet = & self.program.funclets[& encoded_value_funclet_id];

				// Slots are linear (really: affine) when they cross the funclet boundary
				let mut used_slot_ids = HashSet::<scheduling_state::SlotId>::new();

				let mut output_slots = Vec::<scheduling_state::SlotId>::new();
				for (return_index, return_node_id) in return_values.iter().enumerate()
				{
					if let NodeResult::Slot{slot_id} = placement_state.node_results[return_node_id]
					{
						assert!(used_slot_ids.insert(slot_id));
						// To do: Check value compatibility
						output_slots.push(slot_id);

						let slot_value_tag = slot_value_tags[& slot_id];

						match & self.program.types[& funclet.output_types[return_index]]
						{
							ir::Type::Slot { value_type, value_tag_opt, queue_stage, queue_place, fence_id } =>
							{
								// Scheduling state checks are easy...
								assert_eq!(placement_state.scheduling_state.get_slot_queue_place(slot_id), * queue_place);
								assert_eq!(placement_state.scheduling_state.get_slot_queue_stage(slot_id), * queue_stage);
								// To do: Fence

								// Value tag checks are something else...
								match (slot_value_tag, * value_tag_opt)
								{
									(_, None) => (),
									(Some(ir::ValueTag::Operation{remote_node_id}), Some(ir::ValueTag::Operation{remote_node_id : remote_node_id_2})) =>
									{
										assert_eq!(remote_node_id, remote_node_id_2);
									}
									(Some(ir::ValueTag::Operation{remote_node_id}), Some(ir::ValueTag::ConcreteOutput{funclet_id, index})) =>
									{
										// Sanity
										assert_eq!(remote_node_id.funclet_id, funclet_id);
										assert_eq!(encoded_value_funclet.output_types[index], * value_type);

										match & encoded_value_funclet.tail_edge
										{
											ir::TailEdge::Return { return_values } => assert_eq!(return_values[index], remote_node_id.node_id),
											_ => panic!("Unimplemented")
										}
									}
									_ => panic!("Ill-formed")
								};
							}
							_ => panic!("Not a slot type")
						}
					}
					else
					{
						panic!("Output is not slot");
					}
				}

				return output_slots.into_boxed_slice();
			}
			/*ir::TailEdge::Yield { funclet_ids, captured_arguments, return_values } =>
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
			}*/
			ir::TailEdge::ScheduleCall { value_operation, /*input_slots,*/ callee_funclet_id, callee_arguments, continuation_funclet_id, continuation_arguments } =>
			{
				assert_eq!(funclet_scheduling_extra.value_funclet_id, value_operation.funclet_id);
				let encoded_value_funclet = & self.program.funclets[& value_operation.funclet_id];
				let encoded_node = & encoded_value_funclet.nodes[value_operation.node_id];
				let encoded_value_funclet_id = value_operation.funclet_id;
				match & encoded_node
				{
					ir::Node::CallValueFunction { function_id, arguments } =>
					{
						// Slots are linear (really: affine) when they cross the funclet boundary
						let mut used_slot_ids = HashSet::<scheduling_state::SlotId>::new();

						// Step 1: Check callee
						let callee_funclet = & self.program.funclets[callee_funclet_id];

						assert_eq!(callee_funclet.kind, ir::FuncletKind::ScheduleExplicit);
						let callee_funclet_scheduling_extra = & self.program.scheduling_funclet_extras[callee_funclet_id];
						
						let callee_value_funclet_id = callee_funclet_scheduling_extra.value_funclet_id;
						let callee_value_funclet = & self.program.funclets[& callee_value_funclet_id];
						assert_eq!(callee_value_funclet.kind, ir::FuncletKind::Value);

						// To do: Check that the value function is compatibile with the value funclet for the callee scheduling funclet we're calling

						// To do: Require that slots passed outside of the input binding table have no associated value
						/*assert_eq!(callee_funclet_scheduling_extra.input_binding_start, callee_arguments.len());
						let mut callee_slot_offset = 0usize;
						let mut callee_argument_index = callee_funclet_scheduling_extra.input_binding_start;
						for (argument_index, argument_node_id) in arguments.iter().enumerate()
						{
							let slot_count = funclet_scheduling_extra.per_input_slot_counts[argument_index];
							for i in 0 .. slot_count
							{
								let input_slot_node_id = input_slots[callee_slot_offset];
								if let NodeResult::Slot{slot_id} = placement_state.node_results[& input_slot_node_id]
								{
									assert!(used_slot_ids.insert(& slot_id));
									// To do: Check value compatibility
								}
								else
								{
									panic!("Input is not slot");
								}
								
								callee_slot_offset += 1;
								callee_argument_index += 1;
							}
						}
						assert_eq!(callee_argument_index, callee_funclet.input_types.len());*/

						let mut callee_input_slots = Vec::<scheduling_state::SlotId>::new();
						assert_eq!(callee_arguments.len(), callee_funclet.input_types.len());
						for (callee_argument_index, callee_argument_node_id) in callee_arguments.iter().enumerate()
						{
							if let NodeResult::Slot{slot_id} = placement_state.node_results[callee_argument_node_id]
							{
								assert!(used_slot_ids.insert(slot_id));
								callee_input_slots.push(slot_id);

								let slot_value_tag = slot_value_tags[& slot_id];

								match & self.program.types[& callee_funclet.input_types[callee_argument_index]]
								{
									ir::Type::Slot { value_type, value_tag_opt, queue_stage, queue_place, fence_id } =>
									{
										// Scheduling state checks are easy...
										assert_eq!(placement_state.scheduling_state.get_slot_queue_place(slot_id), * queue_place);
										assert_eq!(placement_state.scheduling_state.get_slot_queue_stage(slot_id), * queue_stage);
										// To do: Fence

										// Value tag checks are something else...
										match (slot_value_tag, * value_tag_opt)
										{
											(_, None) => (),
											(Some(ir::ValueTag::Operation{remote_node_id}), Some(ir::ValueTag::ConcreteInput{funclet_id, index})) =>
											{
												// Sanity
												assert_eq!(remote_node_id.funclet_id, funclet_id);
												assert_eq!(callee_value_funclet.input_types[index], * value_type);

												// All this ceremony leads up to this:
												// We need to check if the value for this slot matches the argument in this position for the specified value funclet
												assert_eq!(arguments[index], remote_node_id.node_id);
												// That's "it"
											}
											_ => panic!("Ill-formed")
										};
									}
									_ => panic!("Not a slot type")
								}
							}
							else
							{
								panic!("Input is not slot");
							}
						}

						// To do: Check type compatibility

						/*for (input_index, input_type_id) in callee_funclet.input_types.iter().enumerate()
						{
							let callee_argument_node_id = callee_arguments[input_index];
							let slot_id = if let NodeResult::Slot{slot_id} { * slot_id } else { panic!("Not a slot") };
							let slot_value_tag = slot_value_tags[& slot_id];
							match & self.program.types[input_type_id]
							{
								ir::Type::Slot { value_type, value_tag_opt, queue_stage, queue_place, fence_id } =>
								{
								}
								_ => panic!("Unimplemented")
							}
						}*/

						// Step 2: Continuation

						let continuation_funclet = & self.program.funclets[continuation_funclet_id];
						assert_eq!(continuation_funclet.kind, ir::FuncletKind::ScheduleExplicit);
						let continuation_funclet_scheduling_extra = & self.program.scheduling_funclet_extras[continuation_funclet_id];
						assert_eq!(funclet_scheduling_extra.value_funclet_id, continuation_funclet_scheduling_extra.value_funclet_id);

						let continuation_value_funclet_id = continuation_funclet_scheduling_extra.value_funclet_id;
						let continuation_value_funclet = & self.program.funclets[& continuation_value_funclet_id];
						assert_eq!(continuation_value_funclet.kind, ir::FuncletKind::Value);

						assert_eq!(encoded_value_funclet_id, continuation_value_funclet_id);

						/*// No input binding table
						assert_eq!(continuation_funclet_scheduling_extra.per_input_slot_counts.len(), 0);
						// The continuation will output instead
						assert_eq!(funclet_scheduling_extra.per_output_slot_counts.len(), 0);*/

						assert_eq!(continuation_arguments.len() + callee_funclet.output_types.len(), continuation_funclet.input_types.len());

						let mut continuation_input_slots = Vec::<scheduling_state::SlotId>::new();
						for (continuation_argument_index, continuation_argument_node_id) in continuation_arguments.iter().enumerate()
						{
							if let NodeResult::Slot{slot_id} = placement_state.node_results[continuation_argument_node_id]
							{
								assert!(used_slot_ids.insert(slot_id));

								let slot_value_tag = slot_value_tags[& slot_id];

								match & self.program.types[& continuation_funclet.input_types[continuation_argument_index]]
								{
									ir::Type::Slot { value_type, value_tag_opt, queue_stage, queue_place, fence_id } =>
									{
										// Scheduling state checks are easy...
										assert_eq!(placement_state.scheduling_state.get_slot_queue_place(slot_id), * queue_place);
										assert_eq!(placement_state.scheduling_state.get_slot_queue_stage(slot_id), * queue_stage);
										// To do: Fence

										// Value tag checks are something else...
										match (slot_value_tag, * value_tag_opt)
										{
											(_, None) => (),
											(Some(ir::ValueTag::Operation{remote_node_id}), Some(ir::ValueTag::Operation{remote_node_id : remote_node_id_2})) =>
											{
												assert_eq!(remote_node_id, remote_node_id_2);
											}
											_ => panic!("Ill-formed")
										};
									}
									_ => panic!("Not a slot type")
								}
							}
							else
							{
								panic!("Input is not slot");
							}
						}

						for (callee_output_index, callee_output_type) in callee_funclet.output_types.iter().enumerate()
						{
							let continuation_input_index = continuation_arguments.len() + callee_output_index;
							match (& self.program.types[callee_output_type], & self.program.types[& continuation_funclet.input_types[continuation_input_index]])
							{
								(ir::Type::Slot { value_type, value_tag_opt, queue_stage, queue_place, fence_id }, ir::Type::Slot { value_type : value_type_2, value_tag_opt : value_tag_opt_2, queue_stage : queue_stage_2, queue_place : queue_place_2, fence_id : fence_id_2 }) =>
								{
									// Scheduling state checks are easy...
									assert_eq!(* queue_place_2, * queue_place);
									assert_eq!(* queue_stage_2, * queue_stage);
									assert_eq!(* value_type_2, * value_type);
									// To do: Fence

									// Value tag checks are something else...
									match (* value_tag_opt, * value_tag_opt_2)
									{
										(_, None) => (),
										(Some(ir::ValueTag::ConcreteOutput{funclet_id, index : output_index}), Some(ir::ValueTag::Operation{remote_node_id})) =>
										{
											assert_eq!(remote_node_id.funclet_id, value_operation.funclet_id);
											assert_eq!(funclet_id, * callee_funclet_id);
											if let ir::Node::ExtractResult{node_id : call_node_id, index} = & self.program.funclets[& funclet_id].nodes[remote_node_id.node_id]
											{
												assert_eq!(* index, output_index);
												assert_eq!(* call_node_id, value_operation.node_id);
											}
											else
											{
												panic!("Target operation is not a result extraction");
											}
										}
										_ => panic!("Ill-formed")
									};
								}
								_ => panic!("Not a slot type")
							}
						}
						
						let mut callee_output_slots = self.compile_scheduling_funclet(* callee_funclet_id, callee_input_slots.as_slice(), pipeline_context, placement_state);
						continuation_input_slots.extend_from_slice(& callee_output_slots);
						return self.compile_scheduling_funclet(* continuation_funclet_id, continuation_input_slots.as_slice(), pipeline_context, placement_state);
					}
					_ => panic!("Node cannot be scheduled via ScheduleCall")
				}
			}
		/*ir::TailEdge::ScheduleReturn { return_values/*, output_slots*/ } =>
			{
				// Slots are linear (really: affine) when they cross the funclet boundary
				let mut used_slot_ids = HashSet::<scheduling_state::SlotId>::new();

				let mut output_slots = Vec::<scheduling_state::SlotId>::new();
				for (argument_index, argument_node_id) in return_values.iter().enumerate()
				{
					if let NodeResult::Slot{slot_id} = placement_state.node_results[argument_node_id]
					{
						assert!(used_slot_ids.insert(slot_id));
						// To do: Check value compatibility
						output_slots.push(slot_id);
					}
					else
					{
						panic!("Output is not slot");
					}
				}

				return output_slots.into_boxed_slice();
			}*/
			_ => panic!("Umimplemented")
		}

		//let old = pipeline_context.funclet_placement_states.insert(funclet_id, placement_state);
		//assert!(old.is_none());

		panic!("Should not reach here")
	}

	fn generate_pipeline(&mut self, entry_funclet_id : ir::FuncletId, pipeline_name : &str)
	{
		let entry_funclet = & self.program.funclets[& entry_funclet_id];
		assert_eq!(entry_funclet.kind, ir::FuncletKind::ScheduleExplicit);

		let mut pipeline_context = PipelineContext::new();
		pipeline_context.pending_funclet_ids.push(entry_funclet_id);

		self.code_generator.begin_pipeline(pipeline_name);

		let mut visited_funclet_ids = HashSet::<ir::FuncletId>::new();
		
		while let Some(funclet_id) = pipeline_context.pending_funclet_ids.pop()
		{
			//if ! pipeline_context.funclet_placement_states.contains_key(& funclet_id)
			if ! visited_funclet_ids.contains(& funclet_id)
			{
				self.compile_externally_visible_scheduling_funclet(funclet_id, &mut pipeline_context);

				assert!(visited_funclet_ids.insert(funclet_id));
			}
		}

		self.code_generator.emit_pipeline_entry_point(entry_funclet_id, &entry_funclet.input_types, &entry_funclet.output_types);
		
		/*match & entry_funclet.tail_edge
		{
			ir::TailEdge::Return {return_values : _} =>
			{
				self.code_generator.emit_oneshot_pipeline_entry_point(entry_funclet_id, &entry_funclet.input_types, &entry_funclet.output_types);
			}

			ir::TailEdge::Yield {funclet_ids : _, captured_arguments : _, return_values : _} => 
			{
				()
			}

			_ => panic!("Umimplemented")
		};*/
		self.code_generator.emit_oneshot_pipeline_entry_point(entry_funclet_id, &entry_funclet.input_types, &entry_funclet.output_types);

		self.code_generator.end_pipeline();
	}

	pub fn generate<'codegen>(& 'codegen mut self) -> String
	{
		for pipeline in self.program.pipelines.iter()
		{
			self.generate_pipeline(pipeline.entry_funclet, pipeline.name.as_str());
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
