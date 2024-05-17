//! Lowers a scheduling function into caiman assembly.
//! Invokes the AST -> HIR transformation and all related passes
//! for this, then applies syntax-directed lowering of HIR to Caiman Assembly.

use std::collections::BTreeSet;

use caiman::assembly::ast::{self as asm, MetaMapping};
use caiman::explication::Hole;

use crate::{
    enum_cast,
    error::{type_error, LocalError},
    lower::{data_type_to_ffi_type, IN_STEM},
    parse::ast::{self, DataType, Flow, SchedTerm, SchedulingFunc, SpecType, Tag},
    typing::{Context, LOCAL_TEMP_FLAGS},
};
use caiman::ir;

use super::{
    data_type_to_storage_type,
    sched_hir::{
        DataMovement, FenceOp, Funclet, Funclets, HirBody, HirFuncCall, Specs, Terminator,
        TripleTag,
    },
    tuple_id,
};

/// A vector of commands with holes.
/// A hole in a command means `???`
type CommandVec = Vec<Hole<asm::Command>>;

/// Gets the name of a temporary variable with the given id
#[inline]
fn temp_var_name(temp_id: usize) -> String {
    format!("_t{temp_id}")
}

/// Constructs a copy from `src` to `dest` using either a local copy or a local
/// do builtin, depending on whether `src` is atomic in the value spec.
fn build_copy_cmd(
    dest: &str,
    src: &SchedTerm,
    f: &Funclet,
    backup_tag: Option<&TripleTag>,
) -> asm::Command {
    let val_quot = src
        .get_tags()
        .map(|t| tag_to_remote_id(&TripleTag::from_tags(t).value))
        .or_else(|| backup_tag.map(|t| tag_to_remote_id(&t.value)))
        .or_else(|| {
            if let SchedTerm::Var { name, .. } = src {
                f.get_out_tag(name).map(|t| tag_to_remote_id(&t.value))
            } else {
                None
            }
        });
    if let Some(quot) = val_quot {
        if f.is_literal_value(&quot) {
            return asm::Command::Node(asm::NamedNode {
                name: None,
                node: asm::Node::LocalDoBuiltin {
                    operation: Hole::Filled(quot),
                    inputs: Hole::Filled(vec![]),
                    outputs: Hole::Filled(vec![Hole::Filled(asm::NodeId(dest.to_string()))]),
                },
            });
        }
    }
    let src = enum_cast!(SchedTerm::Var { name, .. }, name, src);
    if f.is_var_or_ref(src) || f.get_flags().contains_key(src) {
        asm::Command::Node(asm::NamedNode {
            name: None,
            node: asm::Node::LocalCopy {
                input: Hole::Filled(asm::NodeId(src.clone())),
                output: Hole::Filled(asm::NodeId(dest.to_string())),
            },
        })
    } else {
        asm::Command::Node(asm::NamedNode {
            name: None,
            node: asm::Node::WriteRef {
                source: Hole::Filled(asm::NodeId(src.clone())),
                destination: Hole::Filled(asm::NodeId(dest.to_string())),
                storage_type: Hole::Filled(data_type_to_storage_type(f.get_dtype(dest).unwrap())),
            },
        })
    }
}

/// Lowers a flattened declaration statement into a caiman assembly command
/// # Returns
/// A tuple containing the commands that implement the statement
/// and the next available temporary id
fn lower_flat_decl(
    dest: &str,
    dest_tag: &TripleTag,
    rhs: &SchedTerm,
    temp_id: usize,
    f: &Funclet,
) -> (CommandVec, usize) {
    let temp_node_name = temp_var_name(temp_id);
    let temp = asm::Command::Node(asm::NamedNode {
        name: Some(asm::NodeId(temp_node_name.clone())),
        node: asm::Node::AllocTemporary {
            place: Hole::Filled(ir::Place::Local),
            buffer_flags: Hole::Filled(LOCAL_TEMP_FLAGS),
            storage_type: Hole::Filled(data_type_to_ffi_type(f.get_dtype(dest).unwrap())),
        },
    });
    let mv = build_copy_cmd(&temp_node_name, rhs, f, Some(dest_tag));
    let rd_ref = asm::Command::Node(asm::NamedNode {
        name: Some(asm::NodeId(dest.to_string())),
        node: asm::Node::ReadRef {
            source: Hole::Filled(asm::NodeId(temp_node_name)),
            storage_type: Hole::Filled(data_type_to_ffi_type(f.get_dtype(dest).unwrap())),
        },
    });
    (
        vec![Hole::Filled(temp), Hole::Filled(mv), Hole::Filled(rd_ref)],
        temp_id + 1,
    )
}

