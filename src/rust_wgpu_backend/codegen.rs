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
struct Join
{
	funclet_id : ir::FuncletId,
	captures : Box<[scheduling_state::SlotId]>,
	type_id_opt : Option<ir::TypeId>,
	variable_id_opt : Option<usize>
}

#[derive(Debug, Clone)]
enum NodeResult
{
	None,
	Slot { slot_id : scheduling_state::SlotId },
	Fence { place : ir::Place, timestamp : LogicalTimestamp },
	Join(Join)
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
	scheduling_state : scheduling_state::SchedulingState,
	submission_map : HashMap<scheduling_state::SubmissionId, SubmissionId>,
	slot_variable_ids : HashMap<scheduling_state::SlotId, usize>,
}

impl PlacementState
{
	fn new() -> Self
	{
		let mut place_states = HashMap::<ir::Place, PlaceState>::new();
		place_states.insert(ir::Place::Gpu, PlaceState{ .. Default::default() });
		place_states.insert(ir::Place::Local, PlaceState{ .. Default::default() });
		Self{ place_states, scheduling_state : scheduling_state::SchedulingState::new(), /*node_results : Default::default(),*/ submission_map : HashMap::new(), slot_variable_ids : HashMap::new()/*, value_tags : HashMap::new()*/}
	}

	fn update_slot_state(&mut self, slot_id : scheduling_state::SlotId, stage : ir::ResourceQueueStage, var_id : usize)
	{
		self.slot_variable_ids.insert(slot_id, var_id);
		// need to do place and stage
		self.scheduling_state.advance_queue_stage(slot_id, stage);
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

	fn get_slot_var_id(&self, slot_id : scheduling_state::SlotId) -> Option<usize>
	{
		self.slot_variable_ids.get(& slot_id).map(|x| * x)
	}
}

#[derive(Debug)]
struct FuncletScopedState
{
	value_funclet_id : ir::FuncletId,
	scheduling_funclet_id : ir::FuncletId,
	node_results : HashMap<ir::NodeId, NodeResult>,
	// ValueTag is only meaningful locally
	// If we were to make this global, we'd need a key for disambiguation of different call instances and a way to define equivalence classes of valuetags (for example, between a phi and the node used as input for that phi)
	// Because of this, we need to recreate this for each funclet instance
	slot_value_tags : HashMap<scheduling_state::SlotId, ir::ValueTag>,
}

impl FuncletScopedState
{
	fn new(value_funclet_id : ir::FuncletId, scheduling_funclet_id : ir::FuncletId) -> Self
	{
		Self{ value_funclet_id, scheduling_funclet_id, node_results : Default::default(), slot_value_tags : HashMap::new()}
	}

	fn get_node_slot_id(&self, node_id : ir::NodeId) -> Option<scheduling_state::SlotId>
	{
		match & self.node_results[& node_id]
		{
			NodeResult::Slot{slot_id} => Some(* slot_id),
			_ => None
		}
	}

	fn move_node_slot_id(&mut self, node_id : ir::NodeId) -> Option<scheduling_state::SlotId>
	{
		let slot_id_opt = match & self.node_results[& node_id]
		{
			NodeResult::Slot{slot_id} => Some(* slot_id),
			_ => None
		};

		if let Some(slot_id) = slot_id_opt
		{
			* self.node_results.get_mut(& node_id).unwrap() = NodeResult::None;
		}

		slot_id_opt
	}

	fn get_node_join(&self, node_id : ir::NodeId) -> Option<&Join>
	{
		match & self.node_results[& node_id]
		{
			NodeResult::Join(join) => Some(& join),
			_ => None
		}
	}

