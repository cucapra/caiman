use std::collections::BTreeSet;

use caiman::assembly::ast::{self as asm, FuncletArgument, Hole};

use crate::{
    enum_cast,
    error::{type_error, LocalError},
    lower::{data_type_to_ffi_type, sched_hir::TagInfo},
    parse::ast::{
        Arg, DataType, Flow, FullType, Quotient, QuotientReference, SchedTerm, SchedulingFunc, Tag,
    },
    typing::{Context, SpecType},
};
use caiman::ir;

use super::{
    sched_hir::{Funclet, Funclets, HirBody, HirFuncCall, Specs, Terminator, RET_VAR},
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

/// Lowers a flattened declaration statement into a caiman assembly command
/// # Returns
/// A tuple containing the commands that implement the statement
/// and the next available temporary id
fn lower_flat_decl(
    dest: &str,
    dest_tag: &Option<FullType>,
    rhs: &SchedTerm,
    temp_id: usize,
) -> (CommandVec, usize) {
    if let SchedTerm::Lit { .. } = rhs {
        let dest_tag = dest_tag
            .as_ref()
            .expect("We require all variables to have type annotations");
        assert!(!dest_tag.tags.is_empty());
        let temp_node_name = temp_var_name(temp_id);
        let temp = asm::Command::Node(asm::NamedNode {
            name: Some(asm::NodeId(temp_node_name.clone())),
            node: asm::Node::AllocTemporary {
                place: Some(ir::Place::Local),
                buffer_flags: Some(ir::BufferFlags::new()),
                storage_type: Some(data_type_to_ffi_type(&dest_tag.base.as_ref().unwrap().base)),
            },
        });
        let mv = asm::Command::Node(asm::NamedNode {
            name: None,
            node: asm::Node::LocalDoBuiltin {
                operation: Some(tag_to_quot(&dest_tag.tags[0])),
                // no inputs
                inputs: Some(Vec::new()),
                outputs: Some(vec![Some(asm::NodeId(temp_node_name.clone()))]),
            },
        });
        let rd_ref = asm::Command::Node(asm::NamedNode {
            name: Some(asm::NodeId(dest.to_string())),
            node: asm::Node::ReadRef {
                source: Some(asm::NodeId(temp_node_name)),
                storage_type: Some(data_type_to_ffi_type(&dest_tag.base.as_ref().unwrap().base)),
            },
        });
        (vec![Some(temp), Some(mv), Some(rd_ref)], temp_id + 1)
    } else {
        todo!()
    }
}

/// Lowers a variable declaration
fn lower_var_decl(
    dest: &str,
    dest_tag: &Option<FullType>,
    rhs: &Option<SchedTerm>,
    temp_id: usize,
    f: &Funclet,
) -> (CommandVec, usize) {
    let dest_tag = dest_tag
        .as_ref()
        .expect("We require all variables to have type annotations");
    let mut result = vec![Some(asm::Command::Node(asm::NamedNode {
        name: Some(asm::NodeId(dest.to_string())),
        node: asm::Node::AllocTemporary {
            place: Some(ir::Place::Local),
            buffer_flags: Some(ir::BufferFlags::new()),
            storage_type: Some(data_type_to_ffi_type(&dest_tag.base.as_ref().unwrap().base)),
        },
    }))];
    if let Some(SchedTerm::Lit { tag: rhs_tag, .. }) = rhs {
        let rhs_tag = rhs_tag.as_ref().map_or_else(
            || TagInfo::from(dest_tag, f.specs()),
            |rhs_tag| TagInfo::from_tags(rhs_tag, f.specs()),
        );
        assert!(rhs_tag.value.is_some());
        result.push(Some(asm::Command::Node(asm::NamedNode {
            name: None,
            node: asm::Node::LocalDoBuiltin {
                operation: Some(rhs_tag.value.as_ref().unwrap().quot.clone()),
                // no inputs
                inputs: Some(Vec::new()),
                outputs: Some(vec![Some(asm::NodeId(dest.to_string()))]),
            },
        })));
    }
    (result, temp_id)
}

/// Lowers a store lhs <- rhs
fn lower_store(lhs: &str, rhs: &SchedTerm, temp_id: usize, f: &Funclet) -> (CommandVec, usize) {
    let rhs = enum_cast!(SchedTerm::Var { name, .. }, name, rhs);
    (
        vec![Some(asm::Command::Node(asm::NamedNode {
            name: None,
            node: asm::Node::LocalDoBuiltin {
                operation: Some(
                    f.get_out_tag(rhs)
                        .unwrap()
                        .value
                        .expect("Tag must be set")
                        .quot,
                ),
                // no inputs
                inputs: Some(Vec::new()),
                outputs: Some(vec![Some(asm::NodeId(lhs.to_string()))]),
            },
        }))],
        temp_id,
    )
}

/// Lowers an operation into a local-do-external and read-ref
fn lower_op(
    dest: &str,
    dest_tag: &Option<FullType>,
    op: &str,
    args: &[SchedTerm],
    temp_id: usize,
    f: &Funclet,
) -> (CommandVec, usize) {
    let dest_tag = dest_tag
        .as_ref()
        .expect("We require all variables to have type annotations");
    let temp_node_name = temp_var_name(temp_id);
    let temp = asm::Command::Node(asm::NamedNode {
        name: Some(asm::NodeId(temp_node_name.clone())),
        node: asm::Node::AllocTemporary {
            place: Some(ir::Place::Local),
            buffer_flags: Some(ir::BufferFlags::new()),
            storage_type: Some(data_type_to_ffi_type(&dest_tag.base.as_ref().unwrap().base)),
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
            operation: unextract_quotient(get_quotient(f.specs(), &dest_tag.tags, SpecType::Value)),
            inputs: Some(inputs),
            outputs: Some(vec![Some(asm::NodeId(temp_node_name.clone()))]),
            external_function_id: Some(asm::ExternalFunctionId(op.to_string())),
        },
    });
    let read_ref = asm::Command::Node(asm::NamedNode {
        name: Some(asm::NodeId(dest.to_string())),
        node: asm::Node::ReadRef {
            source: Some(asm::NodeId(temp_node_name)),
            storage_type: Some(data_type_to_ffi_type(&dest_tag.base.as_ref().unwrap().base)),
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
        } => lower_flat_decl(lhs, lhs_tag, rhs, temp_id),
        HirBody::VarDecl {
            lhs, lhs_tag, rhs, ..
        } => lower_var_decl(lhs, lhs_tag, rhs, temp_id, f),
        HirBody::RefStore { lhs, rhs, .. } => lower_store(lhs, rhs, temp_id, f),
        HirBody::RefLoad { dest, src, typ } => lower_load(dest, typ, src, temp_id),
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
    }
}

/// Gets the quotient for a particular spec type from a list of tags
fn get_quotient_opt(
    specs: &Specs,
    tag: &Option<Vec<Tag>>,
    qtype: SpecType,
) -> asm::Hole<asm::Quotient> {
    if let Some(tags) = tag {
        for t in tags {
            if let res @ Some(_) = t.quot_var.as_ref().and_then(|qr| {
                if qr.spec_name == specs.value.0 && qtype == SpecType::Value {
                    return Some(tag_to_quot(t));
                }
                if qr.spec_name == specs.timeline.0 && qtype == SpecType::Timeline {
                    return Some(tag_to_quot(t));
                }
                if qr.spec_name == specs.spatial.0 && qtype == SpecType::Spatial {
                    return Some(tag_to_quot(t));
                }
                None
            }) {
                return res;
            }
        }
    }
    None
}

/// Changes the remote node id to the remote id of the result of the call, before
/// extracing the result
fn unextract_quotient(q: Hole<asm::Quotient>) -> Hole<asm::Quotient> {
    match q {
        Hole::Some(
            asm::Quotient::Node(Some(asm::RemoteNodeId {
                funclet,
                node: Some(node),
            }))
            | asm::Quotient::Input(Some(asm::RemoteNodeId {
                funclet,
                node: Some(node),
            }))
            | asm::Quotient::Output(Some(asm::RemoteNodeId {
                funclet,
                node: Some(node),
            })),
        ) => Hole::Some(asm::Quotient::Node(Some(asm::RemoteNodeId {
            funclet,
            node: Some(asm::NodeId(tuple_id(&[node.0]))),
        }))),
        x => x,
    }
}

fn get_quotient(specs: &Specs, tags: &[Tag], qtype: SpecType) -> asm::Hole<asm::Quotient> {
    for t in tags {
        if let res @ Some(_) = t.quot_var.as_ref().and_then(|qr| {
            if qr.spec_name == specs.value.0 && qtype == SpecType::Value {
                return Some(tag_to_quot(t));
            }
            if qr.spec_name == specs.timeline.0 && qtype == SpecType::Timeline {
                return Some(tag_to_quot(t));
            }
            if qr.spec_name == specs.spatial.0 && qtype == SpecType::Spatial {
                return Some(tag_to_quot(t));
            }
            None
        }) {
            return res;
        }
    }
    None
}

/// Changes the remote node id to the remote id of the result of the call
fn get_tuple_quot(t: Option<asm::Tag>) -> asm::Hole<asm::Quotient> {
    t.map(|t| match t.quot {
        asm::Quotient::Node(Some(asm::RemoteNodeId {
            funclet,
            node: Some(node),
        })) => asm::Quotient::Node(Some(asm::RemoteNodeId {
            funclet,
            node: Some(asm::NodeId(tuple_id(&[node.0]))),
        })),
        asm::Quotient::Input(Some(asm::RemoteNodeId {
            funclet,
            node: Some(node),
        })) => asm::Quotient::Input(Some(asm::RemoteNodeId {
            funclet,
            node: Some(asm::NodeId(tuple_id(&[node.0]))),
        })),
        asm::Quotient::Output(Some(asm::RemoteNodeId {
            funclet,
            node: Some(node),
        })) => asm::Quotient::Output(Some(asm::RemoteNodeId {
            funclet,
            node: Some(asm::NodeId(tuple_id(&[node.0]))),
        })),
        x => x,
    })
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
    let tags = TagInfo::from_maybe_tags(&call.tag, f.specs());
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
            value_operation: get_tuple_quot(tags.value),
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
            return_values: Some(
                (0..*n)
                    .map(|idx| Some(asm::NodeId(format!("{RET_VAR}{idx}"))))
                    .collect(),
            ),
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
fn lower_select(
    guard_name: &str,
    tags: &Option<Vec<Tag>>,
    temp_id: usize,
    f: &Funclet<'_>,
) -> CommandVec {
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
                get_quotient_opt(f.specs(), tags, SpecType::Value)
                    .expect("Selects need a value node for now"),
            ),
            timeline_operation: Some(
                get_quotient_opt(f.specs(), tags, SpecType::Timeline).unwrap_or_else(|| {
                    asm::Quotient::None(Some(asm::RemoteNodeId {
                        node: None,
                        funclet: Some(f.specs().timeline.clone()),
                    }))
                }),
            ),
            spatial_operation: Some(
                get_quotient_opt(f.specs(), tags, SpecType::Spatial).unwrap_or_else(|| {
                    asm::Quotient::None(Some(asm::RemoteNodeId {
                        node: None,
                        funclet: Some(f.specs().spatial.clone()),
                    }))
                }),
            ),
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
    asm::Tag {
        quot: tag_to_quot(t),
        // TODO: this is a mistake in the IR/assembly, the flow should be able
        // to be a hole
        flow: t.flow.as_ref().map_or(ir::Flow::Usable, |f| match f {
            Flow::Dead => ir::Flow::Dead,
            Flow::Need => ir::Flow::Need,
            Flow::Usable => ir::Flow::Usable,
            Flow::Save => ir::Flow::Save,
        }),
    }
}

/// Converts a high level caiman function argument into a funclet argument
pub fn hlc_arg_to_asm_arg(arg: &Arg<FullType>) -> FuncletArgument {
    let ft = &arg.1;
    FuncletArgument {
        name: Some(asm::NodeId(arg.0.to_string())),
        typ: super::data_type_to_local_type(&ft.base.as_ref().unwrap().base),
        tags: ft.tags.iter().map(tag_to_tag).collect(),
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
    let blocks = Funclets::new(func, specs, ctx);
    Ok(blocks.funclets().iter().map(lower_block).collect())
}