/// Lowers a variable declaration
fn lower_var_decl(
    dest: &str,
    dest_tag: &TripleTag,
    rhs: &Option<SchedTerm>,
    temp_id: usize,
    f: &Funclet,
) -> (CommandVec, usize) {
    let mut result = vec![Hole::Filled(asm::Command::Node(asm::NamedNode {
        name: Some(asm::NodeId(dest.to_string())),
        node: asm::Node::AllocTemporary {
            place: Hole::Filled(ir::Place::Local),
            buffer_flags: Hole::Filled(LOCAL_TEMP_FLAGS),
            storage_type: Hole::Filled(data_type_to_ffi_type(f.get_dtype(dest).unwrap())),
        },
    }))];
    if let Some(rhs) = rhs {
        result.push(Hole::Filled(build_copy_cmd(dest, rhs, f, Some(dest_tag))));
    }
    (result, temp_id)
}

/// Lowers a store lhs <- rhs
fn lower_store(
    lhs: &str,
    lhs_tags: &TripleTag,
    rhs: &SchedTerm,
    temp_id: usize,
    f: &Funclet,
) -> (CommandVec, usize) {
    (
        vec![Hole::Filled(build_copy_cmd(lhs, rhs, f, Some(lhs_tags)))],
        temp_id,
    )
}

/// Lowers an operation into a local-do-external and read-ref
fn lower_op(
    dests: &[(String, TripleTag)],
    op: &str,
    args: &[SchedTerm],
    temp_id: usize,
    f: &Funclet,
) -> (CommandVec, usize) {
    // alloc temps for each destination
    let temps: Vec<_> = dests
        .iter()
        .enumerate()
        .map(|(id, (name, _))| {
            let temp_node_name = temp_var_name(temp_id + id);
            (
                temp_node_name.clone(),
                asm::Command::Node(asm::NamedNode {
                    name: Some(asm::NodeId(temp_node_name)),
                    node: asm::Node::AllocTemporary {
                        place: Hole::Filled(ir::Place::Local),
                        buffer_flags: Hole::Filled(LOCAL_TEMP_FLAGS),
                        storage_type: Hole::Filled(data_type_to_ffi_type(
                            f.get_dtype(name).unwrap(),
                        )),
                    },
                }),
            )
        })
        .collect();
    let called_vars = dests
        .iter()
        .map(|(_, t)| t.value.quot_var.spec_var.as_ref().unwrap().clone())
        .collect::<Vec<_>>();
    let mut inputs = vec![];
    for arg in args {
        let arg = enum_cast!(SchedTerm::Var { name, .. }, name, arg);
        inputs.push(Hole::Filled(asm::NodeId(arg.clone())));
    }
    let local_do = asm::Command::Node(asm::NamedNode {
        name: None,
        node: asm::Node::LocalDoExternal {
            operation: Hole::Filled(asm::RemoteNodeId {
                funclet: SpecType::Value.get_meta_id(),
                node: Some(Hole::Filled(asm::NodeId(tuple_id(&called_vars)))),
            }),
            inputs: Hole::Filled(inputs),
            outputs: Hole::Filled(
                temps
                    .iter()
                    .map(|(n, _)| Hole::Filled(asm::NodeId(n.clone())))
                    .collect(),
            ),
            external_function_id: Hole::Filled(asm::ExternalFunctionId(op.to_string())),
        },
    });
    // read ref for each destination
    let read_refs: Vec<_> = dests
        .iter()
        .zip(temps.iter())
        .map(|((name, _), (temp, _))| {
            Hole::Filled(asm::Command::Node(asm::NamedNode {
                name: Some(asm::NodeId(name.clone())),
                node: asm::Node::ReadRef {
                    source: Hole::Filled(asm::NodeId(temp.clone())),
                    storage_type: Hole::Filled(data_type_to_ffi_type(f.get_dtype(name).unwrap())),
                },
            }))
        })
        .collect();
    (
        temps
            .into_iter()
            .map(|(_, c)| Hole::Filled(c))
            .chain(std::iter::once(Hole::Filled(local_do)))
            .chain(read_refs)
            .collect(),
        temp_id + dests.len(),
    )
}

