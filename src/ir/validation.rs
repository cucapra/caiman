use crate::ir;
use std::collections::{HashMap, BTreeMap, HashSet, BTreeSet};
use std::default::Default;

pub fn validate_external_gpu_function_bindings(function : & ir::ffi::ExternalGpuFunction, input_slot_node_ids : &[ir::NodeId], output_slot_node_ids : &[ir::NodeId])
{
	use std::iter::FromIterator;
	let mut input_slot_counts = HashMap::<ir::NodeId, usize>::from_iter(input_slot_node_ids.iter().chain(output_slot_node_ids.iter()).map(|slot_id| (* slot_id, 0usize)));
	let mut output_slot_bindings = HashMap::<ir::NodeId, Option<usize>>::from_iter(output_slot_node_ids.iter().map(|slot_id| (* slot_id, None)));
	for (binding_index, resource_binding) in function.resource_bindings.iter().enumerate()
	{
		if let Some(index) = resource_binding.input
		{
			* input_slot_counts.get_mut(& input_slot_node_ids[index]).unwrap() += 1;
		}

		if let Some(index) = resource_binding.output
		{
			* output_slot_bindings.get_mut(& output_slot_node_ids[index]).unwrap() = Some(binding_index);
		}
	}

	for (binding_index, resource_binding) in function.resource_bindings.iter().enumerate()
	{
		if let Some(output_index) = resource_binding.output
		{
			let output_slot_id = output_slot_node_ids[output_index];
			assert_eq!(input_slot_counts[& output_slot_id], 0);
			assert_eq!(output_slot_bindings[& output_slot_id], Some(binding_index));

			if let Some(input_index) = resource_binding.input
			{
				let input_slot_id = input_slot_node_ids[input_index];
				assert_eq!(input_slot_counts[& input_slot_id], 1);
			}
		}
	}
}

