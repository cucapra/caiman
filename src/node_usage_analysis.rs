use crate::ir;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BTreeSet;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum Usage
{
	LocalVariable,
	GpuBuffer,
	Extraction(usize),
	TaskSubmission(usize)
}

pub struct SubnodeUsage
{
	//user_node_ids : BTreeSet<usize>,
	pub total_usage_count : usize,
	pub usages : HashMap<Usage, usize>
}

#[derive(Clone, Copy, PartialOrd, Ord, Hash, PartialEq, Eq)]
pub struct SubnodePath(usize, Option<usize>);

// Answers the question: For a given node, how will it be used in the future?
pub struct NodeUsageAnalysis
{
	subnode_usages : HashMap<SubnodePath, SubnodeUsage>
}

impl NodeUsageAnalysis
{
	fn new() -> Self
	{
		Self { subnode_usages : HashMap::<SubnodePath, SubnodeUsage>::new() }
	}

	fn use_subnode_raw(&mut self, used_subnode_path : SubnodePath, usage : Usage)
	{
		if ! self.subnode_usages.contains_key(& used_subnode_path)
		{
			let node_usage = SubnodeUsage { total_usage_count : 0, usages : HashMap::<Usage, usize>::new() };
			self.subnode_usages.insert(used_subnode_path, node_usage);
		}

		let subnode_usage = self.subnode_usages.get_mut(& used_subnode_path).unwrap();
		subnode_usage.total_usage_count += 1;
		if let Some(usage_count) = subnode_usage.usages.get_mut(& usage)
		{
			* usage_count += 1;
		}
		else
		{
			subnode_usage.usages.insert(usage, 1);
		}
	}

	fn use_subnode(&mut self, used_subnode_path : SubnodePath, usage : Usage)
	{
		let SubnodePath(node_id, subnode_id_opt) = used_subnode_path;
		self.use_subnode_raw(SubnodePath(node_id, None), usage);
		if subnode_id_opt.is_some()
		{
			self.use_subnode_raw(SubnodePath(node_id, subnode_id_opt), usage);
		}
	}

	fn use_node(&mut self, node_id : usize, usage : Usage)
	{
		self.use_subnode(SubnodePath(node_id, None), usage)
	}

	pub fn is_subnode_used(& self, used_subnode_path : SubnodePath) -> bool
	{
		if let Some(subnode_usage) = self.subnode_usages.get(& used_subnode_path)
		{
			subnode_usage.total_usage_count > 0
		}
		else
		{
			false
		}
	}

	pub fn get_node_usages(& self, node_id : usize) -> Box<[(Usage, usize)]>
	{
		let mut usages = Vec::<(Usage, usize)>::new();

		if let Some(subnode_usage) = self.subnode_usages.get(& SubnodePath(node_id, None))
		{
			for (usage, usage_count) in subnode_usage.usages.iter()
			{
				usages.push((* usage, * usage_count));
			}
		}
		
		usages.into_boxed_slice()
	}

	pub fn is_node_used(& self, node_id : usize) -> bool
	{
		self.is_subnode_used(SubnodePath(node_id, None))
	}

	pub fn from_funclet(funclet : & ir::Funclet) -> Self
	{
		let mut analysis = Self::new();
		assert_eq!(funclet.execution_scope, Some(ir::Scope::Cpu));

		match & funclet.tail_edge
		{
			ir::TailEdge::Return { return_values } =>
			{
				for & node_id in return_values.iter()
				{
					analysis.use_node(node_id, Usage::LocalVariable);
				}
			}
		}

		for (current_node_id, node) in funclet.nodes.iter().enumerate().rev()
		{
			if ! analysis.is_node_used(current_node_id)
			{
				continue;
			}
			
			match node
			{
				ir::Node::Phi {index} => (),
				ir::Node::ExtractResult { node_id, index } =>
				{
					analysis.use_node(* node_id, Usage::Extraction(current_node_id));
				}
				ir::Node::ConstantInteger{value, type_id} => (),
				ir::Node::ConstantUnsignedInteger{value, type_id} => (),

				// Task nodes are a bit weird in that they use nodes, but will take them in any state
				// They just decide what state they are in afterwards
				// The forward pass gets to decide which state they are in initially
				// As such, these rules aren't really correct since they propagate the synchronization requirement to outside
				ir::Node::GpuTaskStart{local_variable_node_ids, gpu_resident_node_ids} =>
				{
					for & node_id in local_variable_node_ids.iter()
					{
						analysis.use_node(node_id, Usage::LocalVariable)
					}

					for & node_id in gpu_resident_node_ids.iter()
					{
						analysis.use_node(node_id, Usage::GpuBuffer)
					}
				}
				ir::Node::GpuTaskEnd{task_node_id, local_variable_node_ids, gpu_resident_node_ids} =>
				{
					analysis.use_node(* task_node_id, Usage::TaskSubmission(current_node_id));

					for & node_id in local_variable_node_ids.iter()
					{
						analysis.use_node(node_id, Usage::LocalVariable)
					}

					for & node_id in gpu_resident_node_ids.iter()
					{
						analysis.use_node(node_id, Usage::GpuBuffer)
					}
				}

				ir::Node::CallExternalCpu { external_function_id, arguments } =>
				{
					for & node_id in arguments.iter()
					{
						analysis.use_node(node_id, Usage::LocalVariable)
					}
				}
				ir::Node::CallExternalGpuCompute {external_function_id, arguments, dimensions} =>
				{
					for & node_id in dimensions.iter()
					{
						analysis.use_node(node_id, Usage::LocalVariable)
					}

					for & node_id in arguments.iter()
					{
						analysis.use_node(node_id, Usage::GpuBuffer)
					}
				}
				_ =>
				{
					panic!("Unimplemented node")
				}
			}
		}
		
		analysis
	}
}