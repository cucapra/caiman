use crate::ir;
use crate::ir_builders;

use crate::shadergen;
use crate::arena::Arena;
use std::default::Default;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BTreeSet;
use std::collections::BTreeMap;
use crate::rust_wgpu_backend::code_generator::CodeGenerator;
use std::fmt::Write;
use crate::node_usage_analysis::*;

#[derive(Debug, Clone, Copy)]
enum GpuResidencyState
{
	Useable,
	Encoded,
	Submitted
}

#[derive(Debug, Default)]
struct NodeResourceTracker
{
	registered_node_set : HashSet<ir::NodeId>,
	//deferred_node_dependencies : HashMap<ir::NodeId, BTreeSet<ir::NodeId>>,
	proxy_node_map : HashMap::<ir::NodeId, ir::NodeId>,
	active_encoding_node_set : BTreeSet::<ir::NodeId>,
	//submitted_node_map : HashMap::<ir::NodeId, ir::NodeId>,
	node_gpu_residency_state : HashMap<ir::NodeId, GpuResidencyState>,
	locally_resident_node_set : HashSet<ir::NodeId>
}

impl NodeResourceTracker
{
	fn new() -> Self
	{
		Default::default()
	}

	fn register_proxy_node(&mut self, node_id : ir::NodeId, proxied_node_id : ir::NodeId)
	{
		let was_newly_registered = self.registered_node_set.insert(node_id);
		assert!(was_newly_registered);
		let was_newly_proxy = self.proxy_node_map.insert(node_id, proxied_node_id).is_none();
		assert!(was_newly_proxy);
	}

	fn register_passthrough_node(&mut self, node_id : ir::NodeId, passthrough_node_id : ir::NodeId)
	{
		let was_newly_registered = self.registered_node_set.insert(node_id);
		assert!(was_newly_registered);

		if self.proxy_node_map.contains_key(& passthrough_node_id)
		{
			let is_new = self.proxy_node_map.insert(node_id, self.proxy_node_map[& passthrough_node_id]).is_none();
			assert!(is_new);
		}

		if self.active_encoding_node_set.contains(& passthrough_node_id)
		{
			let is_new = self.active_encoding_node_set.insert(node_id);
			assert!(is_new);
		}

		if self.node_gpu_residency_state.contains_key(& passthrough_node_id)
		{
			let is_new = self.node_gpu_residency_state.insert(node_id, self.node_gpu_residency_state[& passthrough_node_id]).is_none();
			assert!(is_new);
		}

		if self.locally_resident_node_set.contains(& passthrough_node_id)
		{
			let is_new = self.locally_resident_node_set.insert(node_id);
			assert!(is_new);
		}


	}

	/*fn add_deferred_node_dependencies(&mut self, node_ids : &[ir::NodeId], dependency_node_ids : &[ir::NodeId])
	{
		for & dependency_node_id in dependency_node_ids.iter()
		{
			for & node_id in node_ids.iter()
			{
				assert!(dependency_node_id < node_id);
				if ! self.deferred_node_dependencies.contains_key(& node_id)
				{
					
				}
			}
		}
	}*/

	fn register_local_nodes(&mut self, node_ids : &[ir::NodeId])
	{
		for & node_id in node_ids.iter()
		{
			let was_newly_registered = self.registered_node_set.insert(node_id);
			assert!(was_newly_registered);
			let was_newly_local = self.locally_resident_node_set.insert(node_id);
			assert!(was_newly_local);
		}
	}

	fn register_gpu_encoded_nodes(&mut self, node_ids : &[ir::NodeId])
	{
		for & node_id in node_ids.iter()
		{
			let was_newly_registered = self.registered_node_set.insert(node_id);
			assert!(was_newly_registered);
			let was_newly_encoded = self.active_encoding_node_set.insert(node_id);
			assert!(was_newly_encoded);
			/*let was_newly_gpu = self.gpu_resident_node_set.insert(node_id);
			assert!(was_newly_gpu);*/
			let was_newly_gpu_resident = self.node_gpu_residency_state.insert(node_id, GpuResidencyState::Encoded);
			assert!(was_newly_gpu_resident.is_none());
		}
	}

	fn register_gpu_submitted_nodes(&mut self, node_ids : &[ir::NodeId])
	{
		for & node_id in node_ids.iter()
		{
			let was_newly_registered = self.registered_node_set.insert(node_id);
			assert!(was_newly_registered);
			/*let was_newly_gpu = self.gpu_resident_node_set.insert(node_id);
			assert!(was_newly_gpu);*/
			let was_newly_gpu_resident = self.node_gpu_residency_state.insert(node_id, GpuResidencyState::Submitted);
			assert!(was_newly_gpu_resident.is_none());
		}
	}

