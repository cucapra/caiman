use caiman::assembly::ast::{self as asm, FuncletArgument};

use crate::{
    error::{type_error, LocalError},
    parse::ast::{
        self, Arg, FlaggedType, Flow, FullType, Quotient, QuotientReference, SchedStmt,
        SchedulingFunc, Tag,
    },
};
use caiman::ir;

use super::{
    cfg::{BasicBlock, Cfg},
    global_context::{Context, SpecType},
};

fn lower_instr(s: &SchedStmt) -> asm::Command {
    todo!()
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

/// Converts a hlc tag to an tag in the assembly
fn tag_to_tag(t: &Tag) -> asm::Tag {
    asm::Tag {
        quot: t
            .quot
            .as_ref()
            .map(|x| match x {
                Quotient::Node => {
                    asm::Quotient::Node(t.quot_var.as_ref().map(quot_ref_to_remote_node))
                }
                Quotient::Input => {
                    asm::Quotient::Input(t.quot_var.as_ref().map(quot_ref_to_remote_node))
                }
                Quotient::Output => {
                    asm::Quotient::Output(t.quot_var.as_ref().map(quot_ref_to_remote_node))
                }
                Quotient::None => asm::Quotient::None,
            })
            .unwrap_or(asm::Quotient::None),
        flow: t.flow.as_ref().map_or(ir::Flow::None, |f| match f {
            Flow::Dead => ir::Flow::None,
            Flow::Need => ir::Flow::Need,
            Flow::Usable => ir::Flow::Have,
            Flow::Save => ir::Flow::Met,
        }),
    }
}

fn hlc_arg_to_asm_arg(arg: &Arg<FullType>) -> FuncletArgument {
    let ft = &arg.1;
    FuncletArgument {
        name: Some(asm::NodeId(arg.0.to_string())),
        typ: super::data_type_to_type(&ft.base.base),
        tags: ft.tags.iter().map(tag_to_tag).collect(),
    }
}

struct FuncInfo {
    name: String,
    input: Vec<Arg<FullType>>,
    output: Arg<FullType>,
}

fn lower_block(func: &FuncInfo, specs: &Specs, cfg: &Cfg, b: &BasicBlock) -> asm::Funclet {
    let commands = b.stmts.iter().map(lower_instr).map(|x| Some(x)).collect();
    asm::Funclet {
        kind: ir::FuncletKind::ScheduleExplicit,
        header: asm::FuncletHeader {
            name: asm::FuncletId(func.name.clone()),
            args: func.input.iter().map(hlc_arg_to_asm_arg).collect(),
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

/// Lower a scheduling function into a caiman assembly funclet.
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
        output: (String::new(), func.output),
    };
    Ok(cfg
        .blocks
        .iter()
        .map(|bb| lower_block(&finfo, &specs, &cfg, bb))
        .collect())
}