fn lower_load(dest: &str, typ: &DataType, src: &str, temp_id: usize) -> (CommandVec, usize) {
    (
        vec![Hole::Filled(asm::Command::Node(asm::NamedNode {
            name: Some(asm::NodeId(dest.to_string())),
            node: asm::Node::ReadRef {
                source: Hole::Filled(asm::NodeId(src.to_string())),
                storage_type: Hole::Filled(data_type_to_ffi_type(typ)),
            },
        }))],
        temp_id,
    )
}

/// Lowers a begin-encode operation into a caiman assembly command
/// # Arguments
/// * `device` - the device to encode on
/// * `device_vars` - the names of the variables to encode
/// * `encoder` - the name of the encoder to use
/// * `tags` - the tags for the operation
/// * `temp_id` - the next available temporary id
/// * `f` - the funclet that contains the operation
fn lower_begin_encode(
    device: &str,
    device_vars: &[(String, TripleTag)],
    encoder: &str,
    tags: &TripleTag,
    temp_id: usize,
    f: &Funclet,
) -> (CommandVec, usize) {
    let place = match device {
        "gpu" => ir::Place::Gpu,
        "cpu" => ir::Place::Cpu,
        _ => ir::Place::Local,
    };
    // TODO: proper device vars to support multiple encodings in a single function
    // TODO: check if device variables should have reference semantics (as implemented here)
    let mut cmds = vec![];
    for (var, _) in device_vars {
        cmds.push(Hole::Filled(asm::Command::Node(asm::NamedNode {
            name: Some(asm::NodeId(var.clone())),
            node: asm::Node::AllocTemporary {
                place: Hole::Filled(place),
                buffer_flags: Hole::Filled(f.get_flags()[var]),
                storage_type: Hole::Filled(data_type_to_storage_type(f.get_dtype(var).unwrap())),
            },
        })));
    }
    cmds.push(Hole::Filled(asm::Command::Node(asm::NamedNode {
        name: Some(asm::NodeId(encoder.to_string())),
        node: asm::Node::BeginEncoding {
            place: Hole::Filled(place),
            event: Hole::Filled(tag_to_remote_id(&tags.timeline)),
            encoded: Hole::Filled(
                device_vars
                    .iter()
                    .map(|(k, _)| Hole::Filled(asm::NodeId(k.clone())))
                    .collect(),
            ),
            fences: Hole::Filled(vec![]),
        },
    })));
    (cmds, temp_id)
}

