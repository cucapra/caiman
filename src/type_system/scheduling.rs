use crate::ir;
use super::value_tag::*;
use super::timeline_tag::*;
use std::collections::{HashMap, BTreeMap, HashSet, BTreeSet};
use std::default::Default;


#[derive(Debug)]
pub struct Slot
{
	storage_type : ir::ffi::TypeId,
	queue_stage : ir::ResourceQueueStage,
	queue_place : ir::Place
}

#[derive(Debug)]
pub struct Fence
{
	queue_place : ir::Place
}

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
	Slot(Slot),
	Fence(Fence),
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
					NodeType::Slot(Slot{storage_type : * storage_type, queue_stage : * queue_stage, queue_place : * queue_place})
				}
				ir::Type::SchedulingJoin{ .. } =>
				{
					panic!("Unimplemented")
				}
				ir::Type::Fence { queue_place } =>
				{
					let fence_info = & self.scheduling_funclet_extra.input_fences[& index];
					self.scalar_node_timeline_tags.insert(index, fence_info.timeline_tag);
					NodeType::Fence(Fence{queue_place : * queue_place})
				}
				_ => panic!("Not a legal argument type for a scheduling funclet")
			};

			self.node_types.insert(index, node_type);
		}
	}

	fn get_funclet_input_tags(&self, funclet : & ir::Funclet, funclet_extra : & ir::SchedulingFuncletExtra, input_index : usize) -> (ir::ValueTag, ir::TimelineTag)
	{
		let type_id = funclet.input_types[input_index];
		match & self.program.types[& type_id]
		{
			ir::Type::Slot{..} =>
			{
				let slot_info = & funclet_extra.input_slots[& input_index];
				(slot_info.value_tag, slot_info.timeline_tag)
			}
			ir::Type::Fence{..} =>
			{
				let fence_info = & funclet_extra.input_fences[& input_index];
				(ir::ValueTag::None, fence_info.timeline_tag)
			}
			_ => panic!("Unimplemented")
		}
	}

	fn get_funclet_output_tags(&self, funclet : & ir::Funclet, funclet_extra : & ir::SchedulingFuncletExtra, output_index : usize) -> (ir::ValueTag, ir::TimelineTag)
	{
		let type_id = funclet.output_types[output_index];
		// Doesn't work with joins as arguments
		match & self.program.types[& type_id]
		{
			ir::Type::Slot{..} =>
			{
				let slot_info = & funclet_extra.output_slots[& output_index];
				(slot_info.value_tag, slot_info.timeline_tag)
			}
			ir::Type::Fence{..} =>
			{
				let fence_info = & funclet_extra.output_fences[& output_index];
				(ir::ValueTag::None, fence_info.timeline_tag)
			}
			_ => panic!("Unimplemented")
		}
	}

	/*fn get_value_node_inputs(&self, node : & ir::Node, place : ir::Place, input_index : usize) -> Box<[(ir::Place, ir::ValueTag)]>
	{
		let results = Vec::new();
		match node
		{
			ir::Node::
		}
		results.into_boxed_slice()
	}*/

	fn transition_slot(&mut self, slot_node_id : ir::NodeId, place : ir::Place, from_stage : ir::ResourceQueueStage, to_stage : ir::ResourceQueueStage)
	{
		let node_type = self.node_types.remove(& slot_node_id).unwrap();
		if let NodeType::Slot(Slot{storage_type, queue_stage, queue_place}) = node_type
		{
			assert_eq!(queue_place, place);
			assert_eq!(queue_stage, from_stage);
			match to_stage
			{
				ir::ResourceQueueStage::Encoded => self.scalar_node_timeline_tags.insert(slot_node_id, self.current_timeline_tag),
				ir::ResourceQueueStage::Submitted => self.scalar_node_timeline_tags.insert(slot_node_id, self.current_timeline_tag),
				_ => self.scalar_node_timeline_tags.insert(slot_node_id, ir::TimelineTag::None)
			};
			self.node_types.insert(slot_node_id, NodeType::Slot(Slot{storage_type, queue_stage : to_stage, queue_place}));
		}
		else
		{
			panic!("Not a slot");
		}
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
				self.node_types.insert(current_node_id, NodeType::Slot(Slot{storage_type : * storage_type, queue_stage : ir::ResourceQueueStage::Bound, queue_place : * place}));
			}
			ir::Node::UnboundSlot { place, storage_type, operation } =>
			{
				self.scalar_node_value_tags.insert(current_node_id, ir::ValueTag::Operation{remote_node_id : * operation});
				self.scalar_node_timeline_tags.insert(current_node_id, ir::TimelineTag::None);
				self.node_types.insert(current_node_id, NodeType::Slot(Slot{storage_type : * storage_type, queue_stage : ir::ResourceQueueStage::Unbound, queue_place : * place}));
			}
			ir::Node::Drop { node : dropped_node_id } =>
			{
				panic!("To do")
			}
			ir::Node::EncodeDo { place, operation, inputs, outputs } =>
			{
				assert_eq!(self.scheduling_funclet_extra.value_funclet_id, operation.funclet_id);

				let encoded_funclet = & self.program.funclets[& operation.funclet_id];
				let encoded_node = & encoded_funclet.nodes[operation.node_id];

				match encoded_node
				{
					ir::Node::ConstantInteger { .. } =>
					{
						assert_eq!(* place, ir::Place::Local);
						assert_eq!(inputs.len(), 0);
						assert_eq!(outputs.len(), 1);

						self.transition_slot(outputs[0], * place, ir::ResourceQueueStage::Bound, ir::ResourceQueueStage::Ready);
					}
					ir::Node::ConstantUnsignedInteger { .. } =>
					{
						assert_eq!(* place, ir::Place::Local);
						assert_eq!(inputs.len(), 0);
						assert_eq!(outputs.len(), 1);

						self.transition_slot(outputs[0], * place, ir::ResourceQueueStage::Bound, ir::ResourceQueueStage::Ready);
					}
					ir::Node::Select { condition, true_case, false_case } =>
					{
						assert_eq!(* place, ir::Place::Local);
						assert_eq!(inputs.len(), 3);
						assert_eq!(outputs.len(), 1);
		
						for (input_index, input_value_node_id) in [* condition, * true_case, * false_case].iter().enumerate()
						{
							let value_tag = self.scalar_node_value_tags[& inputs[input_index]];
							let funclet_id = self.value_funclet_id;
							check_value_tag_compatibility_interior(& self.program, value_tag, ir::ValueTag::Operation{remote_node_id : ir::RemoteNodeId{funclet_id, node_id : * input_value_node_id}});
						}

						self.transition_slot(outputs[0], * place, ir::ResourceQueueStage::Bound, ir::ResourceQueueStage::Ready);
					}
					ir::Node::CallExternalCpu { external_function_id, arguments } =>
					{
						assert_eq!(* place, ir::Place::Local);
						let function = & self.program.native_interface.external_cpu_functions[external_function_id];
						// To do: Input checks 

						for (index, output_type_id) in function.output_types.iter().enumerate()
						{
							self.transition_slot(outputs[index], * place, ir::ResourceQueueStage::Bound, ir::ResourceQueueStage::Ready);
						}
					}
					ir::Node::CallExternalGpuCompute { external_function_id, arguments, dimensions } =>
					{
						assert_eq!(* place, ir::Place::Gpu);
						let function = & self.program.native_interface.external_gpu_functions[external_function_id];
		
						assert_eq!(inputs.len(), dimensions.len() + arguments.len());
						assert_eq!(outputs.len(), function.output_types.len());

						// To do: Input checks
						
						for (index, output_type_id) in function.output_types.iter().enumerate()
						{
							self.transition_slot(outputs[index], * place, ir::ResourceQueueStage::Bound, ir::ResourceQueueStage::Encoded);
						}
					}
					_ => panic!("Cannot encode {:?}", encoded_node)
				}

				let output_is_tuple = match encoded_node
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

				if output_is_tuple
				{
					for (output_index, output_scheduling_node_id) in outputs.iter().enumerate()
					{
						let value_tag = self.scalar_node_value_tags[output_scheduling_node_id];
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
									assert_eq!(output_index, * index);
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
					assert_eq!(outputs.len(), 1);
					let value_tag = self.scalar_node_value_tags[& outputs[0]];
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

				/*for (input_index, node_id) in inputs.iter().enumerate()
				{

				}*/
				//panic!("To do")
			}
			ir::Node::EncodeCopy { place, input, output } =>
			{
				let source_value_tag = self.scalar_node_value_tags[input];
				let destination_value_tag = self.scalar_node_value_tags[output];
				check_value_tag_compatibility_interior(& self.program, source_value_tag, destination_value_tag);
				
				match place
				{
					ir::Place::Local => self.transition_slot(* output, * place, ir::ResourceQueueStage::Bound, ir::ResourceQueueStage::Ready),
					_ => self.transition_slot(* output, * place, ir::ResourceQueueStage::Bound, ir::ResourceQueueStage::Encoded)
				}

			}
			ir::Node::Submit { place, event } =>
			{
				self.current_timeline_tag = check_next_timeline_tag_on_submit(& self.program, * event, self.current_timeline_tag);

				for (node_id, node_type) in self.node_types.iter_mut()
				{
					match * node_type
					{
						NodeType::Slot(Slot{storage_type, queue_place, queue_stage}) =>
						{
							if * place == queue_place && ir::ResourceQueueStage::Encoded == queue_stage
							{
								//submitted_node_ids.push(* node_id);
								//check_timeline_tag_compatibility_interior(& self.program, , current_timeline_tag);
								// To do : move to submitted
								self.scalar_node_timeline_tags.insert(* node_id, self.current_timeline_tag);
								* node_type = NodeType::Slot(Slot{storage_type, queue_place, queue_stage : ir::ResourceQueueStage::Submitted});
							}
						}
						_ => ()
					}
				}
			}
			ir::Node::EncodeFence { place, event } =>
			{
				self.scalar_node_value_tags.insert(current_node_id, ir::ValueTag::None);
				self.scalar_node_timeline_tags.insert(current_node_id, self.current_timeline_tag);
				self.node_types.insert(current_node_id, NodeType::Fence(Fence{queue_place : * place}));
			}
			ir::Node::SyncFence { place : synced_place, fence, event } =>
			{
				self.current_timeline_tag = check_next_timeline_tag_on_sync(& self.program, * event, self.current_timeline_tag);

				// Only implemented for the local queue for now
				assert_eq!(* synced_place, ir::Place::Local);

				let fenced_place = 
					if let NodeType::Fence(Fence{queue_place}) = & self.node_types[fence]
					{
						* queue_place
					}
					else
					{
						panic!("Not a fence");
					};

				let fence_encoding_timeline_event =
					if let Some(ir::TimelineTag::Operation{remote_node_id}) = self.scalar_node_timeline_tags.remove(fence)
					{
						remote_node_id
					}
					else
					{
						panic!("Expected fence to have an operation for a timeline tag")
					};

				for (node_id, node_type) in self.node_types.iter_mut()
				{
					match * node_type
					{
						NodeType::Slot(Slot{storage_type, queue_place, queue_stage}) =>
						{
							if fenced_place == queue_place && ir::ResourceQueueStage::Submitted == queue_stage
							{
								let old_timeline_tag = self.scalar_node_timeline_tags[node_id];
								match old_timeline_tag
								{
									ir::TimelineTag::None => (),
									ir::TimelineTag::Operation{remote_node_id} =>
									{
										assert_eq!(remote_node_id.funclet_id, fence_encoding_timeline_event.funclet_id);
										if remote_node_id.node_id == fence_encoding_timeline_event.node_id
										{
											self.scalar_node_timeline_tags.remove(node_id);
											* node_type = NodeType::Slot(Slot{storage_type, queue_place, queue_stage : ir::ResourceQueueStage::Ready})
										}
									}
									_ => panic!("Not a legal timeline tag")
								}
							}
						}
						_ => ()
					}
				}
			}
			ir::Node::DefaultJoin =>
			{
				let mut value_tags = Vec::<ir::ValueTag>::new();
				let mut timeline_tags = Vec::<ir::TimelineTag>::new();
				for index in 0 .. self.scheduling_funclet.output_types.len()
				{
					let (value_tag, timeline_tag) = self.get_funclet_output_tags(self.scheduling_funclet, self.scheduling_funclet_extra, index);
					value_tags.push(value_tag);
					timeline_tags.push(timeline_tag);
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
					let node_type = self.node_types.remove(capture_node_id);
					panic!("To do: check slot type");
					let (value_tag, timeline_tag) = self.get_funclet_input_tags(join_funclet, join_funclet_extra, capture_index);
					let node_value_tag = self.scalar_node_value_tags[capture_node_id];
					let node_timeline_tag = self.scalar_node_timeline_tags[capture_node_id];

					check_value_tag_compatibility_interior(& self.program, node_value_tag, value_tag);
					check_timeline_tag_compatibility_interior(& self.program, node_timeline_tag, timeline_tag);
				}
			
				let mut remaining_input_value_tags = Vec::<ir::ValueTag>::new();
				let mut remaining_input_timeline_tags = Vec::<ir::TimelineTag>::new();
				for input_index in captures.len() .. join_funclet.input_types.len()
				{
					let (value_tag, timeline_tag) = self.get_funclet_input_tags(join_funclet, join_funclet_extra, input_index);
					remaining_input_value_tags.push(value_tag);
					remaining_input_timeline_tags.push(timeline_tag);
				}

				let continuation_join_value_tags = & continuation_join_point.input_value_tags;
				let continuation_join_timeline_tags = & continuation_join_point.input_timeline_tags;

				for (join_output_index, join_output_type) in join_funclet.output_types.iter().enumerate()
				{
					let (value_tag, timeline_tag) = self.get_funclet_output_tags(join_funclet, join_funclet_extra, join_output_index);
					check_value_tag_compatibility_interior(& self.program, value_tag, continuation_join_value_tags[join_output_index]);
					check_timeline_tag_compatibility_interior(& self.program, timeline_tag, continuation_join_timeline_tags[join_output_index]);
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
					let node_timeline_tag = self.scalar_node_timeline_tags[return_node_id];
					let node_value_tag = self.scalar_node_value_tags[return_node_id];
					let (value_tag, timeline_tag) = self.get_funclet_output_tags(self.scheduling_funclet, self.scheduling_funclet_extra, return_index);
					check_value_tag_compatibility_interior(& self.program, node_value_tag, value_tag);
					check_timeline_tag_compatibility_interior(& self.program, node_timeline_tag, timeline_tag);
				}
			}
			ir::TailEdge::Jump { join, arguments } =>
			{
				let join_point = & self.node_join_points[join];
				let join_value_tags = & join_point.input_value_tags;
				let join_timeline_tags = & join_point.input_timeline_tags;

				check_timeline_tag_compatibility_interior(& self.program, self.current_timeline_tag, join_point.in_timeline_tag);

				for (argument_index, argument_node_id) in arguments.iter().enumerate()
				{
					let node_timeline_tag = self.scalar_node_timeline_tags[argument_node_id];
					let node_value_tag = self.scalar_node_value_tags[argument_node_id];
					check_value_tag_compatibility_interior(& self.program, node_value_tag, join_value_tags[argument_index]);
					check_timeline_tag_compatibility_interior(& self.program, node_timeline_tag, join_timeline_tags[argument_index]);
				}
			}
			ir::TailEdge::ScheduleCall { value_operation : value_operation_ref, callee_funclet_id : callee_scheduling_funclet_id_ref, callee_arguments, continuation_join : continuation_join_node_id } =>
			{
				let value_operation = * value_operation_ref;
				let callee_scheduling_funclet_id = * callee_scheduling_funclet_id_ref;
				let continuation_join_point = & self.node_join_points[continuation_join_node_id];

				assert_eq!(value_operation.funclet_id, self.value_funclet_id);

				let callee_funclet = & self.program.funclets[& callee_scheduling_funclet_id];
				assert_eq!(callee_funclet.kind, ir::FuncletKind::ScheduleExplicit);
				let callee_funclet_scheduling_extra = & self.program.scheduling_funclet_extras[& callee_scheduling_funclet_id];
				let callee_value_funclet_id = callee_funclet_scheduling_extra.value_funclet_id;
				let callee_value_funclet = & self.program.funclets[& callee_value_funclet_id];
				assert_eq!(callee_value_funclet.kind, ir::FuncletKind::Value);

				check_timeline_tag_compatibility_interior(& self.program, self.current_timeline_tag, callee_funclet_scheduling_extra.in_timeline_tag);
				check_timeline_tag_compatibility_interior(& self.program, callee_funclet_scheduling_extra.out_timeline_tag, continuation_join_point.in_timeline_tag);

				// Step 1: Check current -> callee edge
				for (argument_index, argument_node_id) in callee_arguments.iter().enumerate()
				{
					let node_timeline_tag = self.scalar_node_timeline_tags[argument_node_id];
					let node_value_tag = self.scalar_node_value_tags[argument_node_id];
					let (value_tag, timeline_tag) = self.get_funclet_input_tags(callee_funclet, callee_funclet_scheduling_extra, argument_index);
					check_value_tag_compatibility_enter(& self.program, value_operation, node_value_tag, value_tag);
					check_timeline_tag_compatibility_interior(& self.program, node_timeline_tag, timeline_tag);
				}

				// Step 2: Check callee -> continuation edge
				let continuation_join_value_tags = & continuation_join_point.input_value_tags;
				let continuation_join_timeline_tags = & continuation_join_point.input_timeline_tags;
				for (callee_output_index, callee_output_type) in callee_funclet.output_types.iter().enumerate()
				{
					let (value_tag, timeline_tag) = self.get_funclet_output_tags(callee_funclet, callee_funclet_scheduling_extra, callee_output_index);

					//let intermediate_value_tag = ir::ValueTag::Operation{remote_node_id : ir::RemoteNodeId{funclet_id : value_operation.funclet_id, node_id : value_operation.node_id + 1 +  continuation_input_index}};
					//let value_tag_2 = continuation_join_value_tags[callee_output_index];

					check_value_tag_compatibility_exit(& self.program, callee_value_funclet_id, value_tag, value_operation, continuation_join_value_tags[callee_output_index]);
					//check_value_tag_compatibility_interior(& self.program, intermediate_value_tag, continuation_join_value_tags[callee_output_index]);
					check_timeline_tag_compatibility_interior(& self.program, timeline_tag, continuation_join_timeline_tags[callee_output_index]);
				}
			}
			ir::TailEdge::ScheduleSelect { value_operation, condition : condition_slot_node_id, callee_funclet_ids, callee_arguments, continuation_join : continuation_join_node_id } =>
			{
				assert_eq!(value_operation.funclet_id, self.value_funclet_id);

				let continuation_join_point = & self.node_join_points[condition_slot_node_id];

				assert_eq!(callee_funclet_ids.len(), 2);
				let true_funclet_id = callee_funclet_ids[0];
				let false_funclet_id = callee_funclet_ids[1];
				let true_funclet = & self.program.funclets[& true_funclet_id];
				let false_funclet = & self.program.funclets[& false_funclet_id];
				let true_funclet_extra = & self.program.scheduling_funclet_extras[& true_funclet_id];
				let false_funclet_extra = & self.program.scheduling_funclet_extras[& false_funclet_id];

				let current_value_funclet = & self.program.funclets[& value_operation.funclet_id];
				assert_eq!(current_value_funclet.kind, ir::FuncletKind::Value);

				let condition_value_tag = self.scalar_node_value_tags[condition_slot_node_id];

				assert_eq!(value_operation.funclet_id, true_funclet_extra.value_funclet_id);
				assert_eq!(value_operation.funclet_id, false_funclet_extra.value_funclet_id);

				assert_eq!(callee_arguments.len(), true_funclet.input_types.len());
				assert_eq!(callee_arguments.len(), false_funclet.input_types.len());

				check_timeline_tag_compatibility_interior(& self.program, self.current_timeline_tag, true_funclet_extra.in_timeline_tag);
				check_timeline_tag_compatibility_interior(& self.program, self.current_timeline_tag, false_funclet_extra.in_timeline_tag);
				check_timeline_tag_compatibility_interior(& self.program, true_funclet_extra.out_timeline_tag, continuation_join_point.in_timeline_tag);
				check_timeline_tag_compatibility_interior(& self.program, false_funclet_extra.out_timeline_tag, continuation_join_point.in_timeline_tag);

				for (argument_index, argument_node_id) in callee_arguments.iter().enumerate()
				{
					let argument_slot_value_tag = self.scalar_node_value_tags[argument_node_id];
					let argument_slot_timeline_tag = self.scalar_node_timeline_tags[argument_node_id];
					let (true_input_value_tag, true_input_timeline_tag) = self.get_funclet_input_tags(true_funclet, true_funclet_extra, argument_index);
					let (false_input_value_tag, false_input_timeline_tag) = self.get_funclet_input_tags(false_funclet, false_funclet_extra, argument_index);

					check_value_tag_compatibility_interior(& self.program, argument_slot_value_tag, true_input_value_tag);
					check_value_tag_compatibility_interior(& self.program, argument_slot_value_tag, false_input_value_tag);
					check_timeline_tag_compatibility_interior(& self.program, argument_slot_timeline_tag, true_input_timeline_tag);
					check_timeline_tag_compatibility_interior(& self.program, argument_slot_timeline_tag, false_input_timeline_tag);
				}

				let continuation_join_value_tags = & continuation_join_point.input_value_tags;
				let continuation_join_timeline_tags = & continuation_join_point.input_timeline_tags;
				assert_eq!(true_funclet.output_types.len(), false_funclet.output_types.len());
				for (output_index, _) in true_funclet.output_types.iter().enumerate()
				{
					let continuation_input_value_tag = continuation_join_value_tags[output_index];
					let (true_output_value_tag, true_output_timeline_tag) = self.get_funclet_output_tags(true_funclet, true_funclet_extra, output_index);
					let (false_output_value_tag, false_output_timeline_tag) = self.get_funclet_output_tags(false_funclet, false_funclet_extra, output_index);
					check_value_tag_compatibility_interior_branch(& self.program, * value_operation, condition_value_tag, &[true_output_value_tag, false_output_value_tag], continuation_input_value_tag);
					check_timeline_tag_compatibility_interior(& self.program, true_output_timeline_tag, continuation_join_timeline_tags[output_index]);
					check_timeline_tag_compatibility_interior(& self.program, false_output_timeline_tag, continuation_join_timeline_tags[output_index]);
				}
			}
			ir::TailEdge::AllocFromBuffer {buffer : buffer_node_id, slot_count, success_funclet_id, failure_funclet_id, arguments, continuation_join : continuation_join_node_id} =>
			{
				
			}
			_ => panic!("Unimplemented")
		}
	}
}
