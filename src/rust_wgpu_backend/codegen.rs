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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JoinPointId(usize);

#[derive(Debug, Clone)]
enum NodeResult
{
	None,
	Slot { slot_id : scheduling_state::SlotId },
	Fence { place : ir::Place, timestamp : LogicalTimestamp },
	Join{ join_point_id : JoinPointId}
}

#[derive(Debug, Clone)]
struct RootJoinPoint
{
	value_funclet_id : ir::FuncletId,
	input_types : Box<[ir::TypeId]>,
	input_slot_value_tags : HashMap<usize, ir::ValueTag>
}

#[derive(Debug, Clone)]
struct SimpleJoinPoint
{
	value_funclet_id : ir::FuncletId,
	scheduling_funclet_id : ir::FuncletId,
	captures : Box<[scheduling_state::SlotId]>,
	continuation_join_point_id : JoinPointId
}

#[derive(Debug)]
enum JoinPoint
{
	RootJoinPoint(RootJoinPoint),
	SimpleJoinPoint(SimpleJoinPoint),
}

// Should probably get rid of used and make a trait...
impl JoinPoint
{
	fn get_value_funclet_id(&self) -> ir::FuncletId
	{
		match self
		{
			Self::SimpleJoinPoint(join_point) => join_point.value_funclet_id,
			Self::RootJoinPoint(join_point) => join_point.value_funclet_id,
		}
	}

	fn get_input_count(&self, program : & ir::Program) -> usize
	{
		match self
		{
			Self::SimpleJoinPoint(join_point) =>
			{
				let funclet = & program.funclets[& join_point.scheduling_funclet_id];
				funclet.input_types.len()
			}
			Self::RootJoinPoint(join_point) =>
			{
				join_point.input_types.len()
			}
		}
	}

	fn get_capture_count(&self) -> usize
	{
		match self
		{
			Self::SimpleJoinPoint(join_point) => join_point.captures.len(),
			Self::RootJoinPoint(_) => 0,
		}
	}

	fn get_scheduling_input_type(&self, program : & ir::Program, index : usize) -> ir::TypeId
	{
		match self
		{
			Self::SimpleJoinPoint(join_point) =>
			{
				let funclet = & program.funclets[& join_point.scheduling_funclet_id];
				funclet.input_types[index]
			}
			Self::RootJoinPoint(join_point) =>
			{
				join_point.input_types[index]
			}
		}
	}

	fn get_scheduling_input_value_tag(&self, program : & ir::Program, index : usize) -> ir::ValueTag
	{
		match self
		{
			Self::SimpleJoinPoint(join_point) =>
			{
				let funclet = & program.funclets[& join_point.scheduling_funclet_id];
				let extra = & program.scheduling_funclet_extras[& join_point.scheduling_funclet_id];
				extra.input_slots[& index].value_tag
			}
			Self::RootJoinPoint(join_point) =>
			{
				join_point.input_slot_value_tags[& index]
			}
		}
	}
}

#[derive(Debug, Default)]
struct JoinGraph
{
	join_points : Vec<Option<JoinPoint>>
}

impl JoinGraph
{
	fn new() -> Self
	{
		Self { join_points : vec![] }
	}

	fn create(&mut self, join_point : JoinPoint) -> JoinPointId
	{
		let index = self.join_points.len();
		self.join_points.push(Some(join_point));
		JoinPointId(index)
	}

	fn move_join(&mut self, join_point_id : JoinPointId) -> JoinPoint
	{
		let mut join_point = None;
		std::mem::swap(&mut join_point, &mut self.join_points[join_point_id.0]);
		join_point.unwrap()
	}

	fn get_join(& self, join_point_id : JoinPointId) -> & JoinPoint
	{
		self.join_points[join_point_id.0].as_ref().unwrap()
	}
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
	join_graph : JoinGraph
}