/// Lowers a device copy operation into a caiman assembly command. Allocates a new
/// temporary to hold the source if the source is a value.
/// # Arguments
/// * `dest` - the name of the variable to store the result in. Should be a device variable
/// * `src` - the name of the variable to copy from. Should be a host variable
/// * `dir` - the direction of the copy. Must be `HostToDevice` for now
/// * `encoder` - the name of the encoder to use
/// * `temp_id` - the next available temporary id
fn lower_device_copy(
    dest: &str,
    src: &str,
    dir: DataMovement,
    encoder: &str,
    temp_id: usize,
    f: &Funclet,
) -> (CommandVec, usize) {
    assert_eq!(dir, DataMovement::HostToDevice);
    if f.is_var_or_ref(src) {
        (
            vec![Hole::Filled(asm::Command::Node(asm::NamedNode {
                name: None,
                node: asm::Node::EncodeCopy {
                    encoder: Hole::Filled(asm::NodeId(encoder.to_string())),
                    input: Hole::Filled(asm::NodeId(src.to_string())),
                    output: Hole::Filled(asm::NodeId(dest.to_string())),
                },
            }))],
            temp_id,
        )
    } else {
        (
            vec![
                Hole::Filled(asm::Command::Node(asm::NamedNode {
                    name: Some(asm::NodeId(temp_var_name(temp_id))),
                    node: asm::Node::AllocTemporary {
                        place: Hole::Filled(ir::Place::Local),
                        storage_type: Hole::Filled(data_type_to_ffi_type(
                            f.get_dtype(src).unwrap(),
                        )),
                        buffer_flags: Hole::Filled(LOCAL_TEMP_FLAGS),
                    },
                })),
                Hole::Filled(asm::Command::Node(asm::NamedNode {
                    name: None,
                    node: asm::Node::WriteRef {
                        source: Hole::Filled(asm::NodeId(src.to_string())),
                        destination: Hole::Filled(asm::NodeId(temp_var_name(temp_id))),
                        storage_type: Hole::Filled(data_type_to_ffi_type(
                            f.get_dtype(src).unwrap(),
                        )),
                    },
                })),
                Hole::Filled(asm::Command::Node(asm::NamedNode {
                    name: None,
                    node: asm::Node::EncodeCopy {
                        encoder: Hole::Filled(asm::NodeId(encoder.to_string())),
                        input: Hole::Filled(asm::NodeId(temp_var_name(temp_id))),
                        output: Hole::Filled(asm::NodeId(dest.to_string())),
                    },
                })),
            ],
            temp_id + 1,
        )
    }
}

/// Lowers an encode-do operation into a caiman assembly command
/// # Arguments
/// * `dests` - the names of the variables to store the result in. These should be
///            device variables
/// * `func` - the function to call
/// * `encoder` - the name of the encoder to use
/// * `temp_id` - the next available temporary id
fn lower_encode_do(
    dests: &[String],
    func: &HirFuncCall,
    encoder: &str,
    temp_id: usize,
) -> (CommandVec, usize) {
    let local_do = asm::Command::Node(asm::NamedNode {
        name: None,
        node: asm::Node::EncodeDoExternal {
            operation: Hole::Filled(tag_to_remote_id(&func.tag.value)),
            encoder: Hole::Filled(asm::NodeId(encoder.to_string())),
            inputs: Hole::Filled(
                func.args
                    .iter()
                    .map(|x| Hole::Filled(asm::NodeId(x.clone())))
                    .collect(),
            ),
            outputs: Hole::Filled(
                dests
                    .iter()
                    .map(|n| Hole::Filled(asm::NodeId(n.clone())))
                    .collect(),
            ),
            external_function_id: Hole::Filled(asm::ExternalFunctionId(func.target.to_string())),
        },
    });
    (vec![Hole::Filled(local_do)], temp_id)
}

/// Lowers a fence operation into a caiman assembly command
/// # Arguments
/// * `dest` - the name of the variable to store the result in. May be `None`
/// * `op` - the type of fence operation
/// * `src` - the name of the variable to synchronize on for a sync fence or the
///          encoder to submit for a submit fence
/// * `tags` - the tags for the fence
/// * `temp_id` - the next available temporary id
fn lower_fence_op(
    dest: &Option<String>,
    op: FenceOp,
    src: &SchedTerm,
    tags: &TripleTag,
    temp_id: usize,
) -> (CommandVec, usize) {
    let src = enum_cast!(SchedTerm::Var { name, .. }, name, src).clone();
    let local_do = match op {
        FenceOp::Submit => asm::Command::Node(asm::NamedNode {
            name: dest.as_ref().map(|x| asm::NodeId(x.clone())),
            node: asm::Node::Submit {
                encoder: Hole::Filled(asm::NodeId(src)),
                event: Hole::Filled(tag_to_remote_id(&tags.timeline)),
            },
        }),
        FenceOp::Sync => asm::Command::Node(asm::NamedNode {
            name: None,
            node: asm::Node::SyncFence {
                fence: Hole::Filled(asm::NodeId(src)),
                event: Hole::Filled(tag_to_remote_id(&tags.timeline)),
            },
        }),
    };
    (vec![Hole::Filled(local_do)], temp_id)
}

