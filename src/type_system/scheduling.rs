use crate::ir;
use super::value_tag::*;
use super::timeline_tag::*;
use std::collections::{HashMap, BTreeMap, HashSet, BTreeSet};
use std::default::Default;



#[derive(Debug)]
pub struct JoinPoint
{
	pub in_timeline_tag : ir::TimelineTag,
	pub input_timeline_tags : Box<[ir::TimelineTag]>,
	pub input_value_tags : Box<[ir::ValueTag]>
}

#[derive(Debug)]
pub enum NodeType
{
	Slot,
	Fence,
	JoinPoint,
}

#[derive(Debug)]
pub struct FuncletChecker<'program>
{
	program : & 'program ir::Program,
	value_funclet_id : ir::FuncletId,
	value_funclet : & 'program ir::Funclet,
	scheduling_funclet : & 'program ir::Funclet,
	scheduling_funclet_extra : & 'program ir::SchedulingFuncletExtra,
	pub scalar_node_value_tags : HashMap<ir::NodeId, ir::ValueTag>,
	pub scalar_node_timeline_tags : HashMap<ir::NodeId, ir::TimelineTag>,
	//pub join_node_value_tags : HashMap<ir::NodeId, Box<[ir::ValueTag]>>,
	//pub join_node_timeline_tags : HashMap<ir::NodeId, Box<[ir::TimelineTag]>>,
	pub node_join_points : HashMap<ir::NodeId, JoinPoint>,
	pub node_types : HashMap<ir::NodeId, NodeType>,
	current_node_id : ir::NodeId,
	pub current_timeline_tag : ir::TimelineTag,
}

