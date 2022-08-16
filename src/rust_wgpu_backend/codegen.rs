use crate::ir;
use crate::shadergen;
use crate::arena::Arena;
use std::default::Default;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BTreeSet;
use std::collections::BTreeMap;
use crate::rust_wgpu_backend::code_generator;
use crate::rust_wgpu_backend::code_generator::CodeGenerator;
use crate::rust_wgpu_backend::code_generator::SubmissionId;
use std::fmt::Write;
use std::collections::BinaryHeap;
use std::cmp::Reverse;
use crate::type_system::value_tag::*;
use crate::type_system::timeline_tag::*;
use crate::type_system;

use crate::scheduling_state;
use crate::scheduling_state::{LogicalTimestamp};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JoinPointId(usize);

#[derive(Debug, Clone)]
enum NodeResult
{
	None,
	Slot { slot_id : scheduling_state::SlotId, queue_place : ir::Place, storage_type : ir::ffi::TypeId },
	Fence { place : ir::Place, timestamp : LogicalTimestamp, fence_id : code_generator::FenceId },
	Join{ join_point_id : JoinPointId},
}

impl NodeResult
{
	fn get_slot_id(&self) -> Option<scheduling_state::SlotId>
	{
		if let NodeResult::Slot{ slot_id, .. } = self
		{
			Some(* slot_id)
		}
		else
		{
			None
		}
	}
}

#[derive(Debug, Clone)]
struct RootJoinPoint
{
	value_funclet_id : ir::FuncletId,
	input_types : Box<[ir::TypeId]>,
}

#[derive(Debug, Clone)]
struct SimpleJoinPoint
{
	value_funclet_id : ir::FuncletId,
	scheduling_funclet_id : ir::FuncletId,
	captures : Box<[NodeResult]>,
	continuation_join_point_id : JoinPointId
}

#[derive(Debug)]
enum JoinPoint
{
	RootJoinPoint(RootJoinPoint),
	SimpleJoinPoint(SimpleJoinPoint),
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

	fn get_slot_var_id(&self, slot_id : scheduling_state::SlotId) -> Option<usize>
	{
		self.slot_variable_ids.get(& slot_id).map(|x| * x)
	}

	fn get_node_result_var_id(&self, node_result : &NodeResult) -> Option<usize>
	{
		match node_result
		{
			NodeResult::Slot{slot_id, ..} => self.get_slot_var_id(* slot_id),
			_ => None
		}
	}

	fn get_node_result_var_ids(&self, node_results : &[NodeResult]) -> Option<Box<[usize]>>
	{
		let mut var_ids = Vec::<usize>::new();
		for node_result in node_results.iter()
		{	
			if let Some(var_id) = self.get_node_result_var_id(node_result)
			{
				var_ids.push(var_id)
			}
			else
			{
				return None;
			}
		}
		Some(var_ids.into_boxed_slice())
	}
}

#[derive(Debug)]
enum SplitPoint
{
	Next { return_node_results : Box<[NodeResult]>, continuation_join_point_id_opt : Option<JoinPointId> },
	Select { return_node_results : Box<[NodeResult]>, condition_slot_id : scheduling_state::SlotId, true_funclet_id : ir::FuncletId, false_funclet_id : ir::FuncletId, continuation_join_point_id_opt : Option<JoinPointId> }
}

#[derive(Debug)]
struct FuncletScopedState
{
	value_funclet_id : ir::FuncletId,
	scheduling_funclet_id : ir::FuncletId,
	node_results : HashMap<ir::NodeId, NodeResult>,
}

impl FuncletScopedState
{
	fn new(value_funclet_id : ir::FuncletId, scheduling_funclet_id : ir::FuncletId) -> Self
	{
		Self{ value_funclet_id, scheduling_funclet_id, node_results : Default::default()}
	}

	fn move_node_result(&mut self, node_id : ir::NodeId) -> Option<NodeResult>
	{
		self.node_results.remove(& node_id)
	}

	fn get_node_result(& self, node_id : ir::NodeId) -> Option<&NodeResult>
	{
		self.node_results.get(& node_id)
	}

	fn get_node_slot_id(&self, node_id : ir::NodeId) -> Option<scheduling_state::SlotId>
	{
		match & self.node_results[& node_id]
		{
			NodeResult::Slot{slot_id, ..} => Some(* slot_id),
			_ => None
		}
	}

