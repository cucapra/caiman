use crate::ir;

pub fn concretize_input_to_internal_value_tag(
    program: &ir::Program,
    value_tag: ir::ValueTag,
) -> ir::ValueTag {
    match value_tag {
        ir::ValueTag::None => ir::ValueTag::None,
        ir::ValueTag::Operation { remote_node_id } => ir::ValueTag::Operation { remote_node_id },
        ir::ValueTag::Input { funclet_id, index } => ir::ValueTag::Operation {
            remote_node_id: ir::RemoteNodeId {
                funclet_id,
                node_id: index,
            },
        },
        ir::ValueTag::Output { funclet_id, index } => ir::ValueTag::Output { funclet_id, index },
        _ => panic!("Unimplemented"),
    }
}

pub fn check_value_tag_compatibility_enter(
    program: &ir::Program,
    call_operation: ir::RemoteNodeId,
    caller_value_tag: ir::ValueTag,
    callee_value_tag: ir::ValueTag,
) {
    match (caller_value_tag, callee_value_tag) {
        (_, ir::ValueTag::None) => (),
        (ir::ValueTag::Operation { remote_node_id }, ir::ValueTag::Input { funclet_id, index }) => {
            assert_eq!(call_operation.funclet_id, remote_node_id.funclet_id);
            let caller_value_funclet = &program.funclets[call_operation.funclet_id];
            if let ir::Node::CallValueFunction {
                function_id,
                arguments,
            } = &caller_value_funclet.nodes[call_operation.node_id]
            {
                assert_eq!(arguments[index], remote_node_id.node_id);
            } else {
                panic!("Operation is not a call {:?}", call_operation);
            }
        }
        _ => panic!(
            "Ill-formed: {:?} to {:?}",
            caller_value_tag, callee_value_tag
        ),
    }
}

// Check value tag in callee (source) scope transfering to caller (destination) scope
pub fn check_value_tag_compatibility_exit(
    program: &ir::Program,
    callee_funclet_id: ir::FuncletId,
    source_value_tag: ir::ValueTag,
    continuation_value_operation: ir::RemoteNodeId,
    destination_value_tag: ir::ValueTag,
) {
    match (source_value_tag, destination_value_tag) {
        (_, ir::ValueTag::None) => (),
        (
            ir::ValueTag::Output {
                funclet_id,
                index: output_index,
            },
            ir::ValueTag::Operation { remote_node_id },
        ) => {
            assert_eq!(
                remote_node_id.funclet_id,
                continuation_value_operation.funclet_id
            );
            assert_eq!(funclet_id, callee_funclet_id);

            let node = &program.funclets[remote_node_id.funclet_id].nodes[remote_node_id.node_id];
            if let ir::Node::ExtractResult {
                node_id: call_node_id,
                index,
            } = node
            {
                assert_eq!(*index, output_index);
                assert_eq!(*call_node_id, continuation_value_operation.node_id);
            } else {
                panic!(
                    "Target operation is not a result extraction: #{:?} {:?}",
                    remote_node_id, node
                );
            }
        }
        _ => panic!(
            "Ill-formed: {:?} to {:?}",
            source_value_tag, destination_value_tag
        ),
    };
}

pub fn check_value_tag_compatibility_interior_branch(
    program: &ir::Program,
    value_operation: ir::RemoteNodeId,
    condition_value_tag: ir::ValueTag,
    source_value_tags: &[ir::ValueTag],
    destination_value_tag: ir::ValueTag,
) {
    assert_eq!(source_value_tags.len(), 2);
    let current_value_funclet = &program.funclets[value_operation.funclet_id];
    assert_eq!(current_value_funclet.kind, ir::FuncletKind::Value);

    let branch_node_ids = if let ir::Node::Select {
        condition,
        true_case,
        false_case,
    } = &current_value_funclet.nodes[value_operation.node_id]
    {
        check_value_tag_compatibility_interior(
            &program,
            condition_value_tag,
            ir::ValueTag::Operation {
                remote_node_id: ir::RemoteNodeId {
                    funclet_id: value_operation.funclet_id,
                    node_id: *condition,
                },
            },
        );
        [*true_case, *false_case]
    } else {
        panic!("Scheduling select on a node that is not a select");
    };

    for (branch_index, branch_node_id) in branch_node_ids.iter().enumerate() {
        let source_value_tag = source_value_tags[branch_index];
        match source_value_tag {
            ir::ValueTag::Operation { remote_node_id } if remote_node_id == value_operation => {
                check_value_tag_compatibility_interior(
                    &program,
                    source_value_tag,
                    ir::ValueTag::Operation {
                        remote_node_id: ir::RemoteNodeId {
                            funclet_id: value_operation.funclet_id,
                            node_id: *branch_node_id,
                        },
                    },
                );
            }
            _ => {
                check_value_tag_compatibility_interior(
                    &program,
                    source_value_tag,
                    destination_value_tag,
                );
            }
        }
    }
}

// Check value tag transition in same scope
pub fn check_value_tag_compatibility_interior(
    program: &ir::Program,
    source_value_tag: ir::ValueTag,
    destination_value_tag: ir::ValueTag,
) {
    match (source_value_tag, destination_value_tag) {
        (ir::ValueTag::Halt { index }, ir::ValueTag::Halt { index: index_2 }) => {
            assert_eq!(index, index_2)
        }
        (ir::ValueTag::Halt { .. }, _) => panic!("Halt can only match halt"),
        (_, ir::ValueTag::None) => (),
        (ir::ValueTag::Input { funclet_id, index }, ir::ValueTag::Operation { remote_node_id }) => {
            assert_eq!(remote_node_id.funclet_id, funclet_id);

            let destination_value_funclet = &program.funclets[funclet_id];
            assert_eq!(destination_value_funclet.kind, ir::FuncletKind::Value);

            if let ir::Node::Phi { index: phi_index } =
                &destination_value_funclet.nodes[remote_node_id.node_id]
            {
                assert_eq!(*phi_index, index);
            } else {
                panic!("Not a phi");
            }
        }
        (
            ir::ValueTag::Operation { remote_node_id },
            ir::ValueTag::Operation {
                remote_node_id: remote_node_id_2,
            },
        ) => {
            assert_eq!(remote_node_id, remote_node_id_2);
        }
        (
            ir::ValueTag::Operation { remote_node_id },
            ir::ValueTag::Output { funclet_id, index },
        ) => {
            assert_eq!(remote_node_id.funclet_id, funclet_id);

            let source_value_funclet = &program.funclets[funclet_id];
            assert_eq!(source_value_funclet.kind, ir::FuncletKind::Value);

            match &source_value_funclet.tail_edge {
                ir::TailEdge::Return { return_values } => {
                    assert_eq!(return_values[index], remote_node_id.node_id)
                }
                _ => panic!("Not a unit"),
            }
        }
        (
            ir::ValueTag::Output { funclet_id, index },
            ir::ValueTag::Output {
                funclet_id: funclet_id_2,
                index: index_2,
            },
        ) => {
            assert_eq!(funclet_id, funclet_id_2);
            assert_eq!(index, index_2);
        }
        _ => panic!(
            "Ill-formed: {:?} to {:?}",
            source_value_tag, destination_value_tag
        ),
    }
}
