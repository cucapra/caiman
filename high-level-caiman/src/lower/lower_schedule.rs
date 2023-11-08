use caiman::assembly::ast::{self as asm, FuncletArgument, Hole};

use crate::{
    error::{type_error, LocalError},
    parse::ast::{
        Arg, Flow, FullType, NestedExpr, Quotient, QuotientReference, SchedLiteral, SchedStmt,
        SchedTerm, SchedulingFunc, Tag,
    },
};
use caiman::ir;

use super::{
    cfg::{BasicBlock, Cfg, Terminator},
    global_context::{Context, SpecType},
};

/// A vector of commands with holes.
/// A hole in a command means `???`
type CommandVec = Vec<Hole<asm::Command>>;

/// Lowers a flattened declaration statement into a caiman assembly command
/// # Returns
/// A tuple containing the commands that implement the statement
/// and the next available temporary id
fn lower_flat_decl(
    dest: &str,
    dest_tag: Option<FullType>,
    rhs: &NestedExpr<SchedTerm>,
    temp_id: usize,
) -> (CommandVec, usize) {
    if let NestedExpr::Term(SchedTerm::Lit {
        lit: SchedLiteral::Int(_x),
        tag: _tag,
        ..
    }) = rhs
    {
        let dest_tag = dest_tag.expect("We require all variables to have type annotations");
        assert_eq!(dest_tag.tags.len(), 1);
        let temp_node_name = format!("_t{temp_id}");
        let temp = asm::Command::Node(asm::NamedNode {
            name: Some(asm::NodeId(temp_node_name.clone())),
            node: asm::Node::AllocTemporary {
                place: Some(ir::Place::Local),
                buffer_flags: Some(ir::BufferFlags::new()),
                storage_type: Some(asm::TypeId::FFI(asm::FFIType::I64)),
            },
        });
        let mv = asm::Command::Node(asm::NamedNode {
            name: Some(asm::NodeId(format!("_t{}", temp_id + 1))),
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
                storage_type: Some(asm::TypeId::FFI(asm::FFIType::I64)),
            },
        });
        (vec![Some(temp), Some(mv), Some(rd_ref)], temp_id + 2)
    } else {
        todo!()
    }
}

/// Lowers a scheduling statement into a caiman assembly command
/// # Returns
/// A tuple containing the commands that implement the statement
/// and the next available temporary id
fn lower_instr(s: &SchedStmt, temp_id: usize) -> (CommandVec, usize) {
    match s {
        SchedStmt::Decl { lhs, expr, .. } => {
            assert_eq!(lhs.len(), 1);
            lower_flat_decl(&lhs[0].0, lhs[0].1.clone(), expr, temp_id)
        }
        SchedStmt::Return(..) => panic!("Return should be a terminator"),
        _ => todo!(),
    }
}

/// Lowers a basic block terminator into a caiman assembly command
/// # Returns
/// A tuple containing the commands that implement the terminator
/// and the next available temporary id
fn lower_terminator(t: &Terminator, temp_id: usize, _cfg: &Cfg) -> (CommandVec, usize) {
    match t {
        Terminator::Return(Some(NestedExpr::Term(SchedTerm::Var { name, .. }))) => (
            vec![Some(asm::Command::TailEdge(asm::TailEdge::Return {
                return_values: Some(vec![Some(asm::NodeId(name.clone()))]),
            }))],
            temp_id,
        ),
        Terminator::Return(_) => panic!("Return not flattened or its a void return!"),
        _ => todo!(),
    }
}

/// Scheduling funclet specs
struct Specs {
    value: asm::FuncletId,
    timeline: asm::FuncletId,
    spatial: asm::FuncletId,
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
fn tag_to_quot(t: &Tag) -> asm::Quotient {
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
fn tag_to_tag(t: &Tag) -> asm::Tag {
    asm::Tag {
        quot: tag_to_quot(t),
        flow: t.flow.as_ref().map_or(ir::Flow::Usable, |f| match f {
            Flow::Dead => ir::Flow::Dead,
            Flow::Need => ir::Flow::Need,
            Flow::Usable => ir::Flow::Usable,
            Flow::Save => ir::Flow::Save,
        }),
    }
}

/// Converts a high level caiman function argument into a funclet argument
fn hlc_arg_to_asm_arg(arg: &Arg<FullType>) -> FuncletArgument {
    let ft = &arg.1;
    FuncletArgument {
        name: Some(asm::NodeId(arg.0.to_string())),
        typ: super::data_type_to_type(&ft.base.base),
        tags: ft.tags.iter().map(tag_to_tag).collect(),
    }
}

/// Information about a high level caiman function
struct FuncInfo {
    name: String,
    input: Vec<Arg<FullType>>,
    output: Arg<FullType>,
}

/// Lowers a basic block into a caiman assembly funclet
///
fn lower_block(func: &FuncInfo, specs: &Specs, cfg: &Cfg, b: &BasicBlock) -> asm::Funclet {
    let mut commands = vec![];
    let mut temp_id = 0;
    for cmd in &b.stmts {
        let (mut new_cmds, new_id) = lower_instr(cmd, temp_id);
        temp_id = new_id;
        commands.append(&mut new_cmds);
    }
    commands.append(&mut lower_terminator(&b.terminator, temp_id, cfg).0);
    asm::Funclet {
        kind: ir::FuncletKind::ScheduleExplicit,
        header: asm::FuncletHeader {
            // TODO: this only works when the function is a single basic block
            name: asm::FuncletId(func.name.clone()),
            // TODO: this only works when the function is a single basic block
            args: func.input.iter().map(hlc_arg_to_asm_arg).collect(),
            // TODO: this only works when the function is a single basic block
            ret: vec![hlc_arg_to_asm_arg(&func.output)],
            binding: asm::FuncletBinding::ScheduleBinding(asm::ScheduleBinding {
                implicit_tags: None,
                value: Some(specs.value.clone()),
                timeline: Some(specs.timeline.clone()),
                spatial: Some(specs.spatial.clone()),
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
    let cfg = Cfg::new(func.statements);
    let mut val = None;
    let mut timeline = None;
    let mut spatial = None;
    if func.specs.len() > 3 {
        return Err(type_error(func.info, "Too many specs"));
    }
    for spec in &func.specs {
        match ctx.specs.get(spec) {
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
    let finfo = FuncInfo {
        name: func.name,
        input: func.input,
        output: (
            String::new(),
            func.output.expect("Functions must return values for now"),
        ),
    };
    Ok(cfg
        .blocks
        .iter()
        .map(|bb| lower_block(&finfo, &specs, &cfg, bb))
        .collect())
}