impl<'program> FuncletChecker<'program>
{
	pub fn new(program : & 'program ir::Program, scheduling_funclet : & 'program ir::Funclet, scheduling_funclet_extra : & 'program ir::SchedulingFuncletExtra) -> Self
	{
		assert_eq!(scheduling_funclet.kind, ir::FuncletKind::ScheduleExplicit);
		let value_funclet = & program.funclets[& scheduling_funclet_extra.value_funclet_id];
		assert_eq!(value_funclet.kind, ir::FuncletKind::Value);
		let mut state = Self
		{
			program,
			value_funclet_id : scheduling_funclet_extra.value_funclet_id,
			value_funclet,
			scheduling_funclet,
			scheduling_funclet_extra,
			scalar_node_value_tags : HashMap::new(),
			scalar_node_timeline_tags : HashMap::new(),
			//join_node_value_tags : HashMap::new(),
			//join_node_timeline_tags : HashMap::new(),
			node_join_points : HashMap::new(),
			node_types : HashMap::new(),
			current_node_id : 0,
			current_timeline_tag : scheduling_funclet_extra.in_timeline_tag
		};
		state.initialize();
		state
	}

	fn initialize(&mut self)
	{
		/*self.current_timeline_tag = match self.current_timeline_tag
		{
			ir::TimelineTag::None => ir::TimelineTag::None,
			ir::TimelineTag::Input{funclet_id, index} => ir::TimelineTag::Operation{remote_node_id : ir::RemoteNodeId{funclet_id, node_id : index}},
			ir::TimelineTag::Operation{remote_node_id} => ir::TimelineTag::Operation{remote_node_id},
			ir::TimelineTag::Output{funclet_id, index} => ir::TimelineTag::Output{funclet_id, index},
			_ => panic!("")
		};*/

		self.current_timeline_tag = concretize_input_to_internal_timeline_tag(& self.program, self.current_timeline_tag);

		for (index, input_type_id) in self.scheduling_funclet.input_types.iter().enumerate()
		{
			let is_valid = match & self.scheduling_funclet.nodes[index]
			{
				//ir::Node::None => true,
				ir::Node::Phi { .. } => true,
				_ => false
			};
			assert!(is_valid);

			let node_type = match & self.program.types[input_type_id]
			{
				ir::Type::Slot { storage_type, queue_stage, queue_place } =>
				{
					let slot_info = & self.scheduling_funclet_extra.input_slots[& index];
					let value_tag = concretize_input_to_internal_value_tag(& self.program, slot_info.value_tag);
					let timeline_tag = concretize_input_to_internal_timeline_tag(& self.program, slot_info.timeline_tag);
					self.scalar_node_value_tags.insert(index, value_tag);
					self.scalar_node_timeline_tags.insert(index, timeline_tag);
					NodeType::Slot
				}
				ir::Type::SchedulingJoin{ .. } =>
				{
					panic!("Unimplemented")
				}
				ir::Type::Fence { queue_place } =>
				{
					let fence_info = & self.scheduling_funclet_extra.input_fences[& index];
					self.scalar_node_timeline_tags.insert(index, fence_info.timeline_tag);
					NodeType::Fence
				}
				_ => panic!("Not a legal argument type for a scheduling funclet")
			};

			self.node_types.insert(index, node_type);
		}
	}

	// Ideally, this would be an iterator/generator
	fn get_funclet_input_tags(&self, funclet : & ir::Funclet, funclet_extra : & ir::SchedulingFuncletExtra) -> Box<[(ir::ValueTag, ir::TimelineTag)]>
	{
		//let funclet = & self.program.funclets[& funclet_id];
		//let funclet_extra = & self.program.scheduling_funclet_extras[& funclet_id];
		let mut output_pairs = Vec::new();
		for (index, type_id) in funclet.input_types.iter().enumerate()
		{
			// Doesn't work with joins as arguments
			let pair = match & self.program.types[type_id]
			{
				ir::Type::Slot{..} =>
				{
					let slot_info = & funclet_extra.input_slots[& index];
					(slot_info.value_tag, slot_info.timeline_tag)
				}
				ir::Type::Fence{..} =>
				{
					let fence_info = & funclet_extra.input_fences[& index];
					(ir::ValueTag::None, fence_info.timeline_tag)
				}
				_ => panic!("Unimplemented")
			};
			output_pairs.push(pair);
		}
		output_pairs.into_boxed_slice()
	}

	fn get_funclet_output_tags(&self, funclet : & ir::Funclet, funclet_extra : & ir::SchedulingFuncletExtra) -> Box<[(ir::ValueTag, ir::TimelineTag)]>
	{
		//let funclet = & self.program.funclets[& funclet_id];
		//let funclet_extra = & self.program.scheduling_funclet_extras[& funclet_id];
		let mut output_pairs = Vec::new();
		for (index, type_id) in funclet.output_types.iter().enumerate()
		{
			// Doesn't work with joins as arguments
			let pair = match & self.program.types[type_id]
			{
				ir::Type::Slot{..} =>
				{
					let slot_info = & funclet_extra.output_slots[& index];
					(slot_info.value_tag, slot_info.timeline_tag)
				}
				ir::Type::Fence{..} =>
				{
					let fence_info = & funclet_extra.output_fences[& index];
					(ir::ValueTag::None, fence_info.timeline_tag)
				}
				_ => panic!("Unimplemented")
			};
			output_pairs.push(pair);
		}
		output_pairs.into_boxed_slice()
	}

	pub fn check_next_node(&mut self, current_node_id : ir::NodeId)
	{
		assert_eq!(self.current_node_id, current_node_id);

		match & self.scheduling_funclet.nodes[current_node_id]
		{
			ir::Node::None => (),
			ir::Node::Phi { .. } => (),
			//ir::Node::ExtractResult { node_id, index } => (),
			ir::Node::AllocTemporary{ place, storage_type, operation } =>
			{
				self.scalar_node_value_tags.insert(current_node_id, ir::ValueTag::Operation{remote_node_id : * operation});
				self.scalar_node_timeline_tags.insert(current_node_id, ir::TimelineTag::None);
				self.node_types.insert(current_node_id, NodeType::Slot);
			}
			ir::Node::UnboundSlot { place, storage_type, operation } =>
			{
				self.scalar_node_value_tags.insert(current_node_id, ir::ValueTag::Operation{remote_node_id : * operation});
				self.scalar_node_timeline_tags.insert(current_node_id, ir::TimelineTag::None);
				self.node_types.insert(current_node_id, NodeType::Slot);
			}
			ir::Node::Drop { node : dropped_node_id } => (),
			ir::Node::EncodeDo { place, operation, inputs, outputs } => (),
			ir::Node::EncodeCopy { place, input, output } =>
			{
				let source_value_tag = self.scalar_node_value_tags[input];
				let destination_value_tag = self.scalar_node_value_tags[output];
				check_value_tag_compatibility_interior(& self.program, source_value_tag, destination_value_tag);
			}
			ir::Node::Submit { place, event } => (),
			ir::Node::EncodeFence { place, event } =>
			{
				self.scalar_node_timeline_tags.insert(current_node_id, self.current_timeline_tag);
				self.node_types.insert(current_node_id, NodeType::Fence);
			}
			ir::Node::SyncFence { place : synced_place, fence, event } => (),
			ir::Node::DefaultJoin =>
			{
				let mut value_tags = Vec::<ir::ValueTag>::new();
				let mut timeline_tags = Vec::<ir::TimelineTag>::new();
				/*if let Some(default_join_type_id) = self.scheduling_funclet_extra.default_join_type_id_opt
				{
					if let ir::Type::SchedulingJoin{input_types, input_slots, ..} = & self.program.types[& default_join_type_id]
					{
						for (index, input_type) in input_types.iter().enumerate()
						{
							// Doesn't work with joins as arguments
							let value_tag = match & self.program.types[input_type]
							{
								ir::Type::Slot{..} => input_slots[& index].value_tag,
								ir::Type::Fence{..} => ir::ValueTag::None,
								_ => panic!("Unimplemented")
							};
							value_tags.push(value_tag);
						}
					}
					else
					{
						panic!("Unsupported join type")
					}
				}
				else*/
				/*{
					for (index, output_type) in self.scheduling_funclet.output_types.iter().enumerate()
					{
						// Doesn't work with joins as arguments
						let (value_tag, timeline_tag) = match & self.program.types[output_type]
						{
							ir::Type::Slot{..} =>
							{
								let slot_info = & self.scheduling_funclet_extra.output_slots[& index];
								(slot_info.value_tag, slot_info.timeline_tag)
							}
							ir::Type::Fence{..} =>
							{
								let fence_info = & self.scheduling_funclet_extra.output_fences[& index];
								(ir::ValueTag::None, fence_info.timeline_tag)
							}
							_ => panic!("Unimplemented")
						};
						value_tags.push(value_tag);
						timeline_tags.push(timeline_tag);
					}
				}*/
				for (index, (value_tag, timeline_tag)) in self.get_funclet_output_tags(self.scheduling_funclet, self.scheduling_funclet_extra).iter().enumerate()
				{
					value_tags.push(* value_tag);
					timeline_tags.push(* timeline_tag);
				}
				//self.join_node_value_tags.insert(current_node_id, value_tags.into_boxed_slice());
				let join_point = JoinPoint { in_timeline_tag : self.scheduling_funclet_extra.out_timeline_tag, input_timeline_tags : timeline_tags.into_boxed_slice(), input_value_tags : value_tags.into_boxed_slice() };
				self.node_join_points.insert(current_node_id, join_point);
				self.node_types.insert(current_node_id, NodeType::JoinPoint);
			}
			ir::Node::Join { funclet : join_funclet_id, captures, continuation : continuation_join_node_id } =>
			{
				let join_funclet = & self.program.funclets[join_funclet_id];
				let join_funclet_extra = & self.program.scheduling_funclet_extras[join_funclet_id];
				let continuation_join_point = & self.node_join_points[continuation_join_node_id];

				check_timeline_tag_compatibility_interior(& self.program, join_funclet_extra.out_timeline_tag, continuation_join_point.in_timeline_tag);

				for (capture_index, capture_node_id) in captures.iter().enumerate()
				{

					match & self.program.types[& join_funclet.input_types[capture_index]]
					{
						ir::Type::Slot{..} => 
						{
							let node_value_tag = self.scalar_node_value_tags[capture_node_id];
							let node_timeline_tag = self.scalar_node_timeline_tags[capture_node_id];
							let slot_info = & join_funclet_extra.input_slots[& capture_index];

							check_value_tag_compatibility_interior(& self.program, node_value_tag, slot_info.value_tag);
							check_timeline_tag_compatibility_interior(& self.program, node_timeline_tag, slot_info.timeline_tag);
						}
						ir::Type::Fence{..} =>
						{
							let node_timeline_tag = self.scalar_node_timeline_tags[capture_node_id];
							let fence_info = & join_funclet_extra.input_fences[& capture_index];

							check_timeline_tag_compatibility_interior(& self.program, node_timeline_tag, fence_info.timeline_tag);
						}
						_ => panic!("Unimplemented")
					}
				}
			
				let mut remaining_input_value_tags = Vec::<ir::ValueTag>::new();
				let mut remaining_input_timeline_tags = Vec::<ir::TimelineTag>::new();
				for input_index in captures.len() .. join_funclet.input_types.len()
				{
					// Doesn't work with joins as arguments
					let (value_tag, timeline_tag) = match & self.program.types[& join_funclet.input_types[input_index]]
					{
						ir::Type::Slot{..} =>
						{
							let slot_info = & join_funclet_extra.input_slots[& input_index];
							(slot_info.value_tag, slot_info.timeline_tag)
						}
						ir::Type::Fence{..} =>
						{
							let fence_info = & join_funclet_extra.input_fences[& input_index];
							(ir::ValueTag::None, fence_info.timeline_tag)
						}
						_ => panic!("Unimplemented")
					};
					remaining_input_value_tags.push(value_tag);
					remaining_input_timeline_tags.push(timeline_tag);
				}

				let continuation_join_value_tags = & continuation_join_point.input_value_tags;
				let continuation_join_timeline_tags = & continuation_join_point.input_timeline_tags;

				for (join_output_index, join_output_type) in join_funclet.output_types.iter().enumerate()
				{
					match & self.program.types[& join_output_type]
					{
						ir::Type::Slot{queue_place, ..} =>
						{
							let slot_info = & join_funclet_extra.output_slots[& join_output_index];

							check_value_tag_compatibility_interior(& self.program, slot_info.value_tag, continuation_join_value_tags[join_output_index]);
							check_timeline_tag_compatibility_interior(& self.program, slot_info.timeline_tag, continuation_join_timeline_tags[join_output_index]);
						}
						ir::Type::Fence{queue_place} =>
						{
							let fence_info = & join_funclet_extra.output_fences[& join_output_index];

							check_timeline_tag_compatibility_interior(& self.program, fence_info.timeline_tag, continuation_join_timeline_tags[join_output_index]);
						}
						_ => panic!("Unimplemented")
					}
				}

				let join_point = JoinPoint { in_timeline_tag : join_funclet_extra.in_timeline_tag, input_timeline_tags : remaining_input_timeline_tags.into_boxed_slice(), input_value_tags : remaining_input_value_tags.into_boxed_slice() };
				self.node_join_points.insert(current_node_id, join_point);
				self.node_types.insert(current_node_id, NodeType::JoinPoint);
			}
			_ => panic!("Unimplemented")
		}

		self.current_node_id += 1;
	}

	pub fn check_tail_edge(&mut self)
	{
		assert_eq!(self.current_node_id, self.scheduling_funclet.nodes.len());
		match & self.scheduling_funclet.tail_edge
		{
			ir::TailEdge::Return { return_values } =>
			{
				check_timeline_tag_compatibility_interior(& self.program, self.current_timeline_tag, self.scheduling_funclet_extra.out_timeline_tag);

				for (return_index, return_node_id) in return_values.iter().enumerate()
				{
					match & self.program.types[& self.scheduling_funclet.output_types[return_index]]
					{
						ir::Type::Slot { storage_type, queue_stage, queue_place } =>
						{
							let node_timeline_tag = self.scalar_node_timeline_tags[return_node_id];
							let node_value_tag = self.scalar_node_value_tags[return_node_id];
							let slot_info = & self.scheduling_funclet_extra.output_slots[& return_index];
							let value_tag = slot_info.value_tag;
							check_value_tag_compatibility_interior(& self.program, node_value_tag, slot_info.value_tag);
							check_timeline_tag_compatibility_interior(& self.program, node_timeline_tag, slot_info.timeline_tag);
						}
						ir::Type::Fence { queue_place } =>
						{
							let node_timeline_tag = self.scalar_node_timeline_tags[return_node_id];
							let fence_info = & self.scheduling_funclet_extra.output_fences[& return_index];
							check_timeline_tag_compatibility_interior(& self.program, node_timeline_tag, fence_info.timeline_tag);
						}
						_ => panic!("Unimplemented")
					}
				}
			}
			ir::TailEdge::Jump { join, arguments } =>
			{
				let join_point = & self.node_join_points[join];
				check_timeline_tag_compatibility_interior(& self.program, self.current_timeline_tag, join_point.in_timeline_tag);


				/*let slot_value_tag = funclet_checker.scalar_node_value_tags[& arguments[argument_index]];

				for (index, (value_tag, timeline_tag)) in self.get_funclet_output_tags(self.scheduling_funclet, self.scheduling_funclet_extra).iter().enumerate()
				{
					value_tags.push(* value_tag);
					timeline_tags.push(* timeline_tag);
				}

				for (argument_index, argument_node_id) in arguments.iter().enumerate()
				{
					let slot_value_tag = funclet_checker.scalar_node_value_tags[& arguments[argument_index]];

					match * argument_node_result
					{
						NodeResult::Slot {slot_id} =>
						{
							// We need to shift the destination argument index to account for the captures (that are checked at construction)
							let destination_argument_index = argument_index + join_point.get_capture_count();
							check_value_tag_compatibility_interior(& self.program, slot_value_tag, continuation_join_value_tags[argument_index]);
							let place = placement_state.scheduling_state.get_slot_queue_place(slot_id);
							check_slot_type(& self.program, join_point.get_scheduling_input_type(& self.program, destination_argument_index), place, placement_state.scheduling_state.get_slot_queue_stage(slot_id), None);
							let logical_timestamp = placement_state.scheduling_state.get_slot_queue_timestamp(slot_id);
							//timeline_enforcer.record_slot_use(place, logical_timestamp, join_point.get_scheduling_input_external_timestamp_id(& self.program, destination_argument_index));
						}
						NodeResult::Fence { place, timestamp } =>
						{
							timeline_enforcer.record_fence_use(place, timestamp, join_point.get_scheduling_input_external_timestamp_id(& self.program, argument_index).unwrap());
						}
						_ => panic!("Unimplemented")
					}
				}*/

			}
			ir::TailEdge::ScheduleCall { value_operation : value_operation_ref, callee_funclet_id : callee_scheduling_funclet_id_ref, callee_arguments, continuation_join : continuation_join_node_id } =>
			{

			}
			ir::TailEdge::ScheduleSelect { value_operation, condition : condition_slot_node_id, callee_funclet_ids, callee_arguments, continuation_join : continuation_join_node_id } =>
			{
				
			}
			ir::TailEdge::AllocFromBuffer {buffer : buffer_node_id, slot_count, success_funclet_id, failure_funclet_id, arguments, continuation_join : continuation_join_node_id} =>
			{
				
			}
			_ => panic!("Unimplemented")
		}
	}
}

/*#[derive(Debug)]
pub struct FuncletChecker<'program>
{
	program : & 'program ir::Program,
	value_funclet_id : ir::FuncletId,
	scheduling_funclet_id : ir::FuncletId,
	value_funclet : & 'program ir::Funclet,
	scheduling_funclet : & 'program ir::Funclet,
	scheduling_funclet_extra : & 'program ir::SchedulingFuncletExtra,
	scalar_node_value_tags : HashMap<ir::NodeId, ir::ValueTag>,
	scalar_node_timeline_tags : HashMap<ir::NodeId, ir::TimelineTag>,
	join_node_value_tags : HashMap<ir::NodeId, Box<[ir::ValueTag]>>,
	join_node_timeline_tags : HashMap<ir::NodeId, Box<[ir::TimelineTag]>>,
	last_node_id : ir::NodeId
}

impl<'program> FuncletChecker<'program>
{
	pub fn new(program : & 'program ir::Program, scheduling_funclet_id : ir::FuncletId) -> Self
	{
		Self { program, scheduling_funclet_id }
	}

	pub fn check_next_node(&mut self, node_id : ir::NodeId)
	{
		self.last_node_id += 1;
		assert!(self.last_node_id, node_id);

		match & self.funclet[node_id]
		{
			ir::Node::None => (),
			ir::Node::Phi { .. } => (),
			ir::Node::ExtractResult { node_id, index } => (),
			ir::Node::AllocTemporary{ place, storage_type, operation } => (),
			ir::Node::UnboundSlot { place, storage_type, operation } => (),
			ir::Node::Drop { node : dropped_node_id } => (),
			ir::Node::EncodeDo { place, operation, inputs, outputs } => (),
			ir::Node::EncodeCopy { place, input, output } => (),
			ir::Node::Submit { place, event } => (),
			ir::Node::EncodeFence { place, event } => (),
			ir::Node::SyncFence { place : synced_place, fence, event } => (),
			ir::Node::DefaultJoin => (),
			ir::Node::Join { funclet : funclet_id, captures, continuation : continuation_join_node_id } => (),
			_ => panic!("Unimplemented")
		}
	}
}*/