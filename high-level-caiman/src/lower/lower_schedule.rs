//! Lowers a scheduling function into caiman assembly.
//! Invokes the AST -> HIR transformation and all related passes
//! for this, then applies syntax-directed lowering of HIR to Caiman Assembly.

use std::collections::BTreeSet;

use caiman::assembly::ast::{self as asm, Hole};

use crate::{
    enum_cast,
    error::{type_error, LocalError},
    lower::{data_type_to_ffi_type, sched_hir::TagInfo},
    parse::ast::{DataType, Flow, Quotient, QuotientReference, SchedTerm, SchedulingFunc, Tag},
    typing::{Context, SpecType, LOCAL_TEMP_FLAGS},
};
use caiman::ir;

use super::{
    data_type_to_storage_type,
    sched_hir::{Funclet, Funclets, HirBody, HirFuncCall, Specs, Terminator, TripleTag},
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
        .and_then(|t| {
            TripleTag::from_tags(t, f.specs())
                .value
                .as_ref()
                .map(tag_to_quot)
        })
        .or_else(|| {
            backup_tag
                .map(|t| &t.value)
                .and_then(|t| t.as_ref().map(tag_to_quot))
        })
        .or_else(|| {
            if let SchedTerm::Var { name, .. } = src {
                f.get_out_tag(name)
                    .and_then(|t| t.value.as_ref().map(|t| t.quot.clone()))
            } else {
                None
            }
        });
    if let Some(quot) = val_quot {
        if f.is_literal_value(&quot) {
            return asm::Command::Node(asm::NamedNode {
                name: None,
                node: asm::Node::LocalDoBuiltin {
                    operation: Some(quot),
                    inputs: Some(vec![]),
                    outputs: Some(vec![Some(asm::NodeId(dest.to_string()))]),
                },
            });
        }
    }
    let src = enum_cast!(SchedTerm::Var { name, .. }, name, src);
    if f.is_var_or_ref(src) {
        asm::Command::Node(asm::NamedNode {
            name: None,
            node: asm::Node::LocalCopy {
                input: Some(asm::NodeId(src.clone())),
                output: Some(asm::NodeId(dest.to_string())),
            },
        })
    } else {
        asm::Command::Node(asm::NamedNode {
            name: None,
            node: asm::Node::WriteRef {
                source: Some(asm::NodeId(src.clone())),
                destination: Some(asm::NodeId(dest.to_string())),
                storage_type: Some(data_type_to_storage_type(f.get_dtype(dest).unwrap())),
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
    assert!(dest_tag.is_any_specified());
    let temp_node_name = temp_var_name(temp_id);
    let temp = asm::Command::Node(asm::NamedNode {
        name: Some(asm::NodeId(temp_node_name.clone())),
        node: asm::Node::AllocTemporary {
            place: Some(ir::Place::Local),
            buffer_flags: Some(LOCAL_TEMP_FLAGS),
            storage_type: Some(data_type_to_ffi_type(f.get_dtype(dest).unwrap())),
        },
    });
    let mv = build_copy_cmd(&temp_node_name, rhs, f, Some(dest_tag));
    let rd_ref = asm::Command::Node(asm::NamedNode {
        name: Some(asm::NodeId(dest.to_string())),
        node: asm::Node::ReadRef {
            source: Some(asm::NodeId(temp_node_name)),
            storage_type: Some(data_type_to_ffi_type(f.get_dtype(dest).unwrap())),
        },
    });
    (vec![Some(temp), Some(mv), Some(rd_ref)], temp_id + 1)
}

/// Lowers a variable declaration
fn lower_var_decl(
    dest: &str,
    dest_tag: &TripleTag,
    rhs: &Option<SchedTerm>,
    temp_id: usize,
    f: &Funclet,
) -> (CommandVec, usize) {
    let mut result = vec![Some(asm::Command::Node(asm::NamedNode {
        name: Some(asm::NodeId(dest.to_string())),
        node: asm::Node::AllocTemporary {
            place: Some(ir::Place::Local),
            buffer_flags: Some(LOCAL_TEMP_FLAGS),
            storage_type: Some(data_type_to_ffi_type(f.get_dtype(dest).unwrap())),
        },
    }))];
    if let Some(rhs) = rhs {
        result.push(Some(build_copy_cmd(dest, rhs, f, Some(dest_tag))));
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
        vec![Some(build_copy_cmd(lhs, rhs, f, Some(lhs_tags)))],
        temp_id,
    )
}

/// Changes the remote node id to the remote id of the result of the call, before
/// extracing the result
fn to_tuple_quotient(q: asm::RemoteNodeId) -> asm::RemoteNodeId {
    asm::RemoteNodeId {
        funclet: q.funclet,
        node: q.node.map(|o| o.map(|n| asm::NodeId(tuple_id(&[n.0])))),
    }
}

/// Lowers an operation into a local-do-external and read-ref
fn lower_op(
    dest: &str,
    dest_tag: &TripleTag,
    op: &str,
    args: &[SchedTerm],
    temp_id: usize,
    f: &Funclet,
) -> (CommandVec, usize) {
    let temp_node_name = temp_var_name(temp_id);
    let temp = asm::Command::Node(asm::NamedNode {
        name: Some(asm::NodeId(temp_node_name.clone())),
        node: asm::Node::AllocTemporary {
            place: Some(ir::Place::Local),
            buffer_flags: Some(LOCAL_TEMP_FLAGS),
            storage_type: Some(data_type_to_ffi_type(f.get_dtype(dest).unwrap())),
        },
    });
    let mut inputs = vec![];
    for arg in args {
        let arg = enum_cast!(SchedTerm::Var { name, .. }, name, arg);
        inputs.push(Some(asm::NodeId(arg.to_string())));
    }
    let local_do = asm::Command::Node(asm::NamedNode {
        name: None,
        node: asm::Node::LocalDoExternal {
            operation: Some(
                dest_tag
                    .value
                    .as_ref()
                    .map(|t| to_tuple_quotient(tag_to_quot(t)))
                    .expect("Tag must be set for now"),
            ),
            inputs: Some(inputs),
            outputs: Some(vec![Some(asm::NodeId(temp_node_name.clone()))]),
            external_function_id: Some(asm::ExternalFunctionId(op.to_string())),
        },
    });
    let read_ref = asm::Command::Node(asm::NamedNode {
        name: Some(asm::NodeId(dest.to_string())),
        node: asm::Node::ReadRef {
            source: Some(asm::NodeId(temp_node_name)),
            storage_type: Some(data_type_to_ffi_type(f.get_dtype(dest).unwrap())),
        },
    });
    (
        vec![Some(temp), Some(local_do), Some(read_ref)],
        temp_id + 1,
    )
}

fn lower_load(dest: &str, typ: &DataType, src: &str, temp_id: usize) -> (CommandVec, usize) {
    (
        vec![Some(asm::Command::Node(asm::NamedNode {
            name: Some(asm::NodeId(dest.to_string())),
            node: asm::Node::ReadRef {
                source: Some(asm::NodeId(src.to_string())),
                storage_type: Some(data_type_to_ffi_type(typ)),
            },
        }))],
        temp_id,
    )
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
            dest,
            dest_tag,
            op,
            args,
            ..
        } => lower_op(dest, dest_tag, &op.lower(), args, temp_id, f),
        x @ HirBody::Hole(_) => todo!("{x:?}"),
        HirBody::Phi { .. } => panic!("Attempting to lower intermediate form"),
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
    let tags = TagInfo::from(&call.tag);
    vec![
        Some(asm::Command::Node(asm::NamedNode {
            name: Some(asm::NodeId(djoin_name.clone())),
            node: asm::Node::DefaultJoin,
        })),
        Some(asm::Command::Node(asm::NamedNode {
            name: Some(asm::NodeId(join_var.clone())),
            // TODO: for greater generality, should be `SerializedJoin`, but I
            // think that's broken right now
            // TODO: optimize and use inline join whenever possible
            node: asm::Node::InlineJoin {
                funclet: f.next_blocks().first().unwrap().clone(),
                captures: Some(
                    captures
                        .iter()
                        .map(|x| Some(asm::NodeId(x.clone())))
                        .collect(),
                ),
                continuation: Some(asm::NodeId(djoin_name)),
            },
        })),
        Some(asm::Command::TailEdge(asm::TailEdge::ScheduleCall {
            timeline_operation: Some(tags.timeline.as_ref().map_or_else(
                || tags.default_tag(SpecType::Timeline).quot,
                |x| x.quot.clone(),
            )),
            spatial_operation: Some(tags.timeline.as_ref().map_or_else(
                || tags.default_tag(SpecType::Spatial).quot,
                |x| x.quot.clone(),
            )),
            value_operation: tags.value.map(|t| t.quot),
            callee_funclet_id: Some(asm::FuncletId(call.target.clone())),
            callee_arguments: Some(
                call.args
                    .iter()
                    .map(|x| Some(asm::NodeId(x.clone())))
                    .collect(),
            ),
            continuation_join: Some(asm::NodeId(join_var)),
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
fn lower_ret(rets: &[String], temp_id: usize, f: &Funclet) -> CommandVec {
    if f.is_final_return() {
        let djoin_id = temp_id;
        let djoin_name = temp_var_name(djoin_id);
        let join = temp_id + 1;
        let join_var = temp_var_name(join);
        assert_eq!(f.next_blocks().len(), 1);
        vec![
            Some(asm::Command::Node(asm::NamedNode {
                name: Some(asm::NodeId(djoin_name.clone())),
                node: asm::Node::DefaultJoin,
            })),
            Some(asm::Command::Node(asm::NamedNode {
                name: Some(asm::NodeId(join_var.clone())),
                node: asm::Node::InlineJoin {
                    funclet: f.next_blocks().first().unwrap().clone(),
                    captures: Some(vec![]),
                    continuation: Some(asm::NodeId(djoin_name)),
                },
            })),
            Some(asm::Command::TailEdge(asm::TailEdge::Jump {
                arguments: Some(rets.iter().map(|x| Some(asm::NodeId(x.clone()))).collect()),
                join: Some(asm::NodeId(join_var)),
            })),
        ]
    } else {
        vec![Some(asm::Command::TailEdge(asm::TailEdge::Return {
            return_values: Some(rets.iter().map(|x| Some(asm::NodeId(x.clone()))).collect()),
        }))]
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
        Terminator::Return { rets, .. } => lower_ret(rets, temp_id, f),
        Terminator::Next(vars) => {
            vec![Some(asm::Command::TailEdge(asm::TailEdge::Return {
                return_values: Some(vars.iter().map(|v| Some(asm::NodeId(v.clone()))).collect()),
            }))]
        }
        Terminator::FinalReturn(n) => vec![Some(asm::Command::TailEdge(asm::TailEdge::Return {
            return_values: Some(n.iter().map(|v| Some(asm::NodeId(v.clone()))).collect()),
        }))],
        Terminator::Select { guard, tag, .. } => lower_select(guard, tag, temp_id, f),
        // TODO: review this
        Terminator::None => panic!("None terminator not replaced by Next"),
        Terminator::Call(..) => panic!("Call not replaced by CaptureCall"),
        Terminator::CaptureCall { call, captures, .. } => {
            lower_func_call(call, captures, temp_id, f)
        }
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
        Some(asm::Command::Node(asm::NamedNode {
            name: Some(asm::NodeId(djoin_name.clone())),
            node: asm::Node::DefaultJoin,
        })),
        Some(asm::Command::Node(asm::NamedNode {
            name: Some(asm::NodeId(join_var.clone())),
            // TODO: for greater generality, should be `SerializedJoin`, but I
            // think that's broken right now
            // TODO: optimize and use inline join whenever possible
            node: asm::Node::InlineJoin {
                funclet: Some(f.join_funclet()),
                captures: Some(vec![]),
                continuation: Some(asm::NodeId(djoin_name)),
            },
        })),
        Some(asm::Command::TailEdge(asm::TailEdge::ScheduleSelect {
            value_operation: Some(
                tags.value
                    .as_ref()
                    .map(tag_to_quot)
                    .expect("Selects need a value node for now"),
            ),
            timeline_operation: Some(tags.timeline.as_ref().map_or_else(
                || {
                    asm::Quotient::None(Some(asm::RemoteNodeId {
                        node: None,
                        funclet: Some(f.specs().timeline.clone()),
                    }))
                },
                tag_to_quot,
            )),
            spatial_operation: Some(tags.spatial.as_ref().map_or_else(
                || {
                    asm::Quotient::None(Some(asm::RemoteNodeId {
                        node: None,
                        funclet: Some(f.specs().spatial.clone()),
                    }))
                },
                tag_to_quot,
            )),
            condition: Some(asm::NodeId(guard_name.to_string())),
            callee_funclet_ids: Some(f.next_blocks()),
            callee_arguments: Some(f.output_args()),
            continuation_join: Some(asm::NodeId(join_var)),
        })),
    ]
}

/// Converts a quotient reference (the part that refers to a variable in a spec)
/// to a remote node id in the assembly
fn quot_ref_to_remote_node(qr: &QuotientReference) -> asm::RemoteNodeId {
    asm::RemoteNodeId {
        node: qr.spec_var.clone().map(asm::NodeId),
        funclet: Some(asm::FuncletId(qr.spec_name.clone())),
    }
}

/// Gets the assembly quotient from a high level caiman tag
pub fn tag_to_quot(t: &Tag) -> asm::Quotient {
    t.quot.as_ref().map_or_else(
        || asm::Quotient::None(t.quot_var.as_ref().map(quot_ref_to_remote_node)),
        |x| match x {
            Quotient::Node => asm::Quotient::Node(t.quot_var.as_ref().map(quot_ref_to_remote_node)),
            Quotient::Input => {
                asm::Quotient::Input(t.quot_var.as_ref().map(quot_ref_to_remote_node))
            }
            Quotient::Output => {
                asm::Quotient::Output(t.quot_var.as_ref().map(quot_ref_to_remote_node))
            }
            Quotient::None => asm::Quotient::None(t.quot_var.as_ref().map(quot_ref_to_remote_node)),
        },
    )
}

/// Converts a hlc tag to a tag in the assembly
pub fn tag_to_tag(t: &Tag) -> asm::Tag {
    tag_to_tag_def(t, ir::Flow::Usable)
}

/// Converts a hlc tag to a tag in the assembly, using a default flow
/// if the tag does not specify a flow
pub fn tag_to_tag_def(t: &Tag, default_flow: ir::Flow) -> asm::Tag {
    asm::Tag {
        quot: tag_to_quot(t),
        flow: t.flow.as_ref().map_or(default_flow, |f| match f {
            Flow::Dead => ir::Flow::Dead,
            Flow::Need => ir::Flow::Need,
            Flow::Usable => ir::Flow::Usable,
            Flow::Saved => ir::Flow::Saved,
        }),
    }
}

/// Lowers a basic block into a caiman assembly funclet
///
fn lower_block(funclet: &Funclet<'_>) -> asm::Funclet {
    let mut commands = vec![];
    let mut temp_id = 0;
    for cmd in funclet.stmts() {
        let (mut new_cmds, new_id) = lower_instr(cmd, temp_id, funclet);
        temp_id = new_id;
        commands.append(&mut new_cmds);
    }
    commands.extend(lower_terminator(funclet.terminator(), temp_id, funclet));
    asm::Funclet {
        kind: ir::FuncletKind::ScheduleExplicit,
        header: asm::FuncletHeader {
            name: asm::FuncletId(funclet.name()),
            args: funclet.inputs(),
            ret: funclet.outputs(),
            binding: asm::FuncletBinding::ScheduleBinding(asm::ScheduleBinding {
                implicit_tags: None,
                value: Some(funclet.specs().value.clone()),
                timeline: Some(funclet.specs().timeline.clone()),
                spatial: Some(funclet.specs().spatial.clone()),
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
