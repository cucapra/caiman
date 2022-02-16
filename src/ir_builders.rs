use std::collections::HashMap;
use std::default::Default;
use crate::arena::Arena;
use crate::ir::*;

#[derive(Debug, Default)]
pub struct FuncletBuilder
{
	input_types : Vec<TypeId>,
	execution_scope : Option<Scope>,
	output_types : Vec<TypeId>,
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
		self.input_types.len() - 1
	}

	pub fn add_node(&mut self, node : Node) -> NodeId
	{
		self.nodes.push(node);
		self.nodes.len() - 1
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
		};
		self.tail_edge = Some(tail_edge);
	}

	pub fn build(mut self) -> Funclet
	{
		Funclet{input_types : self.input_types.into_boxed_slice(), execution_scope : self.execution_scope, output_types : self.output_types.into_boxed_slice(), nodes : self.nodes.into_boxed_slice(), tail_edge : self.tail_edge.unwrap()}
	}
}