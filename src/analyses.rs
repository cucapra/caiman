// Basically just type inference for the scope functors
mod scope_propagation
{
	use crate::ir;
	//
	//pub types : HashMap<usize, Type>,
	enum NodeState
	{
		SingleResult ( ir::ScopeSet ),
		MultipleResult ( Box<[ir::ScopeSet]> )
	}

	pub struct FuncletState<'funclet>
	{
		//new_types : HashMap<usize, Type>,
		funclet : & 'funclet ir::Funclet,
		node_states : Box<[NodeState]>,
		output_scopes : Box<[ir::ScopeSet]>
	}

	impl<'funclet> FuncletState<'funclet>
	{
		pub fn new<'funclets, F>(funclet : & 'funclet ir::Funclet, input_scopes : &[ir::ScopeSet], get_funclet_output_scopes : F) -> Self
			where F : Fn(ir::FuncletId) -> & 'funclets [ir::ScopeSet]
		{
			let mut node_states = Vec::<NodeState>::new();
			
			for node in funclet.nodes.iter()
			{
				//use crate::ir::{Node, ScopeSet};
				let node_state = match node
				{
					ir::Node::Phi { index } => NodeState::SingleResult ( input_scopes[*index] ),
					ir::Node::ExtractResult { node_id, index } =>
					{
						let scope = match & node_states[*node_id]
						{
							NodeState::SingleResult ( scope ) => *scope,
							NodeState::MultipleResult ( scopes ) => scopes[*node_id]
						};

						NodeState::SingleResult ( scope )
					}
					//Node::ReadBuffer { node_id, type_id, byte_offset } => node_scope_sets[*node_id],
					ir::Node::CallExternalCpu { external_function_id, arguments } => NodeState::SingleResult ( ir::ScopeSet::Cpu ),
					ir::Node::CallExternalGpuCompute { external_function_id, arguments, dimensions } => NodeState::SingleResult ( ir::ScopeSet::Gpu ),
					ir::Node::CallGpuCoordinator { funclet_id, arguments } => NodeState::SingleResult ( ir::ScopeSet::Cpu ),
					_ => NodeState::SingleResult ( ir::ScopeSet::empty() ) //CallExternalCpu { _ }
				};
				node_states.push(node_state);
			}

			let mut output_scopes = Vec::<ir::ScopeSet>::new();
			match & funclet.tail_edge
			{
				ir::TailEdge::Return { return_values } =>
				{
					for node_id in return_values.iter()
					{
						let scope = match & node_states[*node_id]
						{
							NodeState::SingleResult ( scope ) => *scope,
							NodeState::MultipleResult ( scopes ) =>
							{
								panic!("Cannot convert multiple results to single");
								ir::ScopeSet::empty()
							}
						};

						output_scopes.push(scope);
					}
				}
			}
	
			//panic!("Unfinished function");
			Self { funclet, node_states : node_states.into_boxed_slice(), output_scopes : output_scopes.into_boxed_slice() }
		}
	}
}