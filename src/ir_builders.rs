use std::collections::{HashMap, BTreeMap};
use std::default::Default;
use crate::arena::Arena;
use crate::ir::*;

#[derive(Debug, Default)]
pub struct FuncletBuilder
{
	input_types : Vec<TypeId>,
	input_resource_states : Vec<BTreeMap<Place, ResourceState>>,
	execution_scope : Option<Scope>,
	output_types : Vec<TypeId>,
	output_resource_states : Vec<BTreeMap<Place, ResourceState>>,
	nodes : Vec<Node>,
	tail_edge : Option<TailEdge>,
	node_remapping : HashMap<NodeId, NodeId>
}

impl FuncletBuilder
{
	pub fn new() -> Self
	{
		Default::default()
	}

	pub fn new_with_execution_scope(execution_scope : Scope) -> Self
	{
		let mut builder = Self::new();
		builder.execution_scope = Some(execution_scope);
		builder
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

	pub fn get_remapped_node_id(& self, old_node_id : NodeId) -> Option<NodeId>
	{
		self.node_remapping.get(& old_node_id).map(|x| * x)
	}

	pub fn add_node_from_old(&mut self, old_node_id : NodeId, old_node : &Node) -> NodeId
	{
		let node = match old_node
		{
			Node::Phi {index} =>
			{
				// Might want to rethink this
				Node::Phi{index : *index}
			}
			Node::ExtractResult { node_id, index } =>
			{
				Node::ExtractResult{node_id : self.node_remapping[node_id], index : *index}
			}
			Node::ConstantInteger{value, type_id} =>
			{
				Node::ConstantInteger{value : *value, type_id : *type_id}
			}
			Node::ConstantUnsignedInteger{value, type_id} =>
			{
				Node::ConstantUnsignedInteger{value : *value, type_id : *type_id}
			}
			Node::CallExternalCpu { external_function_id, arguments } =>
			{
				let mut new_arguments = Vec::<NodeId>::new();
				for argument in arguments.iter()
				{
					new_arguments.push(self.node_remapping[argument]);
				}
				Node::CallExternalCpu{external_function_id : *external_function_id, arguments : new_arguments.into_boxed_slice()}
			}
			Node::CallExternalGpuCompute {external_function_id, arguments, dimensions} =>
			{
				let mut new_arguments = Vec::<NodeId>::new();
				for argument in arguments.iter()
				{
					new_arguments.push(self.node_remapping[argument]);
				}
				let mut new_dimensions = Vec::<NodeId>::new();
				for dimension in dimensions.iter()
				{
					new_dimensions.push(self.node_remapping[dimension]);
				}
				Node::CallExternalGpuCompute{external_function_id : *external_function_id, arguments : new_arguments.into_boxed_slice(), dimensions : new_dimensions.into_boxed_slice()}
			}
			Node::EncodeGpu{values} =>
			{
				let mut new_values = Vec::<NodeId>::new();
				for value in values.iter()
				{
					new_values.push(self.node_remapping[value]);
				}
				Node::EncodeGpu{values : new_values.into_boxed_slice()}
			}
			Node::SubmitGpu{values} =>
			{
				let mut new_values = Vec::<NodeId>::new();
				for value in values.iter()
				{
					new_values.push(self.node_remapping[value]);
				}
				Node::SubmitGpu{values : new_values.into_boxed_slice()}
			}
			Node::SyncLocal{values} =>
			{
				let mut new_values = Vec::<NodeId>::new();
				for value in values.iter()
				{
					new_values.push(self.node_remapping[value]);
				}
				Node::SubmitGpu{values : new_values.into_boxed_slice()}
			}
			_ => panic!("Unknown node")
		};
		let new_node_id = self.add_node(node);
		self.node_remapping.insert(old_node_id, new_node_id);
		new_node_id
	}

	pub fn set_tail_edge(&mut self, tail_edge : TailEdge)
	{
		self.tail_edge = Some(tail_edge);
	}

	pub fn set_tail_edge_from_old(&mut self, old_tail_edge : &TailEdge)
	{
		let tail_edge = match old_tail_edge
		{
			TailEdge::Return{return_values} =>
			{
				let mut new_return_values = Vec::<NodeId>::new();
				for value in return_values.iter()
				{
					new_return_values.push(self.node_remapping[value]);
				}
				TailEdge::Return{return_values : new_return_values.into_boxed_slice()}
			}
			TailEdge::Yield { funclet_ids, captured_arguments, return_values } =>
			{
				let mut new_captured_arguments = Vec::<NodeId>::new();
				for value in captured_arguments.iter()
				{
					new_captured_arguments.push(self.node_remapping[value]);
				}

				let mut new_return_values = Vec::<NodeId>::new();
				for value in return_values.iter()
				{
					new_return_values.push(self.node_remapping[value]);
				}

				TailEdge::Yield { funclet_ids : funclet_ids.clone(), captured_arguments : new_captured_arguments.into_boxed_slice(), return_values : new_return_values.into_boxed_slice() }
			}
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
		Funclet{input_types : self.input_types.into_boxed_slice(), input_resource_states : self.input_resource_states.into_boxed_slice(), execution_scope : self.execution_scope, output_types : self.output_types.into_boxed_slice(), output_resource_states : self.output_resource_states.into_boxed_slice(), nodes : self.nodes.into_boxed_slice(), tail_edge : self.tail_edge.unwrap()}
	}
}