/// Lowers a scheduling statement into a caiman assembly command
/// # Returns
/// A tuple containing the commands that implement the statement
/// and the next available temporary id
fn lower_instr(s: &HirBody, temp_id: usize, f: &Funclet) -> (CommandVec, usize) {
    match s {
        HirBody::ConstDecl {
            lhs, rhs, lhs_tag, ..
        } => lower_flat_decl(lhs, lhs_tag, rhs, temp_id, f),
        HirBody::VarDecl {
            lhs, lhs_tag, rhs, ..
        } => lower_var_decl(lhs, lhs_tag, rhs, temp_id, f),
        HirBody::RefStore {
            lhs, rhs, lhs_tags, ..
        } => lower_store(lhs, lhs_tags, rhs, temp_id, f),
        HirBody::RefLoad { dest, src, typ, .. } => lower_load(dest, typ, src, temp_id),
        // annotations don't lower to anything
        HirBody::InAnnotation(..) | HirBody::OutAnnotation(..) => (vec![], temp_id),
        HirBody::Op {
            dests, op, args, ..
        } => lower_op(dests, &op.lower(), args, temp_id, f),
        x @ HirBody::Hole(_) => todo!("{x:?}"),
        HirBody::Phi { .. } => panic!("Attempting to lower intermediate form"),
        HirBody::BeginEncoding {
            device,
            device_vars,
            tags,
            encoder,
            ..
        } => lower_begin_encode(device, device_vars, encoder, tags, temp_id, f),
        HirBody::DeviceCopy {
            dest,
            src,
            dir,
            encoder,
            ..
        } => lower_device_copy(dest, src, *dir, encoder, temp_id, f),
        HirBody::EncodeDo {
            dests,
            func,
            encoder,
            ..
        } => lower_encode_do(dests, func, encoder, temp_id),
        HirBody::FenceOp {
            dest,
            op,
            src,
            tags,
            ..
        } => lower_fence_op(dest, *op, src, tags, temp_id),
    }
}

/// Lowers a function call into a caiman assembly command.
/// # Arguments
/// * `call` - the function call to lower
/// * `temp_id` - the next available temporary id
/// * `f` - the funclet that contains the call
/// # Returns
/// A tuple containing the commands that implement the call
fn lower_func_call(
    call: &HirFuncCall,
    captures: &BTreeSet<String>,
    temp_id: usize,
    f: &Funclet,
) -> CommandVec {
    let djoin_id = temp_id;
    let djoin_name = temp_var_name(djoin_id);
    let join = temp_id + 1;
    let join_var = temp_var_name(join);
    vec![
        Hole::Filled(asm::Command::Node(asm::NamedNode {
            name: Some(asm::NodeId(djoin_name.clone())),
            node: asm::Node::DefaultJoin,
        })),
        Hole::Filled(asm::Command::Node(asm::NamedNode {
            name: Some(asm::NodeId(join_var.clone())),
            node: asm::Node::InlineJoin {
                funclet: f.next_blocks().first().unwrap().clone(),
                captures: Hole::Filled(
                    captures
                        .iter()
                        .map(|x| Hole::Filled(asm::NodeId(x.clone())))
                        .collect(),
                ),
                continuation: Hole::Filled(asm::NodeId(djoin_name)),
            },
        })),
        Hole::Filled(asm::Command::TailEdge(asm::TailEdge::ScheduleCall {
            operations: Hole::Filled(
                call.tag
                    .clone()
                    .tags_vec()
                    .into_iter()
                    .map(|x| x.quot)
                    .collect(),
            ),
            callee_funclet_id: Hole::Filled(asm::FuncletId(call.target.clone())),
            callee_arguments: Hole::Filled(
                call.args
                    .iter()
                    .map(|x| Hole::Filled(asm::NodeId(x.clone())))
                    .collect(),
            ),
            continuation_join: Hole::Filled(asm::NodeId(join_var)),
        })),
    ]
}