	fn register_gpu_ready_nodes(&mut self, node_ids : &[ir::NodeId])
	{
		for & node_id in node_ids.iter()
		{
			let was_newly_registered = self.registered_node_set.insert(node_id);
			assert!(was_newly_registered);
			/*let was_newly_gpu = self.gpu_resident_node_set.insert(node_id);
			assert!(was_newly_gpu);*/
			let was_newly_gpu_resident = self.node_gpu_residency_state.insert(node_id, GpuResidencyState::Useable);
			assert!(was_newly_gpu_resident.is_none());
		}
	}

	fn transition_gpu(&mut self, node_ids : &[ir::NodeId], funclet_builder : &mut ir_builders::FuncletBuilder, min_required_state : GpuResidencyState)
	{
		let mut should_submit = false;
		let mut should_encode = false;
		let mut should_sync = false;
		match min_required_state
		{
			GpuResidencyState::Useable =>
			{
				should_submit = true;
				should_encode = true;
				should_sync = true;
			}
			GpuResidencyState::Submitted =>
			{
				should_submit = true;
				should_encode = true;
				should_sync = false;
			}
			GpuResidencyState::Encoded =>
			{
				should_submit = false;
				should_encode = true;
				should_sync = false;
			}
		}

		let mut encoded_node_depedencies = Vec::<ir::NodeId>::new();
		let mut local_node_depedencies = Vec::<ir::NodeId>::new();
		let mut submitted_node_dependencies = Vec::<ir::NodeId>::new();
		//let mut collateral_encoded_node_dependencies = Vec::<ir::NodeId>::new();
		let mut node_dependency_set = HashSet::<ir::NodeId>::new();
		let mut sync_node_dependencies = Vec::<ir::NodeId>::new();

		let mut frontier_node_ids = Vec::<ir::NodeId>::new();
		for & node_id in node_ids.iter().rev()
		{
			frontier_node_ids.push(node_id);
		}

		//for & node_id in node_ids.iter()
		while let Some(node_id) = frontier_node_ids.pop()
		{
			if node_dependency_set.contains(& node_id)
			{
				continue;
			}

			node_dependency_set.insert(node_id);

			assert!(self.registered_node_set.contains(& node_id));

			if self.proxy_node_map.contains_key(& node_id)
			{
				frontier_node_ids.push(self.proxy_node_map[& node_id]);
			}

			let is_locally_resident = self.locally_resident_node_set.contains(& node_id);
			/*let is_gpu_resident = self.gpu_resident_node_set.contains(& node_id);
			let is_locally_resident = self.locally_resident_node_set.contains(& node_id);
			assert!(is_gpu_resident || is_locally_resident); // This will probably change eventually
			if ! is_gpu_resident
			{
				assert!(is_locally_resident);
				local_node_depedencies.push(node_id);
			}
			if self.encoding_node_set.contains(& node_id)
			{
				encoded_node_depedencies.push(node_id);
			}*/

			let gpu_residency_state = & self.node_gpu_residency_state.get(& node_id);
			match gpu_residency_state
			{
				None =>
				{
					if is_locally_resident
					{
						local_node_depedencies.push(node_id);
						encoded_node_depedencies.push(node_id);
					}
				}
				Some(GpuResidencyState::Useable) =>
				{
					// Nothing to do!
				}
				Some(GpuResidencyState::Encoded) =>
				{
					assert!(self.active_encoding_node_set.contains(& node_id));
					encoded_node_depedencies.push(node_id);
					sync_node_dependencies.push(node_id);
				}
				Some(GpuResidencyState::Submitted) =>
				{
					submitted_node_dependencies.push(node_id);
					sync_node_dependencies.push(node_id);
				}
			}
		}

		if should_encode && local_node_depedencies.len() > 0
		{
			for & node_id in local_node_depedencies.iter()
			{
				self.node_gpu_residency_state.insert(node_id, GpuResidencyState::Encoded);
			}

			funclet_builder.add_node(ir::Node::EncodeGpu{values : local_node_depedencies.into_boxed_slice()});
		}

		let mut has_collateral_encoded_nodes = false;
		if encoded_node_depedencies.len() > 0
		{
			has_collateral_encoded_nodes = true;
			for & node_id in self.active_encoding_node_set.iter()
			{
				if node_dependency_set.contains(& node_id)
				{
					continue;
				}

				//collateral_encoded_node_dependencies.push(node_id);
				encoded_node_depedencies.push(node_id);
			}
		}

		if should_submit && encoded_node_depedencies.len() > 0
		{
			for & node_id in encoded_node_depedencies.iter()
			{
				self.node_gpu_residency_state.insert(node_id, GpuResidencyState::Submitted);
			}

			if has_collateral_encoded_nodes
			{
				self.active_encoding_node_set.clear();
			}
			else
			{
				for & node_id in encoded_node_depedencies.iter()
				{
					self.active_encoding_node_set.remove(& node_id);
				}
			}

			funclet_builder.add_node(ir::Node::SubmitGpu{values : encoded_node_depedencies.into_boxed_slice()});
		}

		if should_sync && sync_node_dependencies.len() > 0
		{
			for & node_id in sync_node_dependencies.iter()
			{
				self.node_gpu_residency_state.insert(node_id, GpuResidencyState::Useable);
			}
			funclet_builder.add_node(ir::Node::SyncLocal{values : sync_node_dependencies.into_boxed_slice()});
		}
	}