impl PlacementState
{
	fn new() -> Self
	{
		let mut place_states = HashMap::<ir::Place, PlaceState>::new();
		place_states.insert(ir::Place::Gpu, PlaceState{ .. Default::default() });
		place_states.insert(ir::Place::Local, PlaceState{ .. Default::default() });
		Self{ place_states, scheduling_state : scheduling_state::SchedulingState::new(), /*node_results : Default::default(),*/ submission_map : HashMap::new(), slot_variable_ids : HashMap::new()/*, value_tags : HashMap::new()*/, join_graph : JoinGraph::new()}
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
enum SplitPoint
{
	Next { return_slot_ids : Box<[scheduling_state::SlotId]>, continuation_join_point_id_opt : Option<JoinPointId> },
	Select { return_slot_ids : Box<[scheduling_state::SlotId]>, condition_slot_id : scheduling_state::SlotId, true_funclet_id : ir::FuncletId, false_funclet_id : ir::FuncletId, continuation_join_point_id_opt : Option<JoinPointId> }
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

	fn get_node_join_point_id(&self, node_id : ir::NodeId) -> Option<JoinPointId>
	{
		match & self.node_results[& node_id]
		{
			NodeResult::Join{join_point_id} => Some(* join_point_id),
			_ => None
		}
	}

	fn move_node_join_point_id(&mut self, node_id : ir::NodeId) -> Option<JoinPointId>
	{
		let node_result_opt = self.node_results.remove(& node_id);

		if let Some(node_result) = node_result_opt
		{
			if let NodeResult::Join{join_point_id} = node_result
			{
				self.node_results.insert(node_id, NodeResult::None);
				return Some(join_point_id)
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

fn check_value_tag_compatibility_enter(program : & ir::Program, call_operation : ir::RemoteNodeId, caller_value_tag : ir::ValueTag, callee_value_tag : ir::ValueTag)
{
	match (caller_value_tag, callee_value_tag)
	{
		(_, ir::ValueTag::None) => (),
		(ir::ValueTag::Operation{remote_node_id}, ir::ValueTag::Input{funclet_id, index}) =>
		{
			assert_eq!(call_operation.funclet_id, remote_node_id.funclet_id);
			let caller_value_funclet = & program.funclets[& call_operation.funclet_id];
			if let ir::Node::CallValueFunction{function_id, arguments} = & caller_value_funclet.nodes[call_operation.node_id]
			{
				assert_eq!(arguments[index], remote_node_id.node_id);
			}
			else
			{
				panic!("Operation is not a call {:?}",  call_operation);
			}
		}
		_ => panic!("Ill-formed: {:?} to {:?}", caller_value_tag, callee_value_tag)
	}
}

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
		(ir::ValueTag::Output{funclet_id, index}, ir::ValueTag::Output{funclet_id : funclet_id_2, index : index_2}) =>
		{
			assert_eq!(funclet_id, funclet_id_2);
			assert_eq!(index, index_2);
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

	fn encode_do_node_gpu(&mut self, placement_state : &mut PlacementState, funclet_scoped_state : &mut FuncletScopedState, node : & ir::Node, input_slot_ids : & [scheduling_state::SlotId], output_slot_ids : & [scheduling_state::SlotId])
	{
		match node
		{
			ir::Node::CallExternalGpuCompute {external_function_id, arguments, dimensions} =>
			{
				let function = & self.program.external_gpu_functions[* external_function_id];

				assert_eq!(input_slot_ids.len(), dimensions.len() + arguments.len());
				assert_eq!(output_slot_ids.len(), function.output_types.len());

				for (input_index, input_node_id) in dimensions.iter().chain(arguments.iter()).enumerate()
				{
					let slot_id = input_slot_ids[input_index];
					let value_tag = funclet_scoped_state.slot_value_tags[& slot_id];
					let funclet_id = funclet_scoped_state.value_funclet_id;
					check_value_tag_compatibility_interior(& self.program, value_tag, ir::ValueTag::Operation{remote_node_id : ir::RemoteNodeId{funclet_id, node_id : * input_node_id}});
				}

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

	fn encode_do_node_local(&mut self, placement_state : &mut PlacementState, funclet_scoped_state : &mut FuncletScopedState, node : & ir::Node, input_slot_ids : & [scheduling_state::SlotId], output_slot_ids : &[scheduling_state::SlotId])
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

				for (input_index, input_node_id) in [* condition, * true_case, * false_case].iter().enumerate()
				{
					let slot_id = input_slot_ids[input_index];
					let value_tag = funclet_scoped_state.slot_value_tags[& slot_id];
					let funclet_id = funclet_scoped_state.value_funclet_id;
					check_value_tag_compatibility_interior(& self.program, value_tag, ir::ValueTag::Operation{remote_node_id : ir::RemoteNodeId{funclet_id, node_id : * input_node_id}});
				}

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

				for (input_index, input_node_id) in arguments.iter().enumerate()
				{
					let slot_id = input_slot_ids[input_index];
					let value_tag = funclet_scoped_state.slot_value_tags[& slot_id];
					let funclet_id = funclet_scoped_state.value_funclet_id;
					check_value_tag_compatibility_interior(& self.program, value_tag, ir::ValueTag::Operation{remote_node_id : ir::RemoteNodeId{funclet_id, node_id : * input_node_id}});
				}

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
						let slot_id = placement_state.scheduling_state.insert_hacked_slot(* value_type, * queue_place, * queue_stage);
						placement_state.slot_variable_ids.insert(slot_id, argument_variable_ids[index]);
						argument_slot_ids.push(slot_id);
					}
					_ => panic!("Unimplemented")
				}
			};
		}

		let mut default_join_point_id_opt = 
		{
			let extra = & self.program.scheduling_funclet_extras[& funclet_id];
			let input_types = funclet.output_types.clone();
			let value_funclet_id = extra.value_funclet_id;
			let mut input_slot_value_tags = HashMap::<usize, ir::ValueTag>::new();
			for (input_index, input_slot) in extra.input_slots.iter()
			{
				input_slot_value_tags.insert(* input_index, ir::ValueTag::Output{funclet_id : value_funclet_id, index : * input_index});
			}
			let join_point_id = placement_state.join_graph.create(JoinPoint::RootJoinPoint(RootJoinPoint{value_funclet_id, input_types, input_slot_value_tags}));
			Option::<JoinPointId>::Some(join_point_id)
		};

		enum TraversalState
		{
			SelectIf { branch_input_slot_ids : Box<[scheduling_state::SlotId]>, condition_slot_id : scheduling_state::SlotId, true_funclet_id : ir::FuncletId, false_funclet_id : ir::FuncletId, continuation_join_point_id_opt : Option<JoinPointId> },
			SelectElse { output_slot_ids : Box<[scheduling_state::SlotId]>, branch_input_slot_ids : Box<[scheduling_state::SlotId]>, false_funclet_id : ir::FuncletId, continuation_join_point_id_opt : Option<JoinPointId> },
			SelectEnd { output_slot_ids : Box<[scheduling_state::SlotId]>, continuation_join_point_id_opt : Option<JoinPointId> },
		}

		let mut traversal_state_stack = Vec::<TraversalState>::new();

		let mut current_output_slot_ids = argument_slot_ids.into_boxed_slice();
		let mut current_funclet_id_opt = Some(funclet_id);

		//while let Some(split_point_stack_entry) = split_point_stack.pop()
		while current_funclet_id_opt.is_some()
		{
			while let Some(current_funclet_id) = current_funclet_id_opt
			{
				//current_output_slot_ids = 
				let split_point = self.compile_scheduling_funclet(current_funclet_id, & current_output_slot_ids, pipeline_context, &mut placement_state, default_join_point_id_opt);
				println!("Split point: {:?}", split_point);
				current_output_slot_ids = match split_point
				{
					SplitPoint::Next{return_slot_ids, continuation_join_point_id_opt} =>
					{
						default_join_point_id_opt = continuation_join_point_id_opt;
						return_slot_ids
					}
					SplitPoint::Select{return_slot_ids, condition_slot_id, true_funclet_id, false_funclet_id, continuation_join_point_id_opt} =>
					{
						//assert!(default_join_point_id_opt.is_none());
						traversal_state_stack.push(TraversalState::SelectIf{ branch_input_slot_ids : return_slot_ids, condition_slot_id, true_funclet_id, false_funclet_id, continuation_join_point_id_opt });
						vec![].into_boxed_slice()
					}
				};

				if default_join_point_id_opt.is_none()
				{
					while let Some(traversal_state) = traversal_state_stack.pop()
					{
						match traversal_state
						{
							TraversalState::SelectIf { branch_input_slot_ids, condition_slot_id, true_funclet_id, false_funclet_id, continuation_join_point_id_opt } =>
							{
								let condition_var_id = placement_state.get_slot_var_id(condition_slot_id).unwrap();
								let true_funclet = & self.program.funclets[& true_funclet_id];
								let true_funclet_extra = & self.program.scheduling_funclet_extras[& true_funclet_id];
								//true_funclet_extra.input_slots[& output_index].value_tag
								let output_var_ids = self.code_generator.begin_if_else(condition_var_id, & true_funclet.output_types);
								let mut output_slot_ids = Vec::<scheduling_state::SlotId>::new();
								for (output_index, output_type) in true_funclet.output_types.iter().enumerate()
								{
									let (value_type, queue_stage, queue_place) = if let ir::Type::Slot{value_type, queue_stage, queue_place} = & self.program.types[output_type]
									{
										(* value_type, * queue_stage, * queue_place)
									}
									else
									{
										panic!("Not a slot")
									};
									let slot_id = placement_state.scheduling_state.insert_hacked_slot(value_type, queue_place, queue_stage);
									output_slot_ids.push(slot_id);
								}
								current_funclet_id_opt = Some(true_funclet_id);
								current_output_slot_ids = branch_input_slot_ids.clone();
								traversal_state_stack.push(TraversalState::SelectElse{output_slot_ids : output_slot_ids.into_boxed_slice(), branch_input_slot_ids, false_funclet_id, continuation_join_point_id_opt});
							}
							TraversalState::SelectElse { output_slot_ids, branch_input_slot_ids, false_funclet_id, continuation_join_point_id_opt } =>
							{
								self.code_generator.end_if_begin_else(& placement_state.get_slot_var_ids(& current_output_slot_ids, ir::Place::Local).unwrap());
								current_funclet_id_opt = Some(false_funclet_id);
								current_output_slot_ids = branch_input_slot_ids;
								traversal_state_stack.push(TraversalState::SelectEnd{output_slot_ids, continuation_join_point_id_opt});
							}
							TraversalState::SelectEnd { output_slot_ids, continuation_join_point_id_opt } =>
							{
								self.code_generator.end_else(& placement_state.get_slot_var_ids(& current_output_slot_ids, ir::Place::Local).unwrap());
								default_join_point_id_opt = continuation_join_point_id_opt;
								current_output_slot_ids = output_slot_ids;
							}
						}
					}
				}

				current_funclet_id_opt = None;
				if let Some(join_point_id) = default_join_point_id_opt
				{
					default_join_point_id_opt = None;
					let join_point = placement_state.join_graph.move_join(join_point_id);
					println!("Continuing to {:?} {:?}", join_point_id, join_point);

					match & join_point
					{
						JoinPoint::RootJoinPoint(_) =>
						{
							let return_var_ids = placement_state.get_slot_var_ids(& current_output_slot_ids, ir::Place::Local).unwrap();
							self.code_generator.build_return(& return_var_ids);
						}
						JoinPoint::SimpleJoinPoint(simple_join_point) =>
						{
							let mut input_slot_ids = Vec::<scheduling_state::SlotId>::new();
							input_slot_ids.extend_from_slice(& simple_join_point.captures);
							input_slot_ids.extend_from_slice(& current_output_slot_ids);
							
							current_funclet_id_opt = Some(simple_join_point.scheduling_funclet_id);
							default_join_point_id_opt = Some(simple_join_point.continuation_join_point_id);
							current_output_slot_ids = input_slot_ids.into_boxed_slice();
						}
						_ => panic!("Jump to invalid join point #{:?}: {:?}", join_point_id, join_point)
					}

					println!("{:?} {:?} {:?}", current_funclet_id_opt, default_join_point_id_opt, current_output_slot_ids);
				}
			}

			assert!(current_funclet_id_opt.is_none());
		}

		self.code_generator.end_funclet();
	}

	fn compile_scheduling_funclet(&mut self, funclet_id : ir::FuncletId, argument_slot_ids : &[scheduling_state::SlotId], pipeline_context : &mut PipelineContext, placement_state : &mut PlacementState, mut default_join_point_id_opt : Option<JoinPointId>) -> SplitPoint //Box<[scheduling_state::SlotId]>
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
			println!("Compiling Funclet #{} with join {:?}...\n{:?}\n", funclet_id, default_join_point_id_opt, funclet);
		}

		for (current_node_id, node) in funclet.nodes.iter().enumerate()
		{
			self.code_generator.insert_comment(format!(" node #{}: {:?}", current_node_id, node).as_str());

			if self.print_codegen_debug_info
			{
				println!("#{} {:?} : {:?} {:?}", current_node_id, node, placement_state, funclet_scoped_state);
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
					match place
					{
						ir::Place::Cpu => (),
						ir::Place::Local => (),
						ir::Place::Gpu =>
						{
							let var_id = self.code_generator.build_create_buffer(* type_id);
							placement_state.update_slot_state(slot_id, ir::ResourceQueueStage::None, var_id);
						}
					}
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

					match place
					{
						ir::Place::Local =>
						{
							self.encode_do_node_local(placement_state, &mut funclet_scoped_state, encoded_node, input_slot_ids.as_slice(), output_slot_ids.as_slice());
						}
						ir::Place::Gpu =>
						{
							self.encode_do_node_gpu(placement_state, &mut funclet_scoped_state, encoded_node, input_slot_ids.as_slice(), output_slot_ids.as_slice());
						}
						ir::Place::Cpu => (),
					}
				}
				ir::Node::EncodeCopy { place, input, output } =>
				{
					let src_slot_id = funclet_scoped_state.get_node_slot_id(* input).unwrap();
					let dst_slot_id = funclet_scoped_state.get_node_slot_id(* output).unwrap();

					let src_place = placement_state.scheduling_state.get_slot_queue_place(src_slot_id);
					let dst_place = placement_state.scheduling_state.get_slot_queue_place(dst_slot_id);

					let src_stage = placement_state.scheduling_state.get_slot_queue_stage(src_slot_id);

					assert_eq!(placement_state.scheduling_state.get_slot_type_id(src_slot_id), placement_state.scheduling_state.get_slot_type_id(dst_slot_id));
					assert!(src_stage > ir::ResourceQueueStage::None);
					assert!(src_stage < ir::ResourceQueueStage::Dead);
					assert_eq!(placement_state.scheduling_state.get_slot_queue_stage(dst_slot_id), ir::ResourceQueueStage::None);

					{
						let source_value_tag = funclet_scoped_state.slot_value_tags[& src_slot_id];
						let destination_value_tag = funclet_scoped_state.slot_value_tags[& dst_slot_id];
						check_value_tag_compatibility_interior(& self.program, source_value_tag, destination_value_tag);
					}

					match (* place, dst_place, src_place)
					{
						(ir::Place::Local, ir::Place::Local, ir::Place::Local) =>
						{
							assert!(src_stage == ir::ResourceQueueStage::Ready);
							let src_var_id = placement_state.get_slot_var_id(src_slot_id).unwrap();
							placement_state.update_slot_state(dst_slot_id, ir::ResourceQueueStage::Ready, src_var_id);
						}
						(ir::Place::Local, ir::Place::Local, ir::Place::Gpu) =>
						{
							assert!(src_stage == ir::ResourceQueueStage::Ready);
							let src_var_id = placement_state.get_slot_var_id(src_slot_id).unwrap();
							let dst_var_id = self.code_generator.encode_clone_local_data_from_buffer(src_var_id);
							placement_state.update_slot_state(dst_slot_id, ir::ResourceQueueStage::Ready, dst_var_id);
						}
						(ir::Place::Gpu, ir::Place::Gpu, ir::Place::Local) =>
						{
							let src_var_id = placement_state.get_slot_var_id(src_slot_id).unwrap();
							let dst_var_id = placement_state.get_slot_var_id(dst_slot_id).unwrap();
							self.code_generator.encode_copy_buffer_from_local_data(dst_var_id, src_var_id);
							placement_state.update_slot_state(dst_slot_id, ir::ResourceQueueStage::Encoded, dst_var_id);
						}
						(ir::Place::Gpu, ir::Place::Gpu, ir::Place::Gpu) =>
						{
							let src_var_id = placement_state.get_slot_var_id(src_slot_id).unwrap();
							let dst_var_id = placement_state.get_slot_var_id(dst_slot_id).unwrap();
							self.code_generator.encode_copy_buffer_from_buffer(dst_var_id, src_var_id);
							placement_state.update_slot_state(dst_slot_id, ir::ResourceQueueStage::Encoded, dst_var_id);
						}
						_ => panic!("Unimplemented")
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
				ir::Node::DefaultJoin =>
				{
					if let Some(join_point_id) = default_join_point_id_opt
					{
						default_join_point_id_opt = None;
						funclet_scoped_state.node_results.insert(current_node_id, NodeResult::Join{ join_point_id });
					}
					else
					{
						panic!("No default join point")
					}
				}
				ir::Node::Join { funclet : funclet_id, captures, continuation : continuation_join_node_id } => 
				{
					let mut captured_slot_ids = Vec::<scheduling_state::SlotId>::new();
					//let mut captured_var_ids = Vec::<usize>::new();
					let join_funclet = & self.program.funclets[funclet_id];
					let extra = & self.program.scheduling_funclet_extras[funclet_id];

					// Join points can only be constructed for the value funclet they are created in
					assert_eq!(extra.value_funclet_id, funclet_scoped_state.value_funclet_id);

					for (capture_index, capture_node_id) in captures.iter().enumerate()
					{
						let slot_id = funclet_scoped_state.move_node_slot_id(* capture_node_id).unwrap();
						captured_slot_ids.push(slot_id);
						//captured_var_ids.push(placement_state.get_slot_var_id(slot_id).unwrap());

						let slot_value_tag = funclet_scoped_state.slot_value_tags[& slot_id];
						check_value_tag_compatibility_interior(& self.program, slot_value_tag, extra.input_slots[& capture_index].value_tag);
					}

					let continuation_join_point_id = funclet_scoped_state.move_node_join_point_id(* continuation_join_node_id).unwrap();
					let continuation_join_point = placement_state.join_graph.get_join(continuation_join_point_id);

					for (join_output_index, join_output_type) in join_funclet.output_types.iter().enumerate()
					{
						let continuation_input_index = continuation_join_point.get_capture_count() + join_output_index;
						assert_eq!(* join_output_type, continuation_join_point.get_scheduling_input_type(& self.program, continuation_input_index));

						let value_tag = extra.output_slots[& join_output_index].value_tag;
						let value_tag_2 = continuation_join_point.get_scheduling_input_value_tag(& self.program, continuation_input_index);

						check_value_tag_compatibility_interior(& self.program, value_tag, value_tag_2);
					}

					/*let input_type_ids = & funclet.input_types[captures.len()..];
					let join_var_id = self.code_generator.build_join(* funclet_id, captured_var_ids.as_slice(), input_type_ids, & funclet.output_types);
					pipeline_context.pending_funclet_ids.push(* funclet_id);

					//Some(* type_id)
					funclet_scoped_state.node_results.insert(current_node_id, NodeResult::Join(Join{funclet_id : * funclet_id, captures: captured_slot_ids.into_boxed_slice(), type_id_opt : None, variable_id_opt : Some(join_var_id)}));*/
					let join_point_id = placement_state.join_graph.create(JoinPoint::SimpleJoinPoint(SimpleJoinPoint{value_funclet_id : extra.value_funclet_id, scheduling_funclet_id : * funclet_id, captures : captured_slot_ids.into_boxed_slice(), continuation_join_point_id}));
					println!("Created join point: {:?} {:?}", join_point_id, placement_state.join_graph.get_join(join_point_id));
					funclet_scoped_state.node_results.insert(current_node_id, NodeResult::Join{ join_point_id });
				}
				_ => panic!("Unknown node")
			};
		}

		self.code_generator.insert_comment(format!(" tail edge: {:?}", funclet.tail_edge).as_str());
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

				return SplitPoint::Next{return_slot_ids : output_slots.into_boxed_slice(), continuation_join_point_id_opt : default_join_point_id_opt};
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
			ir::TailEdge::Jump { join, arguments } =>
			{
				let mut join_point_id = funclet_scoped_state.move_node_join_point_id(* join).unwrap();

				let mut argument_slot_ids = Vec::<scheduling_state::SlotId>::new();

				for (argument_index, argument_node_id) in arguments.iter().enumerate()
				{
					let slot_id = funclet_scoped_state.move_node_slot_id(* argument_node_id).unwrap();
					argument_slot_ids.push(slot_id);
				}

				{
					let join_point = placement_state.join_graph.get_join(join_point_id);

					// We shouldn't have to check outputs for join points because all join chains go up to the root

					for (argument_index, argument_slot_id) in argument_slot_ids.iter().enumerate()
					{
						let slot_id = * argument_slot_id;
						let slot_value_tag = funclet_scoped_state.slot_value_tags[& slot_id];
						// We need to shift the destination argument index to account for the captures (that are checked at construction)
						let destination_argument_index = argument_index + join_point.get_capture_count();
						check_value_tag_compatibility_interior(& self.program, slot_value_tag, join_point.get_scheduling_input_value_tag(& self.program, destination_argument_index));
						check_slot_type(& self.program, join_point.get_scheduling_input_type(& self.program, destination_argument_index), placement_state.scheduling_state.get_slot_queue_place(slot_id), placement_state.scheduling_state.get_slot_queue_stage(slot_id), None);
					}
				}

				assert!(default_join_point_id_opt.is_none());
				return SplitPoint::Next { return_slot_ids : argument_slot_ids.into_boxed_slice(), continuation_join_point_id_opt : Some(join_point_id)};
			}
			ir::TailEdge::ScheduleCall { value_operation : value_operation_ref, callee_funclet_id : callee_scheduling_funclet_id_ref, callee_arguments, continuation_join : continuation_join_node_id } =>
			{
				let value_operation = * value_operation_ref;
				let callee_scheduling_funclet_id = * callee_scheduling_funclet_id_ref;

				let continuation_join_point_id = funclet_scoped_state.move_node_join_point_id(* continuation_join_node_id).unwrap();
				let continuation_join_point = placement_state.join_graph.get_join(continuation_join_point_id);

				assert_eq!(value_operation.funclet_id, funclet_scoped_state.value_funclet_id);
				assert_eq!(continuation_join_point.get_value_funclet_id(), funclet_scoped_state.value_funclet_id);

				let callee_funclet = & self.program.funclets[& callee_scheduling_funclet_id];
				assert_eq!(callee_funclet.kind, ir::FuncletKind::ScheduleExplicit);
				let callee_funclet_scheduling_extra = & self.program.scheduling_funclet_extras[& callee_scheduling_funclet_id];
				let callee_value_funclet_id = callee_funclet_scheduling_extra.value_funclet_id;
				let callee_value_funclet = & self.program.funclets[& callee_value_funclet_id];
				assert_eq!(callee_value_funclet.kind, ir::FuncletKind::Value);

				// Step 1: Check current -> callee edge
				let mut argument_slot_ids = Vec::<scheduling_state::SlotId>::new();
				for (argument_index, argument_node_id) in callee_arguments.iter().enumerate()
				{
					let slot_id = funclet_scoped_state.move_node_slot_id(* argument_node_id).unwrap();
					argument_slot_ids.push(slot_id);
					let slot_value_tag = funclet_scoped_state.slot_value_tags[& slot_id];
					check_value_tag_compatibility_enter(& self.program, value_operation, slot_value_tag, callee_funclet_scheduling_extra.input_slots[& argument_index].value_tag);
				}

				// Step 2: Check callee -> continuation edge
				for (callee_output_index, callee_output_type) in callee_funclet.output_types.iter().enumerate()
				{
					let continuation_input_index = continuation_join_point.get_capture_count() + callee_output_index;
					assert_eq!(* callee_output_type, continuation_join_point.get_scheduling_input_type(& self.program, continuation_input_index));

					let value_tag = callee_funclet_scheduling_extra.output_slots[& callee_output_index].value_tag;
					let value_tag_2 = continuation_join_point.get_scheduling_input_value_tag(& self.program, continuation_input_index);

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

				// Don't need to check continuation -> current edge because we maintain the invariant that joins can't leave the value funclet scope they were created in

				assert!(default_join_point_id_opt.is_none());
				let join_point_id = placement_state.join_graph.create(JoinPoint::SimpleJoinPoint(SimpleJoinPoint{value_funclet_id : callee_value_funclet_id, scheduling_funclet_id : callee_scheduling_funclet_id, captures : vec![].into_boxed_slice(), continuation_join_point_id}));
				return SplitPoint::Next { return_slot_ids : argument_slot_ids.into_boxed_slice(), continuation_join_point_id_opt : Some(join_point_id) };
			}
			ir::TailEdge::ScheduleSelect { value_operation, condition : condition_slot_node_id, callee_funclet_ids, callee_arguments, continuation_join : continuation_join_node_id } =>
			{
				assert_eq!(value_operation.funclet_id, funclet_scoped_state.value_funclet_id);

				let condition_slot_id = funclet_scoped_state.get_node_slot_id(* condition_slot_node_id).unwrap();

				let mut continuation_join_point_id = funclet_scoped_state.move_node_join_point_id(* continuation_join_node_id).unwrap();
				let continuation_join_point = placement_state.join_graph.get_join(continuation_join_point_id);

				assert_eq!(callee_funclet_ids.len(), 2);
				let true_funclet_id = callee_funclet_ids[0];
				let false_funclet_id = callee_funclet_ids[1];
				let true_funclet = & self.program.funclets[& true_funclet_id];
				let false_funclet = & self.program.funclets[& false_funclet_id];
				let true_funclet_extra = & self.program.scheduling_funclet_extras[& true_funclet_id];
				let false_funclet_extra = & self.program.scheduling_funclet_extras[& false_funclet_id];

				let current_value_funclet = & self.program.funclets[& value_operation.funclet_id];
				assert_eq!(current_value_funclet.kind, ir::FuncletKind::Value);

				let (true_case_node_id, false_case_node_id) = if let ir::Node::Select{condition, true_case, false_case} = & current_value_funclet.nodes[value_operation.node_id]
				{
					check_value_tag_compatibility_interior(& self.program, funclet_scoped_state.slot_value_tags[& condition_slot_id], ir::ValueTag::Operation{remote_node_id : ir::RemoteNodeId{funclet_id : value_operation.funclet_id, node_id : * condition}});
					(* true_case, * false_case)
				}
				else
				{
					panic!("Scheduling select on a node that is not a select");
				};

				assert_eq!(value_operation.funclet_id, true_funclet_extra.value_funclet_id);
				assert_eq!(value_operation.funclet_id, false_funclet_extra.value_funclet_id);

				assert_eq!(callee_arguments.len(), true_funclet.input_types.len());
				assert_eq!(callee_arguments.len(), false_funclet.input_types.len());

				let mut argument_slot_ids = Vec::<scheduling_state::SlotId>::new();
				for (argument_index, argument_node_id) in callee_arguments.iter().enumerate()
				{
					let slot_id = funclet_scoped_state.move_node_slot_id(* argument_node_id).unwrap();
					argument_slot_ids.push(slot_id);

					assert_eq!(true_funclet.input_types[argument_index], false_funclet.input_types[argument_index]);
					check_slot_type(& self.program, true_funclet.input_types[argument_index], placement_state.scheduling_state.get_slot_queue_place(slot_id), placement_state.scheduling_state.get_slot_queue_stage(slot_id), None);
					check_slot_type(& self.program, false_funclet.input_types[argument_index], placement_state.scheduling_state.get_slot_queue_place(slot_id), placement_state.scheduling_state.get_slot_queue_stage(slot_id), None);
					//assert_eq!(true_funclet_extra.input_slots[& argument_index], false_funclet.input_types[argument_index]);
					let argument_slot_value_tag = funclet_scoped_state.slot_value_tags[& slot_id];
					let true_input_value_tag = true_funclet_extra.input_slots[& argument_index].value_tag;
					let false_input_value_tag = true_funclet_extra.input_slots[& argument_index].value_tag;
					check_value_tag_compatibility_interior(& self.program, argument_slot_value_tag, true_input_value_tag);
					check_value_tag_compatibility_interior(& self.program, argument_slot_value_tag, false_input_value_tag);
				}

				let continuation_input_count = continuation_join_point.get_input_count(& self.program);
				assert_eq!(continuation_input_count, true_funclet.output_types.len());
				assert_eq!(continuation_input_count, false_funclet.output_types.len());
				for output_index in 0 .. continuation_input_count
				{
					assert_eq!(true_funclet.output_types[output_index], false_funclet.output_types[output_index]);
					let continuation_input_type = continuation_join_point.get_scheduling_input_type(& self.program, output_index);
					assert_eq!(true_funclet.output_types[output_index], continuation_input_type);
					let continuation_input_value_tag = continuation_join_point.get_scheduling_input_value_tag(& self.program, output_index);
					let true_output_value_tag = true_funclet_extra.output_slots[& output_index].value_tag;
					let false_output_value_tag = true_funclet_extra.output_slots[& output_index].value_tag;

					match continuation_input_value_tag
					{
						ir::ValueTag::Operation {remote_node_id} if remote_node_id == * value_operation =>
						{
							check_value_tag_compatibility_interior(& self.program, true_output_value_tag, ir::ValueTag::Operation{remote_node_id : ir::RemoteNodeId{funclet_id : value_operation.funclet_id, node_id : true_case_node_id}});
							check_value_tag_compatibility_interior(& self.program, false_output_value_tag, ir::ValueTag::Operation{remote_node_id : ir::RemoteNodeId{funclet_id : value_operation.funclet_id, node_id : false_case_node_id}});
						}
						_ =>
						{
							check_value_tag_compatibility_interior(& self.program, true_output_value_tag, continuation_input_value_tag);
							check_value_tag_compatibility_interior(& self.program, false_output_value_tag, continuation_input_value_tag);
						}
					}
				}

				assert!(default_join_point_id_opt.is_none());
				return SplitPoint::Select{return_slot_ids : argument_slot_ids.into_boxed_slice(), condition_slot_id, true_funclet_id, false_funclet_id, continuation_join_point_id_opt : Some(continuation_join_point_id)};
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