	fn move_node_slot_id(&mut self, node_id : ir::NodeId) -> Option<scheduling_state::SlotId>
	{
		let slot_id_opt = match & self.node_results[& node_id]
		{
			NodeResult::Slot{slot_id, ..} => Some(* slot_id),
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

fn get_slot_type_storage_type(program : & ir::Program, type_id : ir::TypeId) -> ir::ffi::TypeId
{
	match & program.types[& type_id]
	{
		ir::Type::Slot { storage_type, queue_stage : _, queue_place : _ } =>
		{
			* storage_type
		}
		_ => panic!("Not a slot type")
	}
}

fn check_storage_type_implements_value_type(program : & ir::Program, storage_type_id : ir::ffi::TypeId, value_type_id : ir::TypeId)
{
	let storage_type = & program.native_interface.types[& storage_type_id.0];
	let value_type = & program.types[& value_type_id];
	/*match value_type
	{
		ir::Type::Integer{signed, width} =>
		{
			()
		}
		_ => panic!("Unsupported")
	}*/
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
		Self { program : & program, code_generator : CodeGenerator::new(& program.native_interface), print_codegen_debug_info : false }
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

	fn encode_do_node_gpu(&mut self, placement_state : &mut PlacementState, funclet_scoped_state : &mut FuncletScopedState, funclet_checker : &mut type_system::scheduling::FuncletChecker, node : & ir::Node, input_slot_ids : & [scheduling_state::SlotId], output_slot_ids : & [scheduling_state::SlotId])
	{
		match node
		{
			ir::Node::CallExternalGpuCompute {external_function_id, arguments, dimensions} =>
			{
				let function = & self.program.native_interface.external_gpu_functions[external_function_id];

				assert_eq!(input_slot_ids.len(), dimensions.len() + arguments.len());
				assert_eq!(output_slot_ids.len(), function.output_types.len());

				/*let mut input_slot_counts = HashMap::<scheduling_state::SlotId, usize>::from_iter(input_slot_ids.iter().chain(output_slot_ids.iter()).map(|slot_id| (* slot_id, 0usize)));
				let mut output_slot_bindings = HashMap::<scheduling_state::SlotId, Option<usize>>::from_iter(output_slot_ids.iter().map(|slot_id| (* slot_id, None)));
				for (binding_index, resource_binding) in function.resource_bindings.iter().enumerate()
				{
					if let Some(index) = resource_binding.input
					{
						* input_slot_counts.get_mut(& input_slot_ids[index]).unwrap() += 1;
					}

					if let Some(index) = resource_binding.output
					{
						* output_slot_bindings.get_mut(& output_slot_ids[index]).unwrap() = Some(binding_index);
					}
				}

				for (binding_index, resource_binding) in function.resource_bindings.iter().enumerate()
				{
					if let Some(output_index) = resource_binding.output
					{
						let output_slot_id = output_slot_ids[output_index];
						assert_eq!(input_slot_counts[& output_slot_id], 0);
						assert_eq!(output_slot_bindings[& output_slot_id], Some(binding_index));

						if let Some(input_index) = resource_binding.input
						{
							let input_slot_id = input_slot_ids[input_index];
							assert_eq!(input_slot_counts[& input_slot_id], 1);

							assert_eq!(placement_state.scheduling_state.get_slot_type_id(input_slot_id), placement_state.scheduling_state.get_slot_type_id(output_slot_id));

							placement_state.scheduling_state.forward_slot(output_slot_id, input_slot_id);
							let var_id = placement_state.slot_variable_ids.remove(& input_slot_id).unwrap();
							let old = placement_state.slot_variable_ids.insert(output_slot_id, var_id);
							assert!(old.is_none());
						}
					}
				}*/


				for (input_index, _) in arguments.iter().enumerate()
				{
					if let Some(forwarded_output_index) = function.output_of_forwarding_input(input_index)
					{
						let input_slot_id = input_slot_ids[dimensions.len() + input_index];
						let output_slot_id = output_slot_ids[forwarded_output_index];
						//placement_state.scheduling_state.forward_slot(output_slot_id, input_slot_id);
						let var_id = placement_state.slot_variable_ids[& input_slot_id];
						let old = placement_state.slot_variable_ids.insert(output_slot_id, var_id);
						assert!(old.is_none());
					}
				}

				use std::convert::TryInto;
				use std::iter::FromIterator;
				//use core::slice::SlicePattern;
				let dimension_map = |(index, x)| 
				{
					let slot_id = input_slot_ids[index];
					placement_state.slot_variable_ids[& slot_id]
				};
				let argument_map = |(index, x)|
				{
					let slot_id = input_slot_ids[dimensions.len() + index];
					placement_state.slot_variable_ids[& slot_id]
				};
				let dimension_var_ids = Vec::from_iter(dimensions.iter().enumerate().map(dimension_map)).into_boxed_slice();
				let argument_var_ids = Vec::from_iter(arguments.iter().enumerate().map(argument_map)).into_boxed_slice();
				let output_var_ids = output_slot_ids.iter().map(|x| placement_state.get_slot_var_id(* x).unwrap()).collect::<Box<[usize]>>();

				let dimensions_slice : &[usize] = & dimension_var_ids;
				//let raw_outputs = self.code_generator.build_compute_dispatch(* external_function_id, dimensions_slice.try_into().expect("Expected 3 elements for dimensions"), & argument_var_ids);
				self.code_generator.build_compute_dispatch_with_outputs(* external_function_id, dimensions_slice.try_into().expect("Expected 3 elements for dimensions"), & argument_var_ids, & output_var_ids);

				for (index, output_type_id) in function.output_types.iter().enumerate()
				{
					let slot_id = output_slot_ids[index];
					placement_state.update_slot_state(slot_id, ir::ResourceQueueStage::Encoded, output_var_ids[index]);
				}
			}
			_ => panic!("Node cannot be encoded to the gpu")
		}
	}

	fn encode_do_node_local(&mut self, placement_state : &mut PlacementState, funclet_scoped_state : &mut FuncletScopedState, funclet_checker : &mut type_system::scheduling::FuncletChecker, node : & ir::Node, input_slot_ids : & [scheduling_state::SlotId], output_slot_ids : &[scheduling_state::SlotId], output_slot_node_ids : & [ir::NodeId])
	{
		// To do: Do something about the value
		match node
		{
			ir::Node::ConstantInteger{value, type_id} =>
			{
				let slot_id = output_slot_ids[0];
				let storage_type_id =
					if let Some(NodeResult::Slot{storage_type, ..}) = funclet_scoped_state.get_node_result(output_slot_node_ids[0])
					{
						* storage_type
					}
					else
					{
						panic!("Not a slot");
					};
				//let storage_type_id = placement_state.scheduling_state.get_slot_type_id(slot_id);
				let variable_id = self.code_generator.build_constant_integer(* value, storage_type_id);
				check_storage_type_implements_value_type(& self.program, storage_type_id, * type_id);

				placement_state.update_slot_state(slot_id, ir::ResourceQueueStage::Ready, variable_id);
			}
			ir::Node::ConstantUnsignedInteger{value, type_id} =>
			{
				let slot_id = output_slot_ids[0];
				let storage_type_id =
					if let Some(NodeResult::Slot{storage_type, ..}) = funclet_scoped_state.get_node_result(output_slot_node_ids[0])
					{
						* storage_type
					}
					else
					{
						panic!("Not a slot");
					};
				//let storage_type_id = placement_state.scheduling_state.get_slot_type_id(slot_id);
				let variable_id = self.code_generator.build_constant_unsigned_integer(* value, storage_type_id);
				check_storage_type_implements_value_type(& self.program, storage_type_id, * type_id);

				placement_state.update_slot_state(slot_id, ir::ResourceQueueStage::Ready, variable_id);
			}
			ir::Node::Select { condition, true_case, false_case } =>
			{
				let input_var_ids = input_slot_ids.iter().map(|& slot_id| placement_state.get_slot_var_id(slot_id).unwrap()).collect::<Box<[usize]>>();

				let slot_id = output_slot_ids[0];
				let variable_id = self.code_generator.build_select_hack(input_var_ids[0], input_var_ids[1], input_var_ids[2]);

				placement_state.update_slot_state(slot_id, ir::ResourceQueueStage::Ready, variable_id);
			}
			ir::Node::CallExternalCpu { external_function_id, arguments } =>
			{
				let function = & self.program.native_interface.external_cpu_functions[external_function_id];

				use std::iter::FromIterator;

				let argument_var_ids = Vec::from_iter(arguments.iter().enumerate().map(|(index, x)| { let slot_id = input_slot_ids[index]; placement_state.slot_variable_ids[& slot_id] })).into_boxed_slice();
				let raw_outputs = self.code_generator.build_external_cpu_function_call(* external_function_id, & argument_var_ids);

				for (index, output_type_id) in function.output_types.iter().enumerate()
				{
					let slot_id = output_slot_ids[index];
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
		let funclet_extra = & self.program.scheduling_funclet_extras[& funclet_id];

		let input_types = funclet.input_types.iter().map(|slot_id| get_slot_type_storage_type(& self.program, * slot_id)).collect::<Box<[ir::ffi::TypeId]>>();
		let output_types = funclet.output_types.iter().map(|slot_id| get_slot_type_storage_type(& self.program, * slot_id)).collect::<Box<[ir::ffi::TypeId]>>();
		let argument_variable_ids = self.code_generator.begin_funclet(funclet_id, & input_types, & output_types);

		let mut placement_state = PlacementState::new();
		let mut argument_node_results = Vec::<NodeResult>::new();
		
		for (index, input_type_id) in funclet.input_types.iter().enumerate()
		{
			let result = 
			{
				use ir::Type;
				
				match & self.program.types[input_type_id]
				{
					ir::Type::Slot { storage_type, queue_stage, queue_place } =>
					{
						let slot_id = placement_state.scheduling_state.insert_hacked_slot(* storage_type, * queue_place, * queue_stage);
						placement_state.slot_variable_ids.insert(slot_id, argument_variable_ids[index]);
						argument_node_results.push(NodeResult::Slot{slot_id, queue_place : * queue_place, storage_type : * storage_type});
					}
					ir::Type::Fence { queue_place } =>
					{
						
					}
					_ => panic!("Unimplemented")
				}
			};
		}

		let mut default_join_point_id_opt = 
		{
			let input_types = funclet.output_types.clone();
			let value_funclet_id = funclet_extra.value_funclet_id;
			let join_point_id = placement_state.join_graph.create(JoinPoint::RootJoinPoint(RootJoinPoint{value_funclet_id, input_types}));
			Option::<JoinPointId>::Some(join_point_id)
		};

		enum TraversalState
		{
			SelectIf { branch_input_node_results : Box<[NodeResult]>, condition_slot_id : scheduling_state::SlotId, true_funclet_id : ir::FuncletId, false_funclet_id : ir::FuncletId, continuation_join_point_id_opt : Option<JoinPointId> },
			SelectElse { output_node_results : Box<[NodeResult]>, branch_input_node_results : Box<[NodeResult]>, false_funclet_id : ir::FuncletId, continuation_join_point_id_opt : Option<JoinPointId> },
			SelectEnd { output_node_results : Box<[NodeResult]>, continuation_join_point_id_opt : Option<JoinPointId> },
		}

		let mut traversal_state_stack = Vec::<TraversalState>::new();

		let mut current_output_node_results = argument_node_results.into_boxed_slice();
		let mut current_funclet_id_opt = Some(funclet_id);

		while current_funclet_id_opt.is_some()
		{
			while let Some(current_funclet_id) = current_funclet_id_opt
			{
				let split_point = self.compile_scheduling_funclet(current_funclet_id, & current_output_node_results, pipeline_context, &mut placement_state, default_join_point_id_opt);
				println!("Split point: {:?}", split_point);
				current_output_node_results = match split_point
				{
					SplitPoint::Next{return_node_results, continuation_join_point_id_opt} =>
					{
						default_join_point_id_opt = continuation_join_point_id_opt;
						return_node_results
					}
					SplitPoint::Select{return_node_results, condition_slot_id, true_funclet_id, false_funclet_id, continuation_join_point_id_opt} =>
					{
						//assert!(default_join_point_id_opt.is_none());
						traversal_state_stack.push(TraversalState::SelectIf{ branch_input_node_results : return_node_results, condition_slot_id, true_funclet_id, false_funclet_id, continuation_join_point_id_opt });
						vec![].into_boxed_slice()
					}
				};

				if default_join_point_id_opt.is_none()
				{
					while let Some(traversal_state) = traversal_state_stack.pop()
					{
						match traversal_state
						{
							TraversalState::SelectIf { branch_input_node_results, condition_slot_id, true_funclet_id, false_funclet_id, continuation_join_point_id_opt } =>
							{
								let condition_var_id = placement_state.get_slot_var_id(condition_slot_id).unwrap();
								let true_funclet = & self.program.funclets[& true_funclet_id];
								let true_funclet_extra = & self.program.scheduling_funclet_extras[& true_funclet_id];
								//true_funclet_extra.input_slots[& output_index].value_tag
								let output_types = true_funclet.output_types.iter().map(|slot_id| get_slot_type_storage_type(& self.program, * slot_id)).collect::<Box<[ir::ffi::TypeId]>>();
								let output_var_ids = self.code_generator.begin_if_else(condition_var_id, & output_types);
								let mut output_node_results = Vec::<NodeResult>::new();
								for (output_index, output_type) in true_funclet.output_types.iter().enumerate()
								{
									let (storage_type, queue_stage, queue_place) = if let ir::Type::Slot{storage_type, queue_stage, queue_place} = & self.program.types[output_type]
									{
										(* storage_type, * queue_stage, * queue_place)
									}
									else
									{
										panic!("Not a slot")
									};
									let slot_id = placement_state.scheduling_state.insert_hacked_slot(storage_type, queue_place, queue_stage);
									output_node_results.push(NodeResult::Slot{slot_id, queue_place, storage_type});
								}
								current_funclet_id_opt = Some(true_funclet_id);
								current_output_node_results = branch_input_node_results.clone();
								traversal_state_stack.push(TraversalState::SelectElse{output_node_results : output_node_results.into_boxed_slice(), branch_input_node_results, false_funclet_id, continuation_join_point_id_opt});
							}
							TraversalState::SelectElse { output_node_results, branch_input_node_results, false_funclet_id, continuation_join_point_id_opt } =>
							{
								self.code_generator.end_if_begin_else(& placement_state.get_node_result_var_ids(& current_output_node_results).unwrap());
								current_funclet_id_opt = Some(false_funclet_id);
								current_output_node_results = branch_input_node_results;
								traversal_state_stack.push(TraversalState::SelectEnd{output_node_results, continuation_join_point_id_opt});
							}
							TraversalState::SelectEnd { output_node_results, continuation_join_point_id_opt } =>
							{
								self.code_generator.end_else(& placement_state.get_node_result_var_ids(& current_output_node_results).unwrap());
								default_join_point_id_opt = continuation_join_point_id_opt;
								current_output_node_results = output_node_results;
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
							let return_var_ids = placement_state.get_node_result_var_ids(& current_output_node_results).unwrap();
							self.code_generator.build_return(& return_var_ids);
						}
						JoinPoint::SimpleJoinPoint(simple_join_point) =>
						{
							let mut input_node_results = Vec::<NodeResult>::new();
							input_node_results.extend_from_slice(& simple_join_point.captures);
							input_node_results.extend_from_slice(& current_output_node_results);
							
							current_funclet_id_opt = Some(simple_join_point.scheduling_funclet_id);
							default_join_point_id_opt = Some(simple_join_point.continuation_join_point_id);
							current_output_node_results = input_node_results.into_boxed_slice();
						}
						_ => panic!("Jump to invalid join point #{:?}: {:?}", join_point_id, join_point)
					}

					println!("{:?} {:?} {:?}", current_funclet_id_opt, default_join_point_id_opt, current_output_node_results);
				}
			}

			assert!(current_funclet_id_opt.is_none());
		}

		self.code_generator.end_funclet();
	}

	fn compile_scheduling_funclet(&mut self, funclet_id : ir::FuncletId, argument_node_results : &[NodeResult], /*argument_slot_ids : &[scheduling_state::SlotId],*/ pipeline_context : &mut PipelineContext, placement_state : &mut PlacementState, mut default_join_point_id_opt : Option<JoinPointId>) -> SplitPoint //Box<[scheduling_state::SlotId]>
	{
		let funclet = & self.program.funclets[& funclet_id];
		assert_eq!(funclet.kind, ir::FuncletKind::ScheduleExplicit);
		let funclet_scheduling_extra = & self.program.scheduling_funclet_extras[& funclet_id];
		//let scheduled_value_funclet = & self.program.value_funclets[& scheduling_funclet.value_funclet_id];

		let mut funclet_scoped_state = FuncletScopedState::new(funclet_scheduling_extra.value_funclet_id, funclet_id);
		let mut funclet_checker = type_system::scheduling::FuncletChecker::new(& self.program, funclet, funclet_scheduling_extra);

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

			funclet_checker.check_next_node(current_node_id);

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
				ir::Node::AllocTemporary{ place, storage_type, operation } =>
				{
					assert_eq!(funclet_scheduling_extra.value_funclet_id, operation.funclet_id);

					let slot_id = placement_state.scheduling_state.insert_hacked_slot(* storage_type, * place, ir::ResourceQueueStage::Bound);
					funclet_scoped_state.node_results.insert(current_node_id, NodeResult::Slot{slot_id, queue_place : * place, storage_type : * storage_type});

					// To do: Allocate from buffers for GPU/CPU and assign variable
					match place
					{
						ir::Place::Cpu => (),
						ir::Place::Local => (),
						ir::Place::Gpu =>
						{
							let var_id = self.code_generator.build_create_buffer(* storage_type);
							placement_state.update_slot_state(slot_id, ir::ResourceQueueStage::Bound, var_id);
						}
					}
				}
				ir::Node::UnboundSlot { place, storage_type, operation } =>
				{
					assert_eq!(funclet_scheduling_extra.value_funclet_id, operation.funclet_id);

					let slot_id = placement_state.scheduling_state.insert_hacked_slot(* storage_type, * place, ir::ResourceQueueStage::Unbound);
					funclet_scoped_state.node_results.insert(current_node_id, NodeResult::Slot{slot_id, queue_place : * place, storage_type : * storage_type});
				}
				ir::Node::Drop { node : dropped_node_id } =>
				{
					funclet_scoped_state.move_node_result(* dropped_node_id);
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

					match place
					{
						ir::Place::Local =>
						{
							self.encode_do_node_local(placement_state, &mut funclet_scoped_state, &mut funclet_checker, encoded_node, input_slot_ids.as_slice(), output_slot_ids.as_slice(), outputs);
						}
						ir::Place::Gpu =>
						{
							self.encode_do_node_gpu(placement_state, &mut funclet_scoped_state, &mut funclet_checker, encoded_node, input_slot_ids.as_slice(), output_slot_ids.as_slice());
						}
						ir::Place::Cpu => (),
					}
				}
				ir::Node::EncodeCopy { place, input, output } =>
				{
					let src_slot_id = funclet_scoped_state.get_node_slot_id(* input).unwrap();
					let dst_slot_id = funclet_scoped_state.get_node_slot_id(* output).unwrap();

					let (src_slot_id, src_place) =
						if let Some(NodeResult::Slot{slot_id, queue_place, ..}) = funclet_scoped_state.get_node_result(* input)
						{
							(* slot_id, * queue_place)
						}
						else
						{
							panic!("Not a slot")
						};

					let (dst_slot_id, dst_place) =
						if let Some(NodeResult::Slot{slot_id, queue_place, ..}) = funclet_scoped_state.get_node_result(* output)
						{
							(* slot_id, * queue_place)
						}
						else
						{
							panic!("Not a slot")
						};

					match (* place, dst_place, src_place)
					{
						(ir::Place::Local, ir::Place::Local, ir::Place::Local) =>
						{
							let src_var_id = placement_state.get_slot_var_id(src_slot_id).unwrap();
							placement_state.update_slot_state(dst_slot_id, ir::ResourceQueueStage::Ready, src_var_id);
						}
						(ir::Place::Local, ir::Place::Local, ir::Place::Gpu) =>
						{
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
				ir::Node::Submit { place, event } =>
				{
					let submission_id = placement_state.scheduling_state.insert_submission
					(
						* place,
						&mut |scheduling_state, event| ()
					);

					placement_state.submission_map.insert(submission_id, self.code_generator.flush_submission());
				}
				ir::Node::EncodeFence { place, event } =>
				{
					let local_timestamp = self.advance_local_time(placement_state);
					let fence_id = self.code_generator.encode_fence();
					funclet_scoped_state.node_results.insert(current_node_id, NodeResult::Fence { place : * place, timestamp : local_timestamp, fence_id });
				}
				ir::Node::SyncFence { place : synced_place, fence, event } =>
				{
					let local_timestamp = self.advance_local_time(placement_state);
					// Only implemented for the local queue for now
					assert_eq!(* synced_place, ir::Place::Local);

					if let Some(NodeResult::Fence{place : fenced_place, timestamp, fence_id}) = funclet_scoped_state.move_node_result(* fence)
					{
						self.code_generator.sync_gpu_fence(fence_id);

						assert_eq!(fenced_place, ir::Place::Gpu);

						if let Some(newer_timestamp) = self.advance_known_place_time(placement_state, fenced_place, timestamp)
						{
							panic!("Have already synced to a later time")
						}
					}
					else
					{
						panic!("Expected fence")
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
					let mut captured_node_results = Vec::<NodeResult>::new();
					let join_funclet = & self.program.funclets[funclet_id];
					let extra = & self.program.scheduling_funclet_extras[funclet_id];

					// Join points can only be constructed for the value funclet they are created in
					assert_eq!(extra.value_funclet_id, funclet_scoped_state.value_funclet_id);

					for (capture_index, capture_node_id) in captures.iter().enumerate()
					{
						let node_result = funclet_scoped_state.move_node_result(* capture_node_id).unwrap();
						captured_node_results.push(node_result);
					}

					let continuation_join_point_id = funclet_scoped_state.move_node_join_point_id(* continuation_join_node_id).unwrap();
					let continuation_join_point = placement_state.join_graph.get_join(continuation_join_point_id);

					let join_point_id = placement_state.join_graph.create(JoinPoint::SimpleJoinPoint(SimpleJoinPoint{value_funclet_id : extra.value_funclet_id, scheduling_funclet_id : * funclet_id, captures : captured_node_results.into_boxed_slice(), continuation_join_point_id}));
					funclet_scoped_state.node_results.insert(current_node_id, NodeResult::Join{ join_point_id });
				}
				_ => panic!("Unknown node")
			};
		}

		if self.print_codegen_debug_info
		{
			println!("{:?} : {:?} {:?}", funclet.tail_edge, placement_state, funclet_scoped_state);
		}

		funclet_checker.check_tail_edge();

		self.code_generator.insert_comment(format!(" tail edge: {:?}", funclet.tail_edge).as_str());
		let split_point = match & funclet.tail_edge
		{
			ir::TailEdge::Return { return_values } =>
			{
				let encoded_value_funclet_id = funclet_scheduling_extra.value_funclet_id;
				let encoded_value_funclet = & self.program.funclets[& encoded_value_funclet_id];

				let mut output_node_results = Vec::<NodeResult>::new();

				for (return_index, return_node_id) in return_values.iter().enumerate()
				{
					let node_result = funclet_scoped_state.move_node_result(* return_node_id).unwrap();
					output_node_results.push(node_result);
				}

				SplitPoint::Next{return_node_results : output_node_results.into_boxed_slice(), continuation_join_point_id_opt : default_join_point_id_opt}
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

				let mut argument_node_results = Vec::<NodeResult>::new();

				for (argument_index, argument_node_id) in arguments.iter().enumerate()
				{
					let node_result = funclet_scoped_state.move_node_result(* argument_node_id).unwrap();
					argument_node_results.push(node_result);
				}

				assert!(default_join_point_id_opt.is_none());
				SplitPoint::Next { return_node_results : argument_node_results.into_boxed_slice(), continuation_join_point_id_opt : Some(join_point_id)}
			}
			ir::TailEdge::ScheduleCall { value_operation : value_operation_ref, callee_funclet_id : callee_scheduling_funclet_id_ref, callee_arguments, continuation_join : continuation_join_node_id } =>
			{
				let value_operation = * value_operation_ref;
				let callee_scheduling_funclet_id = * callee_scheduling_funclet_id_ref;

				let continuation_join_point_id = funclet_scoped_state.move_node_join_point_id(* continuation_join_node_id).unwrap();
				let continuation_join_point = placement_state.join_graph.get_join(continuation_join_point_id);

				assert_eq!(value_operation.funclet_id, funclet_scoped_state.value_funclet_id);
				//assert_eq!(continuation_join_point.get_value_funclet_id(), funclet_scoped_state.value_funclet_id);

				let callee_funclet = & self.program.funclets[& callee_scheduling_funclet_id];
				assert_eq!(callee_funclet.kind, ir::FuncletKind::ScheduleExplicit);
				let callee_funclet_scheduling_extra = & self.program.scheduling_funclet_extras[& callee_scheduling_funclet_id];
				let callee_value_funclet_id = callee_funclet_scheduling_extra.value_funclet_id;
				let callee_value_funclet = & self.program.funclets[& callee_value_funclet_id];
				assert_eq!(callee_value_funclet.kind, ir::FuncletKind::Value);

				let mut argument_node_results = Vec::<NodeResult>::new();
				for (argument_index, argument_node_id) in callee_arguments.iter().enumerate()
				{
					let node_result = funclet_scoped_state.move_node_result(* argument_node_id).unwrap();
					argument_node_results.push(node_result);
				}

				assert!(default_join_point_id_opt.is_none());
				let join_point_id = placement_state.join_graph.create(JoinPoint::SimpleJoinPoint(SimpleJoinPoint{value_funclet_id : callee_value_funclet_id, scheduling_funclet_id : callee_scheduling_funclet_id, captures : vec![].into_boxed_slice(), continuation_join_point_id}));
				SplitPoint::Next { return_node_results : argument_node_results.into_boxed_slice(), continuation_join_point_id_opt : Some(join_point_id) }
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

				assert_eq!(value_operation.funclet_id, true_funclet_extra.value_funclet_id);
				assert_eq!(value_operation.funclet_id, false_funclet_extra.value_funclet_id);

				assert_eq!(callee_arguments.len(), true_funclet.input_types.len());
				assert_eq!(callee_arguments.len(), false_funclet.input_types.len());

				let mut argument_node_results = Vec::<NodeResult>::new();
				for (argument_index, argument_node_id) in callee_arguments.iter().enumerate()
				{
					let node_result = funclet_scoped_state.move_node_result(* argument_node_id).unwrap();
					argument_node_results.push(node_result);
				}

				assert!(default_join_point_id_opt.is_none());
				SplitPoint::Select{return_node_results : argument_node_results.into_boxed_slice(), condition_slot_id, true_funclet_id, false_funclet_id, continuation_join_point_id_opt : Some(continuation_join_point_id)}
			}
			ir::TailEdge::AllocFromBuffer {buffer : buffer_node_id, slot_count, success_funclet_id, failure_funclet_id, arguments, continuation_join : continuation_join_node_id} =>
			{
				// To do: Yet more type checks in every other tail edge + join...
				// Probably need to clean things up soon...
				// Value/fence checks are uninteresting because it's a space split and not a time or value split, so we could share and avoid the product explosion

				/*let buffer_node_result = funclet_scoped_state.get_node_result(* buffer_node_id);
				//let (buffer_storage_place) = if let NodeResult::Buffer{} = {} else {};
				let buffer_storage_place = ir::Place::Gpu; // not correct
				panic!("Unimplemented");

				let mut continuation_join_point_id = funclet_scoped_state.move_node_join_point_id(* continuation_join_node_id).unwrap();
				let continuation_join_point = placement_state.join_graph.get_join(continuation_join_point_id);

				let true_funclet_id = success_funclet_id;
				let false_funclet_id = failure_funclet_id;
				let true_funclet = & self.program.funclets[& true_funclet_id];
				let false_funclet = & self.program.funclets[& false_funclet_id];
				let true_funclet_extra = & self.program.scheduling_funclet_extras[& true_funclet_id];
				let false_funclet_extra = & self.program.scheduling_funclet_extras[& false_funclet_id];

				assert_eq!(funclet_scoped_state.value_funclet_id, true_funclet_extra.value_funclet_id);
				assert_eq!(funclet_scoped_state.value_funclet_id, false_funclet_extra.value_funclet_id);

				assert_eq!(arguments.len(), true_funclet.input_types.len() + slot_count);
				assert_eq!(arguments.len(), false_funclet.input_types.len());

				let mut true_entry_timeline_enforcer = TimelineEnforcer::new();
				let mut false_entry_timeline_enforcer = TimelineEnforcer::new();
				let mut argument_node_results = Vec::<NodeResult>::new();
				for (argument_index, argument_node_id) in arguments.iter().enumerate()
				{
					let node_result = funclet_scoped_state.move_node_result(* argument_node_id).unwrap();
					match node_result
					{
						NodeResult::Slot{slot_id} =>
						{
							assert_eq!(true_funclet.input_types[argument_index], false_funclet.input_types[argument_index]);
							let place = placement_state.scheduling_state.get_slot_queue_place(slot_id);
							check_slot_type(& self.program, true_funclet.input_types[argument_index], place, placement_state.scheduling_state.get_slot_queue_stage(slot_id), None);
							check_slot_type(& self.program, false_funclet.input_types[argument_index], place, placement_state.scheduling_state.get_slot_queue_stage(slot_id), None);
							//assert_eq!(true_funclet_extra.input_slots[& argument_index], false_funclet.input_types[argument_index]);
							let argument_slot_value_tag = funclet_checker.scalar_node_value_tags[argument_node_id];
							let true_slot_info = & true_funclet_extra.input_slots[& argument_index];
							let true_input_value_tag = true_slot_info.value_tag;
							let false_slot_info = & false_funclet_extra.input_slots[& argument_index];
							let false_input_value_tag = false_slot_info.value_tag;
							check_value_tag_compatibility_interior(& self.program, argument_slot_value_tag, true_input_value_tag);
							check_value_tag_compatibility_interior(& self.program, argument_slot_value_tag, false_input_value_tag);
							let timestamp = placement_state.scheduling_state.get_slot_queue_timestamp(slot_id);
							//true_entry_timeline_enforcer.record_slot_use(place, timestamp, true_slot_info.external_timestamp_id_opt);
							//false_entry_timeline_enforcer.record_slot_use(place, timestamp, false_slot_info.external_timestamp_id_opt);
						}
						NodeResult::Fence{place, timestamp} =>
						{
							//true_entry_timeline_enforcer.record_fence_use(place, timestamp, true_funclet_extra.input_fences[& argument_index].external_timestamp_id);
							//false_entry_timeline_enforcer.record_fence_use(place, timestamp, false_funclet_extra.input_fences[& argument_index].external_timestamp_id);
						}
						_ => panic!("Unimplemented")
					}

					argument_node_results.push(node_result);
				}

				for input_index in arguments.len() .. arguments.len() + slot_count
				{
					let input_type = true_funclet.input_types[input_index];
					match & self.program.types[& input_type]
					{
						ir::Type::Slot{storage_type, queue_stage, queue_place} =>
						{
							assert_eq!(* queue_stage, ir::ResourceQueueStage::Bound);
							assert_eq!(* queue_place, buffer_storage_place);
						}
						_ => panic!("Must be a slot")
					}
				}

				let continuation_join_value_tags = & funclet_checker.join_node_value_tags[continuation_join_node_id];
				let mut true_exit_timeline_enforcer = ExternalTimelineEnforcer::new();
				let mut false_exit_timeline_enforcer = ExternalTimelineEnforcer::new();
				let continuation_input_count = continuation_join_point.get_input_count(& self.program);
				assert_eq!(continuation_input_count, true_funclet.output_types.len());
				assert_eq!(continuation_input_count, false_funclet.output_types.len());
				for output_index in 0 .. continuation_input_count
				{
					assert_eq!(true_funclet.output_types[output_index], false_funclet.output_types[output_index]);
					let continuation_input_type = continuation_join_point.get_scheduling_input_type(& self.program, output_index);
					assert_eq!(true_funclet.output_types[output_index], continuation_input_type);

					match & self.program.types[& output_index]
					{
						ir::Type::Slot{queue_place, ..} =>
						{
							let continuation_input_value_tag = continuation_join_value_tags[output_index];
							let true_slot_info = & true_funclet_extra.output_slots[& output_index];
							let false_slot_info = & false_funclet_extra.output_slots[& output_index];
							let true_output_value_tag = true_slot_info.value_tag;
							let false_output_value_tag = false_slot_info.value_tag;

							let external_timestamp_id_opt = continuation_join_point.get_scheduling_input_external_timestamp_id(& self.program, output_index);
							//true_exit_timeline_enforcer.record_slot_use(* queue_place, true_slot_info.external_timestamp_id_opt, external_timestamp_id_opt);
							//false_exit_timeline_enforcer.record_slot_use(* queue_place, false_slot_info.external_timestamp_id_opt, external_timestamp_id_opt);

							check_value_tag_compatibility_interior(& self.program, true_output_value_tag, continuation_input_value_tag);
							check_value_tag_compatibility_interior(& self.program, false_output_value_tag, continuation_input_value_tag);
						}
						ir::Type::Fence{queue_place} =>
						{
							let external_timestamp_id = continuation_join_point.get_scheduling_input_external_timestamp_id(& self.program, output_index).unwrap();
							//true_exit_timeline_enforcer.record_fence_use(* queue_place, true_funclet_extra.output_fences[& output_index].external_timestamp_id, external_timestamp_id);
							//false_exit_timeline_enforcer.record_fence_use(* queue_place, false_funclet_extra.output_fences[& output_index].external_timestamp_id, external_timestamp_id);
						}
						_ => panic!("Unimplemented")
					}
				}

				assert!(default_join_point_id_opt.is_none());
				//SplitPoint::Select{return_node_results : argument_node_results.into_boxed_slice(), condition_slot_id, true_funclet_id, false_funclet_id, continuation_join_point_id_opt : Some(continuation_join_point_id)}*/
				panic!("Unimplemented")
			}
			_ => panic!("Umimplemented")
		};

		split_point
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

		let input_types = entry_funclet.input_types.iter().map(|slot_id| get_slot_type_storage_type(& self.program, * slot_id)).collect::<Box<[ir::ffi::TypeId]>>();
		let output_types = entry_funclet.output_types.iter().map(|slot_id| get_slot_type_storage_type(& self.program, * slot_id)).collect::<Box<[ir::ffi::TypeId]>>();
		self.code_generator.emit_pipeline_entry_point(entry_funclet_id, & input_types, & output_types);
		
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