	fn encode_gpu(&mut self, node_ids : &[ir::NodeId], funclet_builder : &mut ir_builders::FuncletBuilder)
	{
		self.transition_gpu(node_ids, funclet_builder, GpuResidencyState::Encoded);
	}

	fn submit_gpu(&mut self, node_ids : &[ir::NodeId], funclet_builder : &mut ir_builders::FuncletBuilder)
	{
		self.transition_gpu(node_ids, funclet_builder, GpuResidencyState::Submitted);
	}

	fn sync_local(&mut self, node_ids : &[ir::NodeId], funclet_builder : &mut ir_builders::FuncletBuilder)
	{
		let mut gpu_resident_node_dependencies = Vec::<ir::NodeId>::new();
		
		let mut frontier_node_ids = Vec::<ir::NodeId>::new();
		//frontier_node_ids.extend_from_slice(& node_ids);
		for & node_id in node_ids.iter().rev()
		{
			frontier_node_ids.push(node_id);
		}

		//for & node_id in node_ids.iter()
		while let Some(node_id) = frontier_node_ids.pop()
		{
			assert!(self.registered_node_set.contains(& node_id));
			let is_locally_resident = self.locally_resident_node_set.contains(& node_id);
			let gpu_residency_state = & self.node_gpu_residency_state.get(& node_id);

			if self.proxy_node_map.contains_key(& node_id)
			{
				frontier_node_ids.push(self.proxy_node_map[& node_id]);
			}

			if ! is_locally_resident
			{
				if gpu_residency_state.is_some()
				{
					gpu_resident_node_dependencies.push(node_id);
				}
			}
		}

		if gpu_resident_node_dependencies.len() > 0
		{
			self.transition_gpu(gpu_resident_node_dependencies.as_slice(), funclet_builder, GpuResidencyState::Useable);
			for & node_id in gpu_resident_node_dependencies.iter()
			{
				self.locally_resident_node_set.insert(node_id);
			}
		}
	}
}

struct Explicator<'program>
{
	program : &'program mut ir::Program
}

fn remap_nodes(funclet_builder : & ir_builders::FuncletBuilder, node_ids : &[ir::NodeId]) -> Box<[ir::NodeId]>
{
	let mut remapped_node_ids = Vec::<ir::NodeId>::new();
	for & node_id in node_ids.iter()
	{
		remapped_node_ids.push(funclet_builder.get_remapped_node_id(node_id).unwrap());
	}
	return remapped_node_ids.into_boxed_slice();
}

