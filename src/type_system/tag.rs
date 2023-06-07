use crate::ir;

pub fn concretize_input_to_internal_tag(
    value_tag: ir::Tag,
) -> ir::Tag {
    match value_tag {
        ir::Tag::None => ir::Tag::None,
        ir::Tag::Node { node_id } => ir::Tag::Node { node_id },
        ir::Tag::Input {
            /*funclet_id,*/ index,
        } => ir::Tag::Node { node_id: index },
        ir::Tag::Output {
            /*funclet_id,*/ index,
        } => ir::Tag::Output {
            /*funclet_id,*/ index,
        },
        _ => panic!("Unimplemented"),
    }
}

pub fn check_tag_compatibility_enter(
    input_spec_node_ids : &[ir::NodeId],
    caller_value_tag: ir::Tag,
    callee_value_tag: ir::Tag,
) {
    match (caller_value_tag, callee_value_tag) {
        (_, ir::Tag::None) => (),
        (ir::Tag::Node { node_id }, ir::Tag::Input { index }) => {
            assert_eq!(input_spec_node_ids[index], node_id);
        }
        _ => panic!(
            "Ill-formed: {:?} to {:?}",
            caller_value_tag, callee_value_tag
        ),
    }
}

// Check value tag in callee (source) scope transfering to caller (destination) scope
pub fn check_tag_compatibility_exit(
    caller_spec_funclet: &ir::Funclet,
    caller_spec_node_id: ir::NodeId,
    source_value_tag: ir::Tag,
    destination_value_tag: ir::Tag,
) {
    match (source_value_tag, destination_value_tag) {
        (_, ir::Tag::None) => (),
        (ir::Tag::Output {index: output_index}, ir::Tag::Node { node_id }) => {
            let node = &caller_spec_funclet.nodes[node_id];
            if let ir::Node::ExtractResult {node_id: call_node_id, index} = node {
                assert_eq!(*index, output_index);
                assert_eq!(*call_node_id, caller_spec_node_id);
            }
            else {
                panic!(
                    "Target operation is not a result extraction: #{:?} {:?}",
                    node_id, node
                );
            }
        }
        _ => panic!(
            "Ill-formed: {:?} to {:?}",
            source_value_tag, destination_value_tag
        ),
    };
}

pub fn check_tag_compatibility_interior_branch(
    current_value_funclet: &ir::Funclet,
    value_operation: ir::RemoteNodeId,
    condition_value_tag: ir::Tag,
    source_value_tags: &[ir::Tag],
    destination_value_tag: ir::Tag,
) {
    assert_eq!(source_value_tags.len(), 2);
   // let current_value_funclet = &program.funclets[value_operation.funclet_id];
    assert_eq!(current_value_funclet.kind, ir::FuncletKind::Value);

    let branch_node_ids = if let ir::Node::Select {
        condition,
        true_case,
        false_case,
    } = &current_value_funclet.nodes[value_operation.node_id]
    {
        check_tag_compatibility_interior(
            current_value_funclet,
            condition_value_tag,
            ir::Tag::Node {
                node_id: *condition,
            },
        );
        [*true_case, *false_case]
    } else {
        panic!("Scheduling select on a node that is not a select");
    };

    for (branch_index, branch_node_id) in branch_node_ids.iter().enumerate() {
        let source_value_tag = source_value_tags[branch_index];
        match source_value_tag {
            ir::Tag::Node { node_id } if node_id == value_operation.node_id => {
                check_tag_compatibility_interior(
                    current_value_funclet,
                    source_value_tag,
                    ir::Tag::Node {
                        node_id: *branch_node_id,
                    },
                );
            }
            _ => {
                check_tag_compatibility_interior(
                    current_value_funclet,
                    source_value_tag,
                    destination_value_tag,
                );
            }
        }
    }
}

pub fn check_tag_compatibility_interior_cast(
    current_value_funclet: &ir::Funclet,
    source_value_tag: ir::Tag,
    destination_value_tag: ir::Tag,
    casts : &[(ir::Tag, ir::Tag)]
) {
    if casts.contains(&(source_value_tag, destination_value_tag)) {
        return;
    }

    check_tag_compatibility_interior(
        current_value_funclet,
        source_value_tag,
        destination_value_tag,
    );
}

// Check value tag transition in same scope
pub fn check_tag_compatibility_interior(
    current_value_funclet: &ir::Funclet,
    source_value_tag: ir::Tag,
    destination_value_tag: ir::Tag
) {
    match (source_value_tag, destination_value_tag) {
        /*(ir::Tag::Halt { index }, ir::Tag::Halt { index: index_2 }) => {
            assert_eq!(index, index_2)
        }
        (ir::Tag::Halt { .. }, _) => panic!("Halt can only match halt"),*/
        (_, ir::Tag::None) => (),
        (
            ir::Tag::Input {
                /*funclet_id,*/ index,
            },
            ir::Tag::Node {
                node_id: remote_node_id,
            },
        ) => {
            if let ir::Node::Phi { index: phi_index } =
                &current_value_funclet.nodes[remote_node_id]
            {
                assert_eq!(*phi_index, index);
            } else {
                panic!("Not a phi");
            }
        }
        (ir::Tag::Node { node_id }, ir::Tag::Node { node_id: node_id_2 }) => {
            assert_eq!(node_id, node_id_2);
        }
        (
            ir::Tag::Node { node_id },
            ir::Tag::Output {
                /*funclet_id,*/ index,
            },
        ) => {
            //assert_eq!(remote_node_id.funclet_id, value_funclet_id_opt.unwrap());

            match &current_value_funclet.tail_edge {
                ir::TailEdge::Return { return_values } => assert_eq!(return_values[index], node_id),
                _ => panic!("Not a unit"),
            }
        }
        (
            ir::Tag::Output {
                /*funclet_id,*/ index,
            },
            ir::Tag::Output {
                /*funclet_id : funclet_id_2,*/ index: index_2,
            },
        ) => {
            //assert_eq!(funclet_id, funclet_id_2);
            assert_eq!(index, index_2);
        }
        _ => panic!(
            "Ill-formed: {:?} to {:?}",
            source_value_tag, destination_value_tag
        ),
    }
}
