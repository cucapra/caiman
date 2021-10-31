use crate::ir;
use crate::analyses;

mod split_scope
{
	use crate::ir;
	use crate::analyses;
	use std::collections::HashMap;
	
	// Compute node accessibility
	// Identify all callexternal nodes where accessibility does not meet requirements and legalize them with callgpucoordinator nodes

	// NodeState encodes a subset of the type system relevant to scopes
	enum NodeState
	{
		SingleResult ( ir::Scope ),
		MultipleResult ( ir::Scope )
	}

	struct FuncletState
	{
		scope : ir::Scope,
	}

	struct FuncletReplacement
	{
		
	}

	struct SplitState
	{
		funclet_states : HashMap<ir::FuncletId, FuncletState>,
	}

	fn resolve_scopes(old_scope_opt : Option<ir::Scope>, new_scope_opt : Option<ir::Scope>) -> Option<ir::Scope>
	{
		if let Some(new_scope) = new_scope_opt
		{
			match old_scope_opt
			{
				Some(old_scope) =>
				{
					/*// For now, we resolve to CPU when there's ambiguity
					let scope = match (old_scope, new_scope)
					{
						(ir::Scope::Cpu, _) => ir::Scope::Cpu,
						(_, ir::Scope::Cpu) => ir::Scope::Cpu,
						(_, _) => new_scope
					};
					Some(scope)*/
					// Resolve to None when there's ambiguity to allow backwards pass to make a choice
					if old_scope == new_scope
					{
						new_scope_opt
					}
					else
					{
						None
					}
				}
				None => new_scope_opt
			}
		}
		else
		{
			old_scope_opt
		}
	}

	impl SplitState
	{

		fn inference(&mut self, funclet : &ir::Funclet, funclet_scope : ir::Scope)
		{
			// None means that it's not known where the operation should be computed
			let mut forward_node_scopes = Vec::<Option<ir::Scope>>::new();

			// Forward inference
			// In forwards inference, we resolve all ambiguities towards the most restrictive scope
			for node_index in 0 .. funclet.nodes.len()
			{
				let node = & funclet.nodes[node_index];
				let node_scope = match node
				{
					ir::Node::Phi { index } => Some(funclet_scope),
					ir::Node::ComputedResult { node_ids } =>
					{
						let mut resolved_scope : Option<ir::Scope> = None;
						for &node_id in node_ids.iter()
						{
							resolved_scope = resolve_scopes(resolved_scope, forward_node_scopes[node_id]);
						}
						resolved_scope
					}
					ir::Node::ExtractResult { node_id, index } => forward_node_scopes[*node_id],
					ir::Node::CallExternalCpu { external_function_id, arguments } => Some(ir::Scope::Cpu),
					ir::Node::CallExternalGpuCompute { external_function_id, arguments, dimensions } => Some(ir::Scope::Gpu),
					ir::Node::CallGpuCoordinator { funclet_id, arguments } => Some(ir::Scope::Cpu),
					_ => panic!("Unimplemented")
				};

				forward_node_scopes.push(node_scope);
			}

			// Backward inference
			/*for node_index in (0 .. funclet.nodes.len()).rev()
			{
				let node = & funclet.nodes[node_index];
				let node_scope = node_scopes[node_index];
				match node
				{
					ir::Node::Phi { index } => Some(funclet_scope),
					ir::Node::ComputedResult { node_ids } => None,
					ir::Node::ExtractResult { node_id, index } => node_scopes[*node_id],
					ir::Node::CallExternalCpu { external_function_id, arguments } => Some(ir::Scope::Cpu),
					ir::Node::CallExternalGpuCompute { external_function_id, arguments, dimensions } => Some(ir::Scope::Gpu),
					ir::Node::CallGpuCoordinator { funclet_id, arguments } => Some(ir::Scope::Cpu),
					_ => panic!("Unimplemented")
				};
			}*/

			let mut backward_node_scopes = Vec::<Option<ir::Scope>>::new();
			for _ in 0 .. funclet.nodes.len()
			{
				backward_node_scopes.push(None);
			}

			// Backward inference
			// In backwards inference, we resolve all ambiguities towards the least restrictive scope
			for node_index in (0 .. funclet.nodes.len()).rev()
			{
				let node = & funclet.nodes[node_index];
				let backward_node_scope = backward_node_scopes[node_index];
				//let forward_node_scope = forward_node_scopes[node_index];
				match node
				{
					ir::Node::Phi { index } => Some(funclet_scope),
					ir::Node::ComputedResult { node_ids } => None,
					ir::Node::ExtractResult { node_id, index } => backward_node_scopes[*node_id],
					ir::Node::CallExternalCpu { external_function_id, arguments } => Some(ir::Scope::Cpu),
					ir::Node::CallExternalGpuCompute { external_function_id, arguments, dimensions } => Some(ir::Scope::Gpu),
					ir::Node::CallGpuCoordinator { funclet_id, arguments } => Some(ir::Scope::Cpu),
					_ => panic!("Unimplemented")
				};
			}
		}

		fn simple_split(&mut self, funclet : &ir::Funclet, funclet_scope : ir::Scope)
		{
			for node_index in (0 .. funclet.nodes.len()).rev()
			{

			}
		}
	}

	fn split_funclet(funclet : &ir::Funclet)
	{

	}
}