/// Lowers a yield terminator into a caiman assembly command
/// # Arguments
/// * `captures` - the names of the variables to capture to the continuation
/// * `temp_id` - the next available temporary id
/// * `f` - the funclet that contains the yield
/// # Returns
/// A vec containing the commands that implement the yield
fn lower_yield(captures: &[String], temp_id: usize, f: &Funclet) -> CommandVec {
    let djoin_id = temp_id;
    let djoin_name = temp_var_name(djoin_id);
    let join = temp_id + 1;
    let join_var = temp_var_name(join);
    vec![
        Hole::Filled(asm::Command::Node(asm::NamedNode {
            name: Some(asm::NodeId(djoin_name.clone())),
            node: asm::Node::DefaultJoin,
        })),
        Hole::Filled(asm::Command::Node(asm::NamedNode {
            name: Some(asm::NodeId(join_var.clone())),
            node: asm::Node::SerializedJoin {
                funclet: f.next_blocks().first().unwrap().clone(),
                captures: Hole::Filled(
                    captures
                        .iter()
                        .map(|x| Hole::Filled(asm::NodeId(x.clone())))
                        .collect(),
                ),
                continuation: Hole::Filled(asm::NodeId(djoin_name)),
            },
        })),
        Hole::Filled(asm::Command::TailEdge(asm::TailEdge::ScheduleCallYield {
            operations: Hole::Filled(vec![
                Hole::Filled(asm::RemoteNodeId {
                    node: None,
                    funclet: SpecType::Value.get_meta_id(),
                }),
                Hole::Filled(asm::RemoteNodeId {
                    node: None,
                    funclet: SpecType::Spatial.get_meta_id(),
                }),
                Hole::Filled(asm::RemoteNodeId {
                    node: None,
                    funclet: SpecType::Timeline.get_meta_id(),
                }),
            ]),
            external_function_id: Hole::Filled(asm::ExternalFunctionId(String::from("_loop_impl"))),
            yielded_nodes: Hole::Filled(vec![]),
            continuation_join: Hole::Filled(asm::NodeId(join_var)),
        })),
    ]
}

/// Lowers a return terminator into a caiman assembly command.
/// If the return is a final return, it is lowered into a default join and a
/// jump to the final block. Otherwise it is lowered into a return command.
///
/// # Arguments
/// * `ret` - the name of the variable to return
/// * `temp_id` - the next available temporary id
/// * `f` - the funclet that contains the return
/// # Returns
/// A tuple containing the commands that implement the return
fn lower_ret(rets: &[String], passthrough: &[String], temp_id: usize, f: &Funclet) -> CommandVec {
    assert!(passthrough.len() <= 1 || passthrough.iter().le(passthrough.iter().skip(1)));
    if f.is_final_return() {
        let djoin_id = temp_id;
        let djoin_name = temp_var_name(djoin_id);
        let join = temp_id + 1;
        let join_var = temp_var_name(join);
        assert_eq!(f.next_blocks().len(), 1);
        vec![
            Hole::Filled(asm::Command::Node(asm::NamedNode {
                name: Some(asm::NodeId(djoin_name.clone())),
                node: asm::Node::DefaultJoin,
            })),
            Hole::Filled(asm::Command::Node(asm::NamedNode {
                name: Some(asm::NodeId(join_var.clone())),
                node: asm::Node::InlineJoin {
                    funclet: f.next_blocks().first().unwrap().clone(),
                    captures: Hole::Filled(vec![]),
                    continuation: Hole::Filled(asm::NodeId(djoin_name)),
                },
            })),
            Hole::Filled(asm::Command::TailEdge(asm::TailEdge::Jump {
                arguments: Hole::Filled(
                    rets.iter()
                        .chain(passthrough.iter())
                        .map(|x| Hole::Filled(asm::NodeId(x.clone())))
                        .collect(),
                ),
                join: Hole::Filled(asm::NodeId(join_var)),
            })),
        ]
    } else {
        vec![Hole::Filled(asm::Command::TailEdge(
            asm::TailEdge::Return {
                return_values: Hole::Filled(
                    rets.iter()
                        .chain(passthrough.iter())
                        .map(|x| Hole::Filled(asm::NodeId(x.clone())))
                        .collect(),
                ),
            },
        ))]
    }
}

