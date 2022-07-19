use std::collections::{HashMap, BTreeMap};
use std::default::Default;
use crate::arena::Arena;
use crate::ir::*;


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FuncletBuilderFrameId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct FuncletBuilderFramedNodeId(FuncletBuilderFrameId, NodeId);

#[derive(Debug, Default)]
pub struct FuncletBuilder
{
	kind : Option<FuncletKind>,
	input_types : Vec<TypeId>,
	input_resource_states : Vec<BTreeMap<Place, ResourceState>>,
	output_types : Vec<TypeId>,
	output_resource_states : Vec<BTreeMap<Place, ResourceState>>,
	nodes : Vec<Node>,
	tail_edge : Option<TailEdge>,
	node_remapping : HashMap<FuncletBuilderFramedNodeId, NodeId>,
	next_frame_id : usize
}

impl FuncletBuilder
{
	pub fn new(kind : FuncletKind) -> Self
	{
		FuncletBuilder { kind : Some(kind), next_frame_id : 0, .. Default::default()}
	}

	pub fn add_input(&mut self, type_id : TypeId) -> usize
	{
		self.input_types.push(type_id);
		self.input_resource_states.push(BTreeMap::new());
		self.input_types.len() - 1
	}

	pub fn add_node(&mut self, node : Node) -> NodeId
	{
		self.nodes.push(node);
		self.nodes.len() - 1
	}
	
	pub fn place_input(&mut self, input_index : usize, place : Place, resource_state : ResourceState)
	{
		let is_new = self.input_resource_states[input_index].insert(place, resource_state).is_none();
		assert!(is_new);
	}

	pub fn place_output(&mut self, output_index : usize, place : Place, resource_state : ResourceState)
	{
		let is_new = self.output_resource_states[output_index].insert(place, resource_state).is_none();
		assert!(is_new);
	}

	pub fn create_frame(&mut self) -> FuncletBuilderFrameId
	{
		let id = self.next_frame_id;
		self.next_frame_id += 1;
		return FuncletBuilderFrameId(id);
	}

	pub fn remap_node(&mut self, frame_id : FuncletBuilderFrameId, old_node_id : NodeId, new_node_id : NodeId)
	{
		self.node_remapping.insert(FuncletBuilderFramedNodeId(frame_id, old_node_id), new_node_id);
	}

	pub fn get_remapped_node_id(& self, frame_id : FuncletBuilderFrameId, old_node_id : NodeId) -> Option<NodeId>
	{
		self.node_remapping.get(& FuncletBuilderFramedNodeId(frame_id, old_node_id)).map(|x| * x)
	}

	pub fn add_node_from_old(&mut self, frame_id : FuncletBuilderFrameId, old_node_id : NodeId, old_node : &Node) -> NodeId
	{
		let node = old_node.map_referenced_nodes(|node_id| self.node_remapping[& FuncletBuilderFramedNodeId(frame_id, node_id)]);
		let new_node_id = self.add_node(node);
		self.node_remapping.insert(FuncletBuilderFramedNodeId(frame_id, old_node_id), new_node_id);
		new_node_id
	}

	pub fn set_tail_edge(&mut self, tail_edge : TailEdge)
	{
		self.tail_edge = Some(tail_edge);
	}

	pub fn set_tail_edge_from_old(&mut self, frame_id : FuncletBuilderFrameId, old_tail_edge : &TailEdge)
	{
		let tail_edge = match old_tail_edge
		{
			TailEdge::Return{return_values} =>
			{
				let mut new_return_values = Vec::<NodeId>::new();
				for value in return_values.iter()
				{
					new_return_values.push(self.node_remapping[& FuncletBuilderFramedNodeId(frame_id, *value)]);
				}
				TailEdge::Return{return_values : new_return_values.into_boxed_slice()}
			}
			_ => todo!()
		};
		self.tail_edge = Some(tail_edge);
	}

	pub fn set_output_types(&mut self, output_types : &[TypeId])
	{
		assert_eq!(self.output_types.len(), 0);
		for & type_id in output_types.iter()
		{
			self.output_types.push(type_id);
			self.output_resource_states.push(BTreeMap::new());
		}
	}

	pub fn build(mut self) -> Funclet
	{
		Funclet{kind : self.kind.unwrap(), input_types : self.input_types.into_boxed_slice(), input_resource_states : self.input_resource_states.into_boxed_slice(), output_types : self.output_types.into_boxed_slice(), output_resource_states : self.output_resource_states.into_boxed_slice(), nodes : self.nodes.into_boxed_slice(), tail_edge : self.tail_edge.unwrap(), local_meta_variables : BTreeMap::new()}
	}
}