	fn move_node_join(&mut self, node_id : ir::NodeId) -> Option<Join>
	{
		let node_result_opt = self.node_results.remove(& node_id);

		if let Some(node_result) = node_result_opt
		{
			if let NodeResult::Join(join) = node_result
			{
				self.node_results.insert(node_id, NodeResult::None);
				return Some(join)
			}
			else
			{
				self.node_results.insert(node_id, node_result);
				return None
			}
		}
		
		return None
	}
}

// Check value tag in inner (source) scope transfering to outer (destination) scope
fn check_value_tag_compatibility_exit(program : & ir::Program, source_value_tag : ir::ValueTag, destination_value_tag : ir::ValueTag)
{
	match (source_value_tag, destination_value_tag)
	{
		(_, ir::ValueTag::None) => (),
		(ir::ValueTag::Operation{remote_node_id}, ir::ValueTag::Output{funclet_id, index}) =>
		{
			assert_eq!(remote_node_id.funclet_id, funclet_id);

			let source_value_funclet = & program.funclets[& funclet_id];
			assert_eq!(source_value_funclet.kind, ir::FuncletKind::Value);

			match & source_value_funclet.tail_edge
			{
				ir::TailEdge::Return { return_values } => assert_eq!(return_values[index], remote_node_id.node_id),
				_ => panic!("Not a unit")
			}
		}
		_ => panic!("Ill-formed: {:?} to {:?}", source_value_tag, destination_value_tag)
	}
}

// Check value tag in outer (source) scope transfering to inner (destination) scope
/*fn check_value_tag_compatibility_enter(program : & ir::Program, source_value_tag : ir::ValueTag, destination_value_tag : ir::ValueTag, value_operation : ir::RemoteNodeId)
{
	match (source_value_tag, destination_value_tag)
	{
		(_, ir::ValueTag::None) => (),
		(ir::ValueTag::Operation{remote_node_id}, ir::ValueTag::ConcreteInput{funclet_id, index}) =>
		{

		}
		_ => panic!("Ill-formed: {:?} to {:?}", source_value_tag, destination_value_tag)
	}
}*/

// Check value tag transition in same scope
fn check_value_tag_compatibility_interior(program : & ir::Program, source_value_tag : ir::ValueTag, destination_value_tag : ir::ValueTag)
{
	match (source_value_tag, destination_value_tag)
	{
		(ir::ValueTag::Halt{index}, ir::ValueTag::Halt{index : index_2}) => assert_eq!(index, index_2),
		(ir::ValueTag::Halt{..}, _) => panic!("Halt can only match halt"),
		(_, ir::ValueTag::None) => (),
		(ir::ValueTag::Input{funclet_id, index}, ir::ValueTag::Operation{remote_node_id}) =>
		{
			assert_eq!(remote_node_id.funclet_id, funclet_id);

			let destination_value_funclet = & program.funclets[& funclet_id];
			assert_eq!(destination_value_funclet.kind, ir::FuncletKind::Value);

			if let ir::Node::Phi{index : phi_index} = & destination_value_funclet.nodes[remote_node_id.node_id]
			{
				assert_eq!(* phi_index, index);
			}
			else
			{
				panic!("Not a phi");
			}
		}
		(ir::ValueTag::Operation{remote_node_id}, ir::ValueTag::Operation{remote_node_id : remote_node_id_2}) =>
		{
			assert_eq!(remote_node_id, remote_node_id_2);
		}
		(ir::ValueTag::Operation{remote_node_id}, ir::ValueTag::Output{funclet_id, index}) =>
		{
			assert_eq!(remote_node_id.funclet_id, funclet_id);

			let source_value_funclet = & program.funclets[& funclet_id];
			assert_eq!(source_value_funclet.kind, ir::FuncletKind::Value);

			match & source_value_funclet.tail_edge
			{
				ir::TailEdge::Return { return_values } => assert_eq!(return_values[index], remote_node_id.node_id),
				_ => panic!("Not a unit")
			}
		}
		_ => panic!("Ill-formed: {:?} to {:?}", source_value_tag, destination_value_tag)
	}
}

fn check_slot_type(program : & ir::Program, type_id : ir::TypeId, queue_place : ir::Place, queue_stage : ir::ResourceQueueStage, value_type_opt : Option<ir::TypeId>)
{
	match & program.types[& type_id]
	{
		ir::Type::Slot { value_type : value_type_2, queue_stage : queue_stage_2, queue_place : queue_place_2 } =>
		{
			assert_eq!(* queue_place_2, queue_place);
			assert_eq!(* queue_stage_2, queue_stage);
			if let Some(value_type) = value_type_opt
			{
				assert_eq!(value_type, * value_type_2);
			}
			// To do: Fence
		}
		_ => panic!("Not a slot type")
	}
}

#[derive(Default)]
struct PipelineContext
{
	pending_funclet_ids : Vec<ir::FuncletId>,
}

impl PipelineContext
{
	fn new() -> Self
	{
		Self { pending_funclet_ids : Default::default() }
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
			ir::Node::Select { condition, true_case, false_case } =>
			{
				assert_eq!(input_slot_ids.len(), 3);
				assert_eq!(output_slot_ids.len(), 1);

				let input_var_ids = input_slot_ids.iter().map(|& slot_id| placement_state.get_slot_var_id(slot_id).unwrap()).collect::<Box<[usize]>>();

				let slot_id = output_slot_ids[0];
				let variable_id = self.code_generator.build_select_hack(input_var_ids[0], input_var_ids[1], input_var_ids[2]);

				//assert_eq!(placement_state.scheduling_state.get_slot_type_id(slot_id), * type_id);
				assert_eq!(placement_state.scheduling_state.get_slot_queue_stage(slot_id), ir::ResourceQueueStage::None);
				assert_eq!(placement_state.scheduling_state.get_slot_queue_place(slot_id), ir::Place::Local);

				placement_state.update_slot_state(slot_id, ir::ResourceQueueStage::Ready, variable_id);
			}
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
					ir::Type::Slot { value_type, queue_stage, queue_place } =>
					{
						//if let Some(value_tag) = value_tag_opt
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
		// Temporary hack while I get join points working
		if output_slot_ids.len() > 0
		{
			let return_var_ids = placement_state.get_slot_var_ids(& output_slot_ids, ir::Place::Local).unwrap();
			self.code_generator.build_return(& return_var_ids);
		}
		self.code_generator.end_funclet();
	}

	fn compile_scheduling_funclet(&mut self, funclet_id : ir::FuncletId, argument_slot_ids : &[scheduling_state::SlotId], pipeline_context : &mut PipelineContext, placement_state : &mut PlacementState) -> Box<[scheduling_state::SlotId]>
	{
		let funclet = & self.program.funclets[& funclet_id];
		assert_eq!(funclet.kind, ir::FuncletKind::ScheduleExplicit);
		let funclet_scheduling_extra = & self.program.scheduling_funclet_extras[& funclet_id];
		//let scheduled_value_funclet = & self.program.value_funclets[& scheduling_funclet.value_funclet_id];

		let mut funclet_scoped_state = FuncletScopedState::new(funclet_scheduling_extra.value_funclet_id, funclet_id);

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
			let slot_info = & funclet_scheduling_extra.input_slots[& index];

			match & self.program.types[input_type_id]
			{
				ir::Type::Slot { value_type, queue_stage, queue_place } =>
				{
					let tag = match slot_info.value_tag
					{
						ir::ValueTag::None => ir::ValueTag::None,
						ir::ValueTag::Operation{remote_node_id} => ir::ValueTag::Operation{remote_node_id},
						ir::ValueTag::Input{funclet_id, index} => ir::ValueTag::Operation{remote_node_id : ir::RemoteNodeId{funclet_id, node_id : index}},
						_ => panic!("Unimplemented")
					};
					funclet_scoped_state.slot_value_tags.insert(slot_id, tag);
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

					funclet_scoped_state.node_results.insert(current_node_id, argument_node_results[* index as usize].clone());
				}
				ir::Node::ExtractResult { node_id, index } =>
				{
					// Extracts must appear directly after the call (so that node order reflects scheduling order)
					assert_eq!(current_node_id, * node_id + (* index as usize));

					match & funclet_scoped_state.node_results[node_id]
					{
						_ => panic!("Funclet #{} at node #{} {:?}: Node #{} does not have multiple returns {:?}", funclet_id, current_node_id, node, node_id, placement_state)
					}
				}
				ir::Node::AllocTemporary{ place, type_id, operation } =>
				{
					assert_eq!(funclet_scheduling_extra.value_funclet_id, operation.funclet_id);

					let slot_id = placement_state.scheduling_state.insert_hacked_slot(* type_id, * place, ir::ResourceQueueStage::None);
					funclet_scoped_state.node_results.insert(current_node_id, NodeResult::Slot{slot_id});

					funclet_scoped_state.slot_value_tags.insert(slot_id, ir::ValueTag::Operation{remote_node_id : * operation});

					// To do: Allocate from buffers for GPU/CPU and assign variable
				}
				ir::Node::EncodeDo { place, operation, inputs, outputs } =>
				{
					assert_eq!(funclet_scheduling_extra.value_funclet_id, operation.funclet_id);

					let mut input_slot_ids = Vec::<scheduling_state::SlotId>::new();
					let mut output_slot_ids = Vec::<scheduling_state::SlotId>::new();

					let encoded_funclet = & self.program.funclets[& operation.funclet_id];
					let encoded_node = & encoded_funclet.nodes[operation.node_id];

					for & input_node_id in inputs.iter()
					{
						if let Some(slot_id) = funclet_scoped_state.get_node_slot_id(input_node_id)
						{
							input_slot_ids.push(slot_id);
						}
						else
						{
							panic!("Node #{} (content: {:?}) is not a slot", input_node_id, funclet_scoped_state.node_results[& input_node_id]);
						}
					}

					for & output_node_id in outputs.iter()
					{
						if let Some(slot_id) = funclet_scoped_state.get_node_slot_id(output_node_id)
						{
							output_slot_ids.push(slot_id);
						}
						else
						{
							panic!("Node #{} (content: {:?}) is not a slot", output_node_id, funclet_scoped_state.node_results[& output_node_id]);
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
						ir::Node::Select { .. } => false,
						// Multiple return nodes
						ir::Node::CallExternalCpu { .. } => true,
						ir::Node::CallExternalGpuCompute { .. } => true,
						_ => panic!("Cannot encode {:?}", encoded_node)
					};

					if is_tuple
					{
						for (slot_index, slot_id) in output_slot_ids.iter().enumerate()
						{
							let value_tag = funclet_scoped_state.slot_value_tags[slot_id];
							match value_tag
							{
								ir::ValueTag::None => (),
								ir::ValueTag::FunctionInput{function_id, index} => panic!("{:?} is not a concrete value", value_tag),
								ir::ValueTag::FunctionOutput{function_id, index} => panic!("{:?} is not a concrete value", value_tag),
								ir::ValueTag::Operation{remote_node_id} =>
								{
									assert_eq!(operation.funclet_id, remote_node_id.funclet_id);
									if let ir::Node::ExtractResult { node_id, index } = & encoded_funclet.nodes[remote_node_id.node_id]
									{
										assert_eq!(slot_index, * index);
										assert_eq!(operation.node_id, * node_id);
									}
								}
								ir::ValueTag::Input{funclet_id, index} => panic!("{:?} can only appear in interface of funclet", value_tag),
								ir::ValueTag::Output{funclet_id, index} => panic!("{:?} can only appear in interface of funclet", value_tag),
								ir::ValueTag::Halt{..} => panic!("")
							}
						}
					}
					else
					{
						assert_eq!(output_slot_ids.len(), 1);
						let value_tag = funclet_scoped_state.slot_value_tags[& output_slot_ids[0]];
						match value_tag
						{
							ir::ValueTag::None => (),
							ir::ValueTag::FunctionInput{function_id, index} => panic!("{:?} is not a concrete value", value_tag),
							ir::ValueTag::FunctionOutput{function_id, index} => panic!("{:?} is not a concrete value", value_tag),
							ir::ValueTag::Operation{remote_node_id} =>
							{
								assert_eq!(operation.funclet_id, remote_node_id.funclet_id);
								assert_eq!(operation.node_id, remote_node_id.node_id);
							}
							ir::ValueTag::Input{funclet_id, index} => panic!("{:?} can only appear in interface of funclet", value_tag),
							ir::ValueTag::Output{funclet_id, index} => panic!("{:?} can only appear in interface of funclet", value_tag),
							ir::ValueTag::Halt{..} => panic!("")
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
					let src_slot_id = funclet_scoped_state.get_node_slot_id(* input).unwrap();
					let dst_slot_id = funclet_scoped_state.get_node_slot_id(* output).unwrap();

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
					funclet_scoped_state.node_results.insert(current_node_id, NodeResult::Fence { place : * place, timestamp : local_timestamp });
				}
				ir::Node::SyncFence { place : synced_place, fence } =>
				{
					let local_timestamp = self.advance_local_time(placement_state);
					// Only implemented for the local queue for now
					assert_eq!(* synced_place, ir::Place::Local);
					// To do: Need to update nodes
					let value_opt = match funclet_scoped_state.node_results.get(fence)
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
				ir::Node::Join { funclet : funclet_id, captures } => 
				{
					let mut captured_slot_ids = Vec::<scheduling_state::SlotId>::new();
					let mut captured_var_ids = Vec::<usize>::new();
					let funclet = & self.program.funclets[funclet_id];
					let extra = & self.program.scheduling_funclet_extras[funclet_id];
					for (capture_index, capture_node_id) in captures.iter().enumerate()
					{
						let slot_id = funclet_scoped_state.move_node_slot_id(* capture_node_id).unwrap();
						captured_slot_ids.push(slot_id);
						captured_var_ids.push(placement_state.get_slot_var_id(slot_id).unwrap());

						let slot_value_tag = funclet_scoped_state.slot_value_tags[& slot_id];
						check_value_tag_compatibility_interior(& self.program, slot_value_tag, extra.input_slots[& capture_index].value_tag);
					}

					/*// This is a temporary hack, I hope
					let input_types = & funclet.input_types[captures.len()..];
					let (join_var_id, input_var_ids) = self.code_generator.begin_join(input_types, & funclet.output_types);
					let mut input_slot_ids = Vec::<scheduling_state::SlotId>::new();
					for (input_index, input_var_id) in input_var_ids.iter().enumerate()
					{
						let old_slot_id = captured_slot_ids[input_index];
						let place = placement_state.scheduling_state.get_slot_queue_place(old_slot_id);
						let stage = placement_state.scheduling_state.get_slot_queue_stage(old_slot_id);
						let slot_id = placement_state.scheduling_state.insert_hacked_slot(input_types[input_index], place, stage);
						placement_state.update_slot_state(slot_id, stage, * input_var_id);
						funclet_scoped_state.slot_value_tags.insert(slot_id, extra.input_slots[& input_index].value_tag);
						input_slot_ids.push(slot_id);
					}
					let output_slots = self.compile_scheduling_funclet(* funclet_id, input_slot_ids.as_slice(), pipeline_context, placement_state);
					self.code_generator.end_join(& (output_slots.iter().map(|x| placement_state.get_slot_var_id(* x).unwrap()).collect::<Box<[usize]>>()));*/

					let input_type_ids = & funclet.input_types[captures.len()..];
					let join_var_id = self.code_generator.build_join(* funclet_id, captured_var_ids.as_slice(), input_type_ids, & funclet.output_types);
					pipeline_context.pending_funclet_ids.push(* funclet_id);

					//Some(* type_id)
					funclet_scoped_state.node_results.insert(current_node_id, NodeResult::Join(Join{funclet_id : * funclet_id, captures: captured_slot_ids.into_boxed_slice(), type_id_opt : None, variable_id_opt : Some(join_var_id)}));
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

				let mut output_slots = Vec::<scheduling_state::SlotId>::new();

				for (return_index, return_node_id) in return_values.iter().enumerate()
				{
					let slot_id = funclet_scoped_state.move_node_slot_id(* return_node_id).unwrap();
					output_slots.push(slot_id);

					let slot_value_tag = funclet_scoped_state.slot_value_tags[& slot_id];
					let value_tag = funclet_scheduling_extra.output_slots[& return_index].value_tag;
					check_value_tag_compatibility_interior(& self.program, slot_value_tag, value_tag);
					check_slot_type(& self.program, funclet.output_types[return_index], placement_state.scheduling_state.get_slot_queue_place(slot_id), placement_state.scheduling_state.get_slot_queue_stage(slot_id), None);
				}

				//let return_var_ids = placement_state.get_slot_var_ids(output_slots.as_slice(), ir::Place::Local).unwrap();
				//self.code_generator.build_return(& return_var_ids);
				return output_slots.into_boxed_slice();
				//return vec![].into_boxed_slice();
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
			ir::TailEdge::ScheduleCall { value_operation, /*input_slots,*/ callee_funclet_id, callee_arguments, continuation_funclet_id, continuation_arguments /*continuation_join : continuation_join_node_id*/ } =>
			{
				assert_eq!(funclet_scheduling_extra.value_funclet_id, value_operation.funclet_id);
				let encoded_value_funclet = & self.program.funclets[& value_operation.funclet_id];
				let encoded_node = & encoded_value_funclet.nodes[value_operation.node_id];
				let encoded_value_funclet_id = value_operation.funclet_id;
				match & encoded_node
				{
					ir::Node::CallValueFunction { function_id, arguments } =>
					{
						// Step 1: Check callee
						let callee_funclet = & self.program.funclets[callee_funclet_id];

						assert_eq!(callee_funclet.kind, ir::FuncletKind::ScheduleExplicit);
						let callee_funclet_scheduling_extra = & self.program.scheduling_funclet_extras[callee_funclet_id];
						
						let callee_value_funclet_id = callee_funclet_scheduling_extra.value_funclet_id;
						let callee_value_funclet = & self.program.funclets[& callee_value_funclet_id];
						assert_eq!(callee_value_funclet.kind, ir::FuncletKind::Value);

						// To do: Check that the value function is compatibile with the value funclet for the callee scheduling funclet we're calling

						let mut callee_input_slots = Vec::<scheduling_state::SlotId>::new();
						assert_eq!(callee_arguments.len(), callee_funclet.input_types.len());
						for (callee_argument_index, callee_argument_node_id) in callee_arguments.iter().enumerate()
						{
							let slot_id = funclet_scoped_state.move_node_slot_id(* callee_argument_node_id).unwrap();
							{
								callee_input_slots.push(slot_id);

								let slot_value_tag = funclet_scoped_state.slot_value_tags[& slot_id];

								let value_tag = callee_funclet_scheduling_extra.input_slots[& callee_argument_index].value_tag;

								// Value tag checks are something else...
								let value_type_opt = match (slot_value_tag, value_tag)
								{
									(_, ir::ValueTag::None) => None,
									(ir::ValueTag::Operation{remote_node_id}, ir::ValueTag::Input{funclet_id, index}) =>
									{
										// Sanity
										assert_eq!(remote_node_id.funclet_id, funclet_scoped_state.value_funclet_id);
										assert_eq!(callee_value_funclet_id, funclet_id);
										// To do: Need to add this to check_slot_type() To do: Done
										//assert_eq!(callee_value_funclet.input_types[index], * value_type);

										// All this ceremony leads up to this:
										// We need to check if the value for this slot matches the argument in this position for the specified value funclet
										assert_eq!(arguments[index], remote_node_id.node_id);
										// That's "it"
										Some(callee_value_funclet.input_types[index])
									}
									_ => panic!("Ill-formed: {:?} to {:?}", slot_value_tag, value_tag)
								};

								check_slot_type(& self.program, callee_funclet.input_types[callee_argument_index], placement_state.scheduling_state.get_slot_queue_place(slot_id), placement_state.scheduling_state.get_slot_queue_stage(slot_id), value_type_opt);
							}
						}

						// To do: Check type compatibility

						// Step 2: Continuation

						/*let continuation_join = funclet_scoped_state.move_node_join(* continuation_join_node_id).unwrap();
						let continuation_funclet_id = & continuation_join.funclet_id;
						let continuation_captures = continuation_join.captures;
						let continuation_var_id = continuation_join.variable_id_opt.unwrap();*/

						let continuation_funclet = & self.program.funclets[continuation_funclet_id];
						assert_eq!(continuation_funclet.kind, ir::FuncletKind::ScheduleExplicit);
						let continuation_funclet_scheduling_extra = & self.program.scheduling_funclet_extras[continuation_funclet_id];
						assert_eq!(funclet_scheduling_extra.value_funclet_id, continuation_funclet_scheduling_extra.value_funclet_id);

						let continuation_value_funclet_id = continuation_funclet_scheduling_extra.value_funclet_id;
						let continuation_value_funclet = & self.program.funclets[& continuation_value_funclet_id];
						assert_eq!(continuation_value_funclet.kind, ir::FuncletKind::Value);

						assert_eq!(encoded_value_funclet_id, continuation_value_funclet_id);


						let mut continuation_input_slots = Vec::<scheduling_state::SlotId>::new();
						//continuation_input_slots.extend_from_slice(& continuation_captures);

						assert_eq!(continuation_arguments.len() + callee_funclet.output_types.len(), continuation_funclet.input_types.len());

						let mut continuation_input_slots = Vec::<scheduling_state::SlotId>::new();
						for (continuation_argument_index, continuation_argument_node_id) in continuation_arguments.iter().enumerate()
						{
							let slot_id = funclet_scoped_state.move_node_slot_id(* continuation_argument_node_id).unwrap();
							{

								let slot_value_tag = funclet_scoped_state.slot_value_tags[& slot_id];
								let value_tag = continuation_funclet_scheduling_extra.input_slots[& continuation_argument_index].value_tag;
								check_value_tag_compatibility_interior(& self.program, slot_value_tag, value_tag);
								check_slot_type(& self.program, continuation_funclet.input_types[continuation_argument_index], placement_state.scheduling_state.get_slot_queue_place(slot_id), placement_state.scheduling_state.get_slot_queue_stage(slot_id), None);
							}
						}

						for (callee_output_index, callee_output_type) in callee_funclet.output_types.iter().enumerate()
						{
							let continuation_input_index = continuation_arguments.len() + callee_output_index;
							assert_eq!(* callee_output_type, continuation_funclet.input_types[continuation_input_index]);

							let value_tag = callee_funclet_scheduling_extra.output_slots[& callee_output_index].value_tag;
							let value_tag_2 = continuation_funclet_scheduling_extra.input_slots[& continuation_input_index].value_tag;

							match (value_tag, value_tag_2)
							{
								(_, ir::ValueTag::None) => (),
								(ir::ValueTag::Output{funclet_id, index : output_index}, ir::ValueTag::Operation{remote_node_id}) =>
								{
									assert_eq!(remote_node_id.funclet_id, value_operation.funclet_id);
									assert_eq!(funclet_id, callee_value_funclet_id);
									let node = & self.program.funclets[& remote_node_id.funclet_id].nodes[remote_node_id.node_id];
									if let ir::Node::ExtractResult{node_id : call_node_id, index} = node
									{
										assert_eq!(* index, output_index);
										assert_eq!(* call_node_id, value_operation.node_id);
									}
									else
									{
										panic!("Target operation is not a result extraction: #{:?} {:?}", remote_node_id, node);
									}
								}
								_ => panic!("Ill-formed: {:?} to {:?}", value_tag, value_tag_2)
							};
						}
						
						let mut callee_output_slots = self.compile_scheduling_funclet(* callee_funclet_id, callee_input_slots.as_slice(), pipeline_context, placement_state);
						continuation_input_slots.extend_from_slice(& callee_output_slots);
						return self.compile_scheduling_funclet(* continuation_funclet_id, continuation_input_slots.as_slice(), pipeline_context, placement_state);
						//let continuation_input_var_ids = continuation_input_slots.iter().map(|x| placement_state.slot_variable_ids[x]).collect::<Box<[usize]>>();
						//self.code_generator.call_join(continuation_var_id, & continuation_input_var_ids);
						//return vec![].into_boxed_slice();
					}
					_ => panic!("Node cannot be scheduled via ScheduleCall")
				}
			}
			ir::TailEdge::ScheduleSelect { value_operation, callee_funclet_ids, callee_arguments, continuation_funclet_id } =>
			{

				// The operation of each output of the callee funclets must be exactly equal to the corresponding input of the continuation funclet
				// Except for the 0th output, which will be the respective branch in the callee funclet and the select itself in the continuation input

				assert_eq!(funclet_scheduling_extra.value_funclet_id, value_operation.funclet_id);
				let encoded_value_funclet = & self.program.funclets[& value_operation.funclet_id];
				let encoded_node = & encoded_value_funclet.nodes[value_operation.node_id];
				let encoded_value_funclet_id = value_operation.funclet_id;

				let continuation_funclet = & self.program.funclets[continuation_funclet_id];
				assert_eq!(continuation_funclet.kind, ir::FuncletKind::ScheduleExplicit);
				let continuation_funclet_scheduling_extra = & self.program.scheduling_funclet_extras[continuation_funclet_id];
				assert_eq!(funclet_scheduling_extra.value_funclet_id, continuation_funclet_scheduling_extra.value_funclet_id);

				let continuation_value_funclet_id = continuation_funclet_scheduling_extra.value_funclet_id;
				let continuation_value_funclet = & self.program.funclets[& continuation_value_funclet_id];
				assert_eq!(continuation_value_funclet.kind, ir::FuncletKind::Value);

				assert_eq!(encoded_value_funclet_id, continuation_value_funclet_id);

				match & encoded_node
				{
					ir::Node::Select { condition, true_case, false_case } =>
					{

						// To do: Refactor compatibility check in ScheduleCall so writing the check for ScheduleSelect doesn't cause me physical pain

						//let condition_slot_id = if let NodeResult::Slot{slot_id} = placement_state.node_results[& callee_arguments[0]] { * slot_id } else { panic!("") };
		
						let input_slot_ids = callee_arguments.iter().map(|& node_id| funclet_scoped_state.get_node_slot_id(node_id).unwrap()).collect::<Box<[scheduling_state::SlotId]>>();
						let condition_var_id = placement_state.get_slot_var_id(input_slot_ids[0]).unwrap();
						
						let output_var_ids = self.code_generator.begin_if_else(condition_var_id, & continuation_funclet.input_types);
						let if_branch_output_slots = self.compile_scheduling_funclet(callee_funclet_ids[0], & input_slot_ids, pipeline_context, placement_state);
						self.code_generator.end_if_begin_else(& placement_state.get_slot_var_ids(& if_branch_output_slots, ir::Place::Local).unwrap());
						let else_branch_output_slots = self.compile_scheduling_funclet(callee_funclet_ids[1], & input_slot_ids, pipeline_context, placement_state);
						self.code_generator.end_else(& placement_state.get_slot_var_ids(& else_branch_output_slots, ir::Place::Local).unwrap());
		
						let mut output_slot_ids = Vec::<scheduling_state::SlotId>::new();
						for (output_index, var_id) in output_var_ids.iter().enumerate()
						{
							let slot_id = placement_state.scheduling_state.insert_hacked_slot(continuation_funclet.input_types[output_index], ir::Place::Local, ir::ResourceQueueStage::None);
							//slot_value_tags.insert(slot_id, Some(ir::ValueTag::Operation{remote_node_id : * operation}));
							placement_state.update_slot_state(slot_id, ir::ResourceQueueStage::Ready, * var_id);
							output_slot_ids.push(slot_id);
						}

						return self.compile_scheduling_funclet(* continuation_funclet_id, output_slot_ids.as_slice(), pipeline_context, placement_state);
					}
					_ => panic!("Must be a select node")
				}
			}
			/*ir::TailEdge::ScheduleTailCall { value_operation, callee_funclet_id, arguments } =>
			{
				
			}
			ir::TailEdge::ScheduleTailSelect { value_operation, condition, callee_funclet_ids, arguments } =>
			{

			}*/
			ir::TailEdge::Jump { join, arguments } =>
			{
				let join = funclet_scoped_state.move_node_join(* join).unwrap();

				let destination_funclet = & self.program.funclets[& join.funclet_id];
				let destination_extra = & self.program.scheduling_funclet_extras[& join.funclet_id];

				let mut input_slot_ids = Vec::<scheduling_state::SlotId>::new();
				input_slot_ids.extend_from_slice(& join.captures);

				for (argument_index, argument_node_id) in arguments.iter().enumerate()
				{
					let slot_id = funclet_scoped_state.move_node_slot_id(* argument_node_id).unwrap();
					input_slot_ids.push(slot_id);
					let slot_value_tag = funclet_scoped_state.slot_value_tags[& slot_id];
					// We need to shift the destination argument index to account for the captures (that are checked at construction)
					let destination_argument_index = argument_index + join.captures.len();
					check_value_tag_compatibility_interior(& self.program, slot_value_tag, destination_extra.input_slots[& destination_argument_index].value_tag);
					check_slot_type(& self.program, destination_funclet.input_types[destination_argument_index], placement_state.scheduling_state.get_slot_queue_place(slot_id), placement_state.scheduling_state.get_slot_queue_stage(slot_id), None);
				}

				// source and destination refer to the funclets and not the slots themselves, so this is seemingly backwards
				for (source_output_index, source_slot) in funclet_scheduling_extra.output_slots.iter()
				{
					let destination_slot = & destination_extra.output_slots[source_output_index];
					check_value_tag_compatibility_interior(& self.program, destination_slot.value_tag, source_slot.value_tag);
					assert_eq!(destination_funclet.output_types[* source_output_index], funclet.output_types[* source_output_index]);
				}
				
				//return self.compile_scheduling_funclet(join.funclet_id, input_slot_ids.as_slice(), pipeline_context, placement_state);
				let continuation_input_var_ids = input_slot_ids.iter().map(|x| placement_state.slot_variable_ids[x]).collect::<Box<[usize]>>();
				self.code_generator.call_join(join.variable_id_opt.unwrap(), & continuation_input_var_ids);
			}
			_ => panic!("Umimplemented")
		}

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
		//self.code_generator.emit_oneshot_pipeline_entry_point(entry_funclet_id, &entry_funclet.input_types, &entry_funclet.output_types);

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