impl<'program> Explicator<'program>
{
	pub fn new(program : &'program mut ir::Program) -> Self
	{
		Self {program}
	}

	pub fn run(&mut self)
	{
		for (funclet_id, funclet) in self.program.funclets.iter_mut()
		{

			match funclet.kind
			{
				ir::FuncletKind::MixedExplicit => (),
				ir::FuncletKind::MixedImplicit => *funclet = Self::explicate_funclet(funclet),
			}
		}
	}

	fn explicate_funclet(original_funclet : & ir::Funclet) -> ir::Funclet
	{
		match original_funclet.kind
		{
			ir::FuncletKind::MixedExplicit => panic!("Should not be here"),
			ir::FuncletKind::MixedImplicit => (),
		}

		// funclet_id : ir::FuncletId
		{
			//let original_funclet = & self.program.funclets[& funclet_id];
			//let original_funclet = & program.funclets[& funclet_id];

			let mut funclet_builder = ir_builders::FuncletBuilder::new(ir::FuncletKind::MixedExplicit);

			let mut per_input_input_resource_states = Vec::<BTreeMap<ir::Place, ir::ResourceState>>::new();

			for (input_index, input_type) in original_funclet.input_types.iter().enumerate()
			{
				funclet_builder.add_input(* input_type);

				per_input_input_resource_states.push(BTreeMap::new());

				if let Some(input_resource_states) = original_funclet.input_resource_states.get(input_index)
				{
					for (&place, &resource_state) in input_resource_states.iter()
					{
						funclet_builder.place_input(input_index, place, resource_state);
						per_input_input_resource_states[input_index].insert(place, resource_state);
					}
				}
				else
				{
					let place = ir::Place::Simple{scope : ir::Scope::Local};
					let resource_state = ir::ResourceState{stage : ir::ResourceQueueStage::Ready, is_exclusive : false};
					per_input_input_resource_states[input_index].insert(place, resource_state);
					funclet_builder.place_input(input_index, place, resource_state);
				}
			}

			let mut node_resource_tracker = NodeResourceTracker::new();
			let node_usage_analysis = NodeUsageAnalysis::from_funclet(original_funclet);
			//let node_remapping = HashMap::<ir::NodeId, ir::NodeId>::new();

			//let encoded_node_set = HashSet::<ir::NodeId>::new();
			//let gpu_resident_node_set = HashSet::<ir::NodeId>::new();
			//let local_resident_node_set = HashSet::<ir::NodeId>::new();
	
			for (current_node_id, node) in original_funclet.nodes.iter().enumerate()
			{
				if ! node_usage_analysis.is_node_used(current_node_id)
				{
					//node_result_tracker.store_node_result(current_node_id, NodeResult::Retired, None);
					continue;
				}
	
				match node
				{
					ir::Node::Phi {index} =>
					{
						let new_node_id = funclet_builder.add_node_from_old(current_node_id, & node);
						for (&place, &resource_state) in per_input_input_resource_states[* index].iter()
						{
							match place
							{
								ir::Place::Simple{scope : ir::Scope::Local} => node_resource_tracker.register_local_nodes(&[new_node_id]),
								ir::Place::Simple{scope : ir::Scope::Gpu} =>
								{
									// Doesn't handle exclusivity yet
									match resource_state.stage
									{
										ir::ResourceQueueStage::None => (),
										ir::ResourceQueueStage::Encoded => node_resource_tracker.register_gpu_encoded_nodes(&[new_node_id]),
										ir::ResourceQueueStage::Submitted => node_resource_tracker.register_gpu_submitted_nodes(&[new_node_id]),
										ir::ResourceQueueStage::Ready => node_resource_tracker.register_gpu_ready_nodes(&[new_node_id]),
										ir::ResourceQueueStage::Dead => (),
									}
								}
								_ => panic!("Unimplemented placement for explication of phi nodes")
							}
							
						}
					}
					ir::Node::ExtractResult { node_id, index } =>
					{
						// This isn't right
						//node_resource_tracker.sync_local(& remap_nodes(& funclet_builder, &[* node_id]), &mut funclet_builder);
						let new_node_id = funclet_builder.add_node_from_old(current_node_id, & node);
						//node_resource_tracker.register_local_nodes(&[new_node_id]);
						node_resource_tracker.register_passthrough_node(new_node_id, funclet_builder.get_remapped_node_id(* node_id).unwrap());
					}
					ir::Node::ConstantInteger{value, type_id} =>
					{
						let new_node_id = funclet_builder.add_node_from_old(current_node_id, & node);
						node_resource_tracker.register_local_nodes(&[new_node_id]);
					}
					ir::Node::ConstantUnsignedInteger{value, type_id} =>
					{
						let new_node_id = funclet_builder.add_node_from_old(current_node_id, & node);
						node_resource_tracker.register_local_nodes(&[new_node_id]);
					}
					ir::Node::CallExternalCpu { external_function_id, arguments } =>
					{
						node_resource_tracker.sync_local(& remap_nodes(& funclet_builder, arguments), &mut funclet_builder);
						let new_node_id = funclet_builder.add_node_from_old(current_node_id, & node);
						node_resource_tracker.register_local_nodes(&[new_node_id]);
					}
					ir::Node::CallExternalGpuCompute {external_function_id, arguments, dimensions} =>
					{
						node_resource_tracker.encode_gpu(& remap_nodes(& funclet_builder, arguments), &mut funclet_builder);
						node_resource_tracker.sync_local(& remap_nodes(& funclet_builder, dimensions), &mut funclet_builder);
						let new_node_id = funclet_builder.add_node_from_old(current_node_id, & node);
						node_resource_tracker.register_gpu_encoded_nodes(&[new_node_id]);
					}
					ir::Node::EncodeGpu{values} =>
					{
						let new_node_id = funclet_builder.add_node_from_old(current_node_id, & node);
					}
					ir::Node::SubmitGpu{values} =>
					{
						node_resource_tracker.encode_gpu(& remap_nodes(& funclet_builder, values), &mut funclet_builder);
						let new_node_id = funclet_builder.add_node_from_old(current_node_id, & node);
					}
					ir::Node::SyncLocal{values} =>
					{
						node_resource_tracker.sync_local(& remap_nodes(& funclet_builder, values), &mut funclet_builder);
						let new_node_id = funclet_builder.add_node_from_old(current_node_id, & node);
					}
					_ => panic!("Unknown node")
				};
			}
	
			let mut output_nodes = Vec::<ir::NodeId>::new();

			match & original_funclet.tail_edge
			{
				ir::TailEdge::Return { return_values } =>
				{
					funclet_builder.set_output_types(& original_funclet.output_types);
					output_nodes.extend_from_slice(& return_values);
					node_resource_tracker.sync_local(& remap_nodes(& funclet_builder, return_values), &mut funclet_builder);
					funclet_builder.set_tail_edge_from_old(& original_funclet.tail_edge)
				}
				ir::TailEdge::Yield { funclet_ids, captured_arguments, return_values } =>
				{
					funclet_builder.set_output_types(& original_funclet.output_types);
					output_nodes.extend_from_slice(& captured_arguments);
					output_nodes.extend_from_slice(& return_values);
					node_resource_tracker.sync_local(& remap_nodes(& funclet_builder, captured_arguments), &mut funclet_builder); // Not ideal, but required for now
					node_resource_tracker.sync_local(& remap_nodes(& funclet_builder, return_values), &mut funclet_builder);
					funclet_builder.set_tail_edge_from_old(& original_funclet.tail_edge)
				}
			}

			{
				let mut gpu_encoded_nodes = Vec::<ir::NodeId>::new();
				let mut gpu_submitted_nodes = Vec::<ir::NodeId>::new();
				let mut gpu_ready_nodes = Vec::<ir::NodeId>::new();
				let mut local_nodes = Vec::<ir::NodeId>::new();

				for (output_index, & node_id) in output_nodes.iter().enumerate()
				{
					if let Some(output_resource_states) = original_funclet.output_resource_states.get(output_index)
					{
						// This will be a problem if we can ever have more than one gpu or local placement for a node
						for (&place, &resource_state) in output_resource_states.iter()
						{
							funclet_builder.place_output(output_index, place, resource_state);
							match place
							{
								ir::Place::Simple{scope : ir::Scope::Local} => local_nodes.push(node_id),
								ir::Place::Simple{scope : ir::Scope::Gpu} =>
								{
									match resource_state.stage
									{
										ir::ResourceQueueStage::None => (),
										ir::ResourceQueueStage::Encoded => gpu_encoded_nodes.push(node_id),
										ir::ResourceQueueStage::Submitted => gpu_submitted_nodes.push(node_id),
										ir::ResourceQueueStage::Ready => gpu_ready_nodes.push(node_id),
										ir::ResourceQueueStage::Dead => (),
									}
								}
								_ => panic!("Unimplemented")
							}
						}
					}
					else
					{
						local_nodes.push(node_id);
						funclet_builder.place_output(output_index, ir::Place::Simple{scope : ir::Scope::Local}, ir::ResourceState{stage : ir::ResourceQueueStage::Ready, is_exclusive : false});
					}
				}

				node_resource_tracker.encode_gpu(& remap_nodes(& funclet_builder, gpu_encoded_nodes.as_slice()), &mut funclet_builder);
				node_resource_tracker.submit_gpu(& remap_nodes(& funclet_builder, gpu_submitted_nodes.as_slice()), &mut funclet_builder);
				// Should really be a sync_gpu, but that doesn't exist yet
				node_resource_tracker.sync_local(& remap_nodes(& funclet_builder, gpu_ready_nodes.as_slice()), &mut funclet_builder);
				node_resource_tracker.sync_local(& remap_nodes(& funclet_builder, local_nodes.as_slice()), &mut funclet_builder);
			}

			return funclet_builder.build();
		}
	}
}

// Converts the dataflow to control flow and makes the implicit scheduling explicit using a canonical interpretation when no hints are given
// Transitions from a language where scheduling is optional to one where it is required
pub fn explicate_scheduling(program : &mut ir::Program)
{
	let mut explicator = Explicator::new(program);
	explicator.run();
	//explicator.explicate_funclet();
}
