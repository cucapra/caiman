use crate::ir;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::default::Default;

pub fn validate_program(program: &ir::Program) {
    for (funclet_id, funclet) in program.funclets.iter() {
        validate_funclet_common(program, funclet);

        match funclet.kind {
            ir::FuncletKind::Timeline => validate_timeline_funclet(program, funclet),
            _ => (),
        }
    }
}

pub fn validate_gpu_kernel_bindings(
    kernel: &ir::ffi::GpuKernel,
    input_slot_node_ids: &[ir::NodeId],
    output_slot_node_ids: &[ir::NodeId],
) {
    use std::iter::FromIterator;
    let mut input_slot_counts = HashMap::<ir::NodeId, usize>::from_iter(
        input_slot_node_ids
            .iter()
            .chain(output_slot_node_ids.iter())
            .map(|slot_id| (*slot_id, 0usize)),
    );
    let mut output_slot_bindings = HashMap::<ir::NodeId, Option<usize>>::from_iter(
        output_slot_node_ids.iter().map(|slot_id| (*slot_id, None)),
    );
    for (binding_index, resource_binding) in kernel.resource_bindings.iter().enumerate() {
        if let Some(index) = resource_binding.input {
            *input_slot_counts
                .get_mut(&input_slot_node_ids[index])
                .unwrap() += 1;
        }

        if let Some(index) = resource_binding.output {
            *output_slot_bindings
                .get_mut(&output_slot_node_ids[index])
                .unwrap() = Some(binding_index);
        }
    }

    for (binding_index, resource_binding) in kernel.resource_bindings.iter().enumerate() {
        if let Some(output_index) = resource_binding.output {
            let output_slot_id = output_slot_node_ids[output_index];
            assert_eq!(input_slot_counts[&output_slot_id], 0);
            assert_eq!(output_slot_bindings[&output_slot_id], Some(binding_index));

            if let Some(input_index) = resource_binding.input {
                let input_slot_id = input_slot_node_ids[input_index];
                assert_eq!(input_slot_counts[&input_slot_id], 1);
            }
        }
    }
}

fn validate_funclet_common(program: &ir::Program, funclet: &ir::Funclet) {
    // All phis must be at the start so that index = node id
    // This is relied on heavily by type checking and codegen (for boundary to interior tag conversion)
    for (index, input_type_id) in funclet.input_types.iter().enumerate() {
        let is_valid = match &funclet.nodes[index] {
            ir::Node::Phi { .. } => true,
            _ => false,
        };
        assert!(is_valid);
    }
}

pub fn validate_timeline_funclet(program: &ir::Program, funclet: &ir::Funclet) {}

/*pub fn validate_timeline_funclet(program: &ir::Program, funclet: &ir::Funclet) {
    // A timeline funclet records the synchronization events as known by the coordinator (local)

    for (input_index, input_type) in funclet.input_types.iter().enumerate() {
        match &program.types[*input_type] {
            ir::Type::Event { place } => assert_eq!(*place, ir::Place::Local),
            _ => panic!(
                "Timeline funclet's input #{} has an unsupported type: #{}",
                input_index, input_type
            ),
        }
    }

    for (output_index, output_type) in funclet.output_types.iter().enumerate() {
        match &program.types[*output_type] {
            ir::Type::Event { place } => assert_eq!(*place, ir::Place::Local),
            _ => panic!(
                "Timeline funclet's output #{} has an unsupported type: #{}",
                output_index, output_type
            ),
        }
    }

    // Need to enforce that synchronization is in order of submission
    // For the sake of efficiently checking this, inputs and outputs must be (pairwise) ordered with respect to each other in time

    let mut most_recently_synchronized_submissions = Vec::<Option<ir::NodeId>>::new();

    for (current_node_id, current_node) in funclet.nodes.iter().enumerate() {
        let last_synchronized_submission = match current_node {
            ir::Node::None => None,
            ir::Node::Phi { index } => {
                assert_eq!(*index, current_node_id);
                assert!(*index < funclet.input_types.len());
                None
            }
            ir::Node::SubmissionEvent {
                here_place,
                there_place,
                local_past,
            } => {
                assert!(*local_past < current_node_id);
                assert_eq!(*here_place, ir::Place::Local);
                assert_ne!(*there_place, ir::Place::Local);
                most_recently_synchronized_submissions[*local_past]
            }
            ir::Node::SynchronizationEvent {
                here_place,
                there_place,
                local_past,
                remote_local_past,
            } => {
                assert!(*local_past < current_node_id);
                assert!(*remote_local_past < current_node_id);
                // Local is always up to date with respect to itself (everything else is in the past)
                assert!(*remote_local_past <= *local_past);
                assert_eq!(*here_place, ir::Place::Local);
                assert_ne!(*there_place, ir::Place::Local);

                let most_recently_sychronized_submission_opt =
                    most_recently_synchronized_submissions[*local_past];
                if let Some(most_recently_sychronized_submission) =
                    most_recently_sychronized_submission_opt
                {
                    // Must synchronize new(er) event
                    assert!(most_recently_sychronized_submission < *remote_local_past);
                }

                if let ir::Node::SubmissionEvent {
                    here_place: submission_here_place,
                    there_place: submission_there_place,
                    local_past: submission_local_past,
                } = &funclet.nodes[*remote_local_past]
                {
                    assert_eq!(*submission_here_place, *here_place);
                    assert_eq!(*submission_there_place, *there_place);
                    assert!(*submission_local_past <= *local_past);
                } else {
                    panic!("Remote can only know of submissions from local")
                }

                Some(*remote_local_past)
            }
            _ => panic!(
                "Node #{}: {:?} is not valid for a timeline funclet",
                current_node_id, current_node
            ),
        };
        most_recently_synchronized_submissions.push(last_synchronized_submission);
    }

    match &funclet.tail_edge {
        ir::TailEdge::Return { return_values } => {
            assert_eq!(return_values.len(), funclet.output_types.len());
            for node_id in return_values.iter() {
                assert!(*node_id < funclet.nodes.len());
            }
        }
        _ => panic!("Tail edge of a timeline funclet must be a return!"),
    }
}*/

pub fn validate_spatial_funclet(program: &ir::Program, funclet: &ir::Funclet) {
    // To do once splitting is a feature we support
    // GPU spaces can't be split in wgpu
    // CPU spaces can
    // Merging is associative
    // Splitting is coassociative
}
