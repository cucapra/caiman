use std::collections::HashMap;

use crate::scheduling_language::ast as schedule_ast;
//use crate::spec;
use crate::value_language::ast as value_ast;
//use crate::value_language::typing;
//use caiman::arena::Arena;
use caiman::assembly_ast as asm;
use caiman::assembly_context as asm_ctx;
use caiman::ir;
//use std::collections::HashMap;

mod context;
mod label;

mod dual_compatibility;
mod index;

mod ir_typing;

mod ir_funclets;

mod to_value_funclets;

mod to_se_funclets;

pub mod to_vil;
pub mod vil;

mod error;
pub use error::ToIRError;
use error::{make_error, ToIRResult};

pub fn go(
    value_ast: &value_ast::TypedProgram,
    schedule_ast: &schedule_ast::ParsedProgram,
) -> ToIRResult<asm::Program>
{
    let mut context = context::Context::new();

    let vil_program = to_vil::value_ast_to_vil(value_ast);
    let matched_schedule_stmts =
        dual_compatibility::match_vil_to_scheduling(&vil_program, schedule_ast)?;

    let value_funclets = to_value_funclets::vil_to_value_funclets(&vil_program, &mut context);
    let schedule_explicit_funclets = to_se_funclets::schedule_ast_to_schedule_explicit_funclets(
        &matched_schedule_stmts,
        &mut context,
    );

    let mut funclets: Vec<asm::FuncletDef> = value_funclets
        .into_iter()
        .map(ir_funclets::make_asm_funclet)
        .chain(schedule_explicit_funclets.into_iter().map(ir_funclets::make_asm_funclet))
        .collect();
    funclets.push(dummy_timeline_funclet(&mut context));

    let main_pipeline = asm::Pipeline {
        name: "main".to_string(),
        funclet: "my_great_scheduleexplicitfunclet".to_string(),
    };

    let (context, types) = context.into_context_and_types();

    let version = asm::Version { major: 0, minor: 0, detailed: 1 };
    Ok(asm::Program {
        context,
        version,
        funclets,
        types,
        pipelines: vec![main_pipeline],
        extras: dummy_extras(),
    })
}

fn dummy_timeline_funclet(context: &mut context::Context) -> asm::FuncletDef
{
    let funclet_name = "my_great_timelinefunclet".to_string();
    context.begin_local_funclet(funclet_name.clone());

    let arg_type_str = context.add_event(ir::Place::Local);
    let arg_str = "e".to_string();
    context.add_arg(arg_str.clone());
    let arg_type_local = asm::Type::Local(arg_type_str);

    let tail_edge =
        Some(asm::TailEdge::Return { return_values: Some(vec![Some(arg_str.clone())]) });

    // For some reason, the trivial.cair data structure has this funclets as its current
    // funclet!! Weird. But the below line is always here if it turns out that's wrong
    //context.end_local_funclet();
    asm::FuncletDef::Local(asm::Funclet {
        kind: ir::FuncletKind::Timeline,
        header: asm::FuncletHeader {
            args: vec![(Some(arg_str), arg_type_local.clone())],
            ret: vec![(None, arg_type_local)],
            name: funclet_name,
        },
        commands: Vec::new(),
        tail_edge,
    })
}

fn dummy_extras() -> asm::Extras
{
    let mut data: asm::UncheckedDict = HashMap::new();
    let mut data_insert = |s: &str, v| data.insert(asm::Value::ID(s.to_string()), v);
    data_insert(
        "value",
        asm::DictValue::Raw(asm::Value::FnName("my_great_valuefunclet".to_string())),
    );
    let empty_dict = asm::DictValue::Dict(HashMap::new());
    data_insert("input_slots", empty_dict.clone());
    let mut output_slots_dict: asm::UncheckedDict = HashMap::new();
    output_slots_dict.insert(
        asm::Value::VarName("out".to_string()),
        asm::DictValue::Raw(asm::Value::SlotInfo(asm::SlotInfo {
            value_tag: asm::ValueTag::Core(asm::TagCore::None),
            spatial_tag: asm::SpatialTag::Core(asm::TagCore::None),
            timeline_tag: asm::TimelineTag::Core(asm::TagCore::None),
        })),
    );
    data_insert("output_slots", asm::DictValue::Dict(output_slots_dict));
    data_insert("input_fences", empty_dict.clone());
    data_insert("output_fences", empty_dict.clone());
    data_insert("input_buffers", empty_dict.clone());
    data_insert("output_buffers", empty_dict.clone());
    let e_tag = asm::DictValue::Raw(asm::Value::Tag(asm::Tag::TimelineTag(
        asm::TimelineTag::Core(asm::TagCore::Input(asm::RemoteNodeId {
            funclet_id: "my_great_timelinefunclet".to_string(),
            node_id: "e".to_string(),
        })),
    )));
    data_insert("in_timeline_tag", e_tag.clone());
    data_insert("out_timeline_tag", e_tag);
    vec![asm::Extra { name: "my_great_scheduleexplicitfunclet".to_string(), data }]
}
