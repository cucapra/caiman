use super::ir_funclets::{InnerFunclet, ScheduleExplicitFunclet};
//use super::ir_typing::{IRType /*vl_type_to_asm_type*/, IRTypesIndex};
//use super::vil::{self, Expr, Value};
use caiman::assembly::ast as asm;
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
    let global_sef_name = asm::FuncletId("my_great_scheduleexplicitfunclet".to_string());

    // For making the last thing into what's returned (very temporary)
    // Remove later. Also these are default values
    let mut last_alloc_id = 0;
    let mut last_type = asm::TypeId::Local("".to_string());

    let mut node_context = context::NodeContext::new();
    for (i, mss) in program.iter().enumerate()
    {
        // Just AllocTemporary and EncodeDo each of these (as alloc statements are
        // not in the lang yet)
        // We would ordinarily match in mss.schedule_stmt but we don't need to until there
        // are statements that are more intersting like alloc

        let place_filling = ir::Place::Local;

        // TODO this is hard coded!!! should convert mss.vl_type
        let value_type = asm::TypeId::FFI(asm::FFIType::I32);
        let slot_str = context.add_slot(
            value_type.clone(),
            place_filling.clone(),
            ir::ResourceQueueStage::Ready,
        );
        last_type = asm::TypeId::Local(slot_str);

        let place = Some(place_filling);

        // TODO find funclet id... once there are multiple funclets
        let operation = Some(asm::RemoteNodeId {
            funclet_name: Some(asm::FuncletId("my_great_valuefunclet".to_string())),
            node_name: Some(label::label_node(mss.vil_index)),
        });
        let operation_cloned = operation.clone();
        let storage_type = Some(value_type);
        node_context.add_node(Some(asm::Node::AllocTemporary { place, operation, storage_type }));
        last_alloc_id = i * 2;
        node_context.add_node(Some(asm::Node::EncodeDo {
            place,
            operation: operation_cloned,
            inputs: Some(Vec::new()),
            outputs: Some(vec![Some(label::label_node(i * 2)) /* The slot */]),
        }));
    }
    // TODO make real header & tail
    let ret_name = asm::NodeId("out".to_string());
    let timeline_funclet_name = "my_great_timelinefunclet";
    let timeline_node_name = "e";
    let timeline_rmi =  asm::RemoteNodeId {
        funclet_name: Some(asm::FuncletId(timeline_funclet_name.to_string())),
        node_name: Some(asm::NodeId(timeline_node_name.to_string())),
    };
    let dummy_header = asm::FuncletHeader {
        ret: vec![asm::FuncletArgument {
            name: Some(ret_name.clone()),
            typ: last_type,
            tags: Vec::new(),
        }],
        name: global_sef_name,
        args: vec![],
        // TODO also obviously hard coded
        binding: asm::FuncletBinding::ScheduleBinding(asm::ScheduleBinding {
            implicit_tags: Some((
                asm::Tag::Input(timeline_rmi.clone()),
                asm::Tag::Output(timeline_rmi)
            )),
            value: Some(asm::FuncletId("my_great_valuefunclet".to_string())),
            timeline: Some(asm::FuncletId(timeline_funclet_name.to_string())),
            spatial: None,
        }),
    };

    // TODO don't simply return the last node like below
    let last_id_str = label::label_node(last_alloc_id);
    let dummy_tail_edge =
        Some(asm::TailEdge::Return { return_values: Some(vec![Some(last_id_str.clone())]) });

    let commands = node_context.into_commands();

    vec![ScheduleExplicitFunclet {
        inner_funclet: InnerFunclet { header: dummy_header, commands, tail_edge: dummy_tail_edge },
    }]
}
