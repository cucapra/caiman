use super::ir_funclets::{InnerFunclet, ScheduleExplicitFunclet};
//use super::ir_typing::{IRType /*vl_type_to_asm_type*/, IRTypesIndex};
//use super::vil::{self, Expr, Value};
use caiman::assembly_ast as asm;
//use super::error::ToIRResult;
use super::dual_compatibility::MatchedScheduleStmt;
use super::label;
use caiman::ir;

/*
pub struct MatchedScheduleStmt
{
    pub schedule_stmt: ast::ParsedStmt,
    pub vil_index: usize,
}
* [*/

pub fn schedule_ast_to_schedule_explicit_funclets(
    program: &Vec<MatchedScheduleStmt>,
) -> Vec<ScheduleExplicitFunclet>
{
    let mut commands: Vec<Option<asm::Node>> = Vec::new();
    for (i, mss) in program.iter().enumerate()
    {
        // Just AllocTemporary and EncodeDo each of these (as alloc statements are
        // not in the lang yet)
        // We would ordinarily match in mss.schedule_stmt but we don't need to until there
        // are statements that are more intersting like alloc
        let storage_type = Some(asm::Type::Local(label::label_slot(i)));
        let place = Some(ir::Place::Local);
        // TODO find funclet id... once there are multiple funclets
        let operation = Some(asm::RemoteNodeId {
            funclet_id: "my_great_valuefunclet".to_string(),
            node_id: label::label_node_index(mss.vil_index),
        });
        let operation_cloned = operation.clone();
        commands.push(Some(asm::Node::AllocTemporary { place, operation, storage_type }));
        commands.push(Some(asm::Node::EncodeDo {
            place,
            operation: operation_cloned,
            // Also TODO
            inputs: None,
            outputs: None,
        }));
    }
    // TODO make real header & tail
    let dummy_header = asm::FuncletHeader {
        ret: vec![],
        name: "my_great_scheduleexplicitfunclet".to_string(),
        args: vec![],
    };
    let dummy_tail_edge = None; 
    vec![ScheduleExplicitFunclet {
        inner_funclet: InnerFunclet {
            header: dummy_header,
            commands,
            tail_edge: dummy_tail_edge,
        },
    }]
}
