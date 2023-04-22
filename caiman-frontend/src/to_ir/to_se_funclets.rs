use super::ir_funclets::{InnerFunclet, ScheduleExplicitFunclet};
//use super::ir_typing::{IRType /*vl_type_to_asm_type*/, IRTypesIndex};
//use super::vil::{self, Expr, Value};
use caiman::assembly_ast as asm;
//use super::error::ToIRResult;
use super::context;
use super::dual_compatibility::MatchedScheduleStmt;
use super::label;
use caiman::ir;

pub fn schedule_ast_to_schedule_explicit_funclets(
    program: &Vec<MatchedScheduleStmt>,
    context: &mut context::Context,
) -> Vec<ScheduleExplicitFunclet>
{
    // XXX For now, we just have a big global funclet. Here's its name
    let global_sef_name = "my_great_scheduleexplicitfunclet".to_string();
    context.begin_local_funclet(global_sef_name.clone());

    // For making the last thing into what's returned (very temporary)
    // Remove later. Also these are default values
    let mut last_alloc_id = 0;
    let mut last_type = asm::Type::Local("".to_string());

    let mut commands: Vec<Option<asm::Node>> = Vec::new();
    for (i, mss) in program.iter().enumerate()
    {
        // Just AllocTemporary and EncodeDo each of these (as alloc statements are
        // not in the lang yet)
        // We would ordinarily match in mss.schedule_stmt but we don't need to until there
        // are statements that are more intersting like alloc

        let place_filling = ir::Place::Local;

        // TODO this is hard coded!!! should convert mss.vl_type
        let value_type = asm::Type::FFI(asm::FFIType::I32);
        let slot_str = context.add_slot(
            value_type.clone(),
            place_filling.clone(),
            ir::ResourceQueueStage::Ready,
        );
        last_type = asm::Type::Local(slot_str);

        let place = Some(place_filling);

        // TODO find funclet id... once there are multiple funclets
        let operation = Some(asm::RemoteNodeId {
            funclet_id: "my_great_valuefunclet".to_string(),
            node_id: label::label_node(mss.vil_index),
        });
        let operation_cloned = operation.clone();
        let storage_type = Some(value_type);
        commands.push(Some(asm::Node::AllocTemporary { place, operation, storage_type }));
        context.add_node();
        last_alloc_id = i * 2;
        commands.push(Some(asm::Node::EncodeDo {
            place,
            operation: operation_cloned,
            inputs: Some(Box::new([])),
            outputs: Some(Box::new([Some(label::label_node(i * 2)) /* The slot */])),
        }));
        context.add_node();
    }
    // TODO make real header & tail
    let ret_name = "out".to_string();
    let dummy_header = asm::FuncletHeader {
        ret: vec![(Some(ret_name.clone()), last_type)],
        name: global_sef_name,
        args: vec![],
    };
    context.add_return(ret_name);

    // TODO don't simply return the last node like below
    let last_id_str = label::label_node(last_alloc_id);
    let dummy_tail_edge =
        Some(asm::TailEdge::Return { return_values: Some(vec![Some(last_id_str.clone())]) });

    context.end_local_funclet();
    vec![ScheduleExplicitFunclet {
        inner_funclet: InnerFunclet { header: dummy_header, commands, tail_edge: dummy_tail_edge },
    }]
}