/// Lowers a basic block terminator into a caiman assembly command
/// # Returns
/// A tuple containing the commands that implement the terminator
/// and the next available temporary id
fn lower_terminator(t: &Terminator, temp_id: usize, f: &Funclet<'_>) -> CommandVec {
    // we do not return the new `temp_id` because this is the last instruction
    // in the block
    match t {
        Terminator::Return {
            rets, passthrough, ..
        } => lower_ret(rets, passthrough, temp_id, f),
        Terminator::Next(vars) => {
            vec![Hole::Filled(asm::Command::TailEdge(
                asm::TailEdge::Return {
                    return_values: Hole::Filled(
                        vars.iter()
                            .map(|v| Hole::Filled(asm::NodeId(v.clone())))
                            .collect(),
                    ),
                },
            ))]
        }
        Terminator::FinalReturn(n) => vec![Hole::Filled(asm::Command::TailEdge(
            asm::TailEdge::Return {
                return_values: Hole::Filled(
                    n.iter()
                        .map(|v| Hole::Filled(asm::NodeId(v.clone())))
                        .collect(),
                ),
            },
        ))],
        Terminator::Select { guard, tag, .. } => lower_select(guard, tag, temp_id, f),
        // TODO: review this
        Terminator::None => panic!("None terminator not replaced by Next"),
        Terminator::Call(..) => panic!("Call not replaced by CaptureCall"),
        Terminator::CaptureCall { call, captures, .. } => {
            lower_func_call(call, captures, temp_id, f)
        }
        Terminator::Yield(captures) => lower_yield(captures, temp_id, f),
    }
}

/// Lowers a select terminator into a series of caiman assembly commands
/// # Returns
/// The commands that implement the terminator
fn lower_select(guard_name: &str, tags: &TripleTag, temp_id: usize, f: &Funclet<'_>) -> CommandVec {
    let djoin_id = temp_id;
    let djoin_name = temp_var_name(djoin_id);
    let join = temp_id + 1;
    let join_var = temp_var_name(join);
    vec![
        Hole::Filled(asm::Command::Node(asm::NamedNode {
            name: Some(asm::NodeId(djoin_name.clone())),
            node: asm::Node::DefaultJoin,
        })),
        Hole::Filled(asm::Command::Node(asm::NamedNode {
            name: Some(asm::NodeId(join_var.clone())),
            node: asm::Node::InlineJoin {
                funclet: Hole::Filled(f.join_funclet()),
                captures: Hole::Filled(vec![]),
                continuation: Hole::Filled(asm::NodeId(djoin_name)),
            },
        })),
        Hole::Filled(asm::Command::TailEdge(asm::TailEdge::ScheduleSelect {
            operations: Hole::Filled(
                tags.clone()
                    .tags_vec()
                    .into_iter()
                    .map(|x| x.quot)
                    .collect(),
            ),
            condition: Hole::Filled(asm::NodeId(guard_name.to_string())),
            callee_funclet_ids: Hole::Filled(f.next_blocks()),
            callee_arguments: Hole::Filled(f.output_args()),
            continuation_join: Hole::Filled(asm::NodeId(join_var)),
        })),
    ]
}

/// Gets the assembly quotient from a high level caiman tag
pub fn tag_to_remote_id(t: &Tag) -> asm::RemoteNodeId {
    asm::RemoteNodeId {
        node: if matches!(t.quot, Some(ast::Quotient::None)) {
            None
        } else {
            Some(
                t.quot_var
                    .spec_var
                    .clone()
                    .map(|x| {
                        if matches!(t.quot, Some(ast::Quotient::Input)) {
                            asm::NodeId(format!("{IN_STEM}{x}"))
                        } else {
                            asm::NodeId(x)
                        }
                    })
                    .into(),
            )
        },
        funclet: t.quot_var.spec_type.get_meta_id(),
    }
}

/// Converts a hlc tag to a tag in the assembly
pub fn tag_to_tag(t: &Tag) -> asm::Tag {
    asm::Tag {
        quot: Hole::Filled(tag_to_remote_id(t)),
        flow: match t.flow.expect("TODO: Holes in flow") {
            Flow::Dead => Hole::Filled(ir::Flow::Dead),
            Flow::Need => Hole::Filled(ir::Flow::Need),
            Flow::Usable => Hole::Filled(ir::Flow::Usable),
            Flow::Save => Hole::Filled(ir::Flow::Saved),
        },
    }
}

/// Lowers a basic block into a caiman assembly funclet
///
fn lower_block(funclet: &Funclet<'_>) -> asm::Funclet {
    let mut commands = vec![];
    let inputs = funclet.inputs();
    for idx in 0..inputs.len() {
        commands.push(Hole::Filled(asm::Command::Node(asm::NamedNode {
            name: None,
            node: asm::Node::Phi {
                index: Hole::Filled(idx),
            },
        })));
    }
    let mut temp_id = 0;
    for cmd in funclet.stmts() {
        let (mut new_cmds, new_id) = lower_instr(cmd, temp_id, funclet);
        temp_id = new_id;
        commands.append(&mut new_cmds);
    }
    commands.extend(lower_terminator(funclet.terminator(), temp_id, funclet));
    // TODO: implicit timeline tag deduction
    let get_tag = |name: &str| {
        let t = if name == "input" {
            funclet.get_input_tag(name)
        } else {
            funclet.get_out_tag(name).cloned()
        };
        t.map_or(
            asm::Tag {
                quot: Hole::Filled(asm::RemoteNodeId {
                    node: None,
                    funclet: SpecType::Timeline.get_meta_id(),
                }),
                flow: Hole::Filled(ir::Flow::Usable),
            },
            |mut t| {
                if t.timeline.flow.is_none() {
                    t.timeline.flow = Some(Flow::Usable);
                }
                assert!(t.timeline.quot_var.spec_var.is_some());
                tag_to_tag(&t.timeline)
            },
        )
    };
    asm::Funclet {
        kind: ir::FuncletKind::ScheduleExplicit,
        header: asm::FuncletHeader {
            name: asm::FuncletId(funclet.name()),
            args: inputs,
            ret: funclet.outputs(),
            binding: asm::FuncletBinding::ScheduleBinding(asm::ScheduleBinding {
                implicit_tags: (get_tag("input"), get_tag("output")),
                meta_map: MetaMapping {
                    value: (SpecType::Value.get_meta_id(), funclet.specs().value.clone()),
                    timeline: (
                        SpecType::Timeline.get_meta_id(),
                        funclet.specs().timeline.clone(),
                    ),
                    spatial: (
                        SpecType::Spatial.get_meta_id(),
                        funclet.specs().spatial.clone(),
                    ),
                },
            }),
        },
        commands,
    }
}

/// Lower a scheduling function into one or more caiman assembly funclet.
/// # Errors
/// Returns an error if the function is missing a spec.
pub fn lower_schedule(
    ctx: &Context,
    func: SchedulingFunc,
) -> Result<Vec<asm::Funclet>, LocalError> {
    let mut val = None;
    let mut timeline = None;
    let mut spatial = None;
    if func.specs.len() > 3 {
        return Err(type_error(func.info, "Too many specs"));
    }
    for spec in &func.specs {
        match ctx.specs.get(spec).map(|s| s.typ) {
            Some(SpecType::Value) => val = Some(spec.to_string()),
            Some(SpecType::Timeline) => timeline = Some(spec.to_string()),
            Some(SpecType::Spatial) => spatial = Some(spec.to_string()),
            None => return Err(type_error(func.info, &format!("Spec '{spec}' not found"))),
        }
    }
    let specs = Specs {
        value: val
            .map(asm::FuncletId)
            .ok_or_else(|| type_error(func.info, "Missing value spec"))?,
        timeline: timeline
            .map(asm::FuncletId)
            .ok_or_else(|| type_error(func.info, "Missing timeline spec"))?,
        spatial: spatial
            .map(asm::FuncletId)
            .ok_or_else(|| type_error(func.info, "Missing spatial spec"))?,
    };
    let blocks = Funclets::new(func, &specs, ctx);
    Ok(blocks.funclets().iter().map(lower_block).collect())
}
