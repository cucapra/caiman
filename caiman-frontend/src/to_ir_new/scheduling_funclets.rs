use super::funclet_util::make_asm_funclet;
use super::funclet_util::vf_node_with_name;
//use super::function_classes::FunctionClassContext;
use super::typing::TypingContext;
use super::value_funclets::ValueFunclet;
use super::typing;
use crate::syntax::ast;
use caiman::assembly::ast as asm;
use caiman::ir;

pub struct ASMSchedulingFunclet(pub asm::Funclet);

pub fn lower_scheduling_funclets(
    //function_class_ctx: &FunctionClassContext, XXX is this needed?
    typing_ctx: &mut TypingContext,
    value_funclets: &Vec<ValueFunclet>,
    program: &ast::Program,
) -> Vec<ASMSchedulingFunclet>
{
    let mut asm_scheduling_funclets: Vec<ASMSchedulingFunclet> = Vec::new();
    for (info, decl_kind) in program.iter() {
        match decl_kind {
            ast::DeclKind::SchedulingImpl { value_funclet_name, scheduling_funclets } => {
                let value_funclet = value_funclets
                    .iter()
                    .find(|vf| &vf.0.header.name.0 == value_funclet_name)
                    .unwrap_or_else(|| {
                        panic!(
                            "Cannot schedule for undeclared value funclet {} at {:?}",
                            value_funclet_name, info
                        )
                    });

                for sf in scheduling_funclets.iter() {
                    let sf_lowered = lower_scheduling_funclet(typing_ctx, value_funclet, sf);
                    asm_scheduling_funclets.push(sf_lowered);
                }
            },

            _ => (),
        }
    }
    asm_scheduling_funclets
}

enum SchedulingNodeCombo
{
    LocalValue
    {
        place: Option<ir::Place>,
        operation: Option<asm::RemoteNodeId>,
        storage_type: Option<asm::TypeId>,
    },
}

impl SchedulingNodeCombo
{
    fn to_named_nodes(self, name: &str) -> Vec<Option<asm::NamedNode>>
    {
        match self {
            SchedulingNodeCombo::LocalValue { place, operation, storage_type } => {
                let alloc_temp_name = asm::NodeId(name.to_string());
                let encode_do_name = asm::NodeId(name.to_string() + "_ENCODEDO");
                let encode_do_node = asm::Node::EncodeDo {
                    place: place.clone(),
                    operation: operation.clone(),
                    inputs: Some(Vec::new()), // TODO: What goes here ??????
                    outputs: Some(vec![Some(alloc_temp_name.clone())]),
                };
                let alloc_temp_node = asm::Node::AllocTemporary { place, operation, storage_type };
                let alloc_temp_nn =
                    Some(asm::NamedNode { name: alloc_temp_name, node: alloc_temp_node });
                let encode_do_nn =
                    Some(asm::NamedNode { name: encode_do_name, node: encode_do_node });
                vec![alloc_temp_nn, encode_do_nn]
            },
        }
    }
}

fn schedule_expr_to_node_combo(
    expr: &ast::scheduling::ScheduledExpr,
    funclet_being_scheduled: &ValueFunclet,
) -> SchedulingNodeCombo
{
    // TODO obviously hole-able stuff should be possible here (that would be a case where search
    // for var fails)
    let vf_node =
        vf_node_with_name(funclet_being_scheduled, &expr.value_var).unwrap_or_else(|| {
            panic!("Scheduling an unknown variable {} at {:?}", expr.value_var, expr.info)
        });
    let place = Some(ir::Place::Local);
    let operation = Some(asm::RemoteNodeId {
        funclet_name: Some(funclet_being_scheduled.0.header.name.clone()),
        node_name: Some(vf_node.name.clone()),
    });
    let storage_type = Some(typing::type_of_asm_node(&vf_node.node));
    SchedulingNodeCombo::LocalValue { place, operation, storage_type }
}

fn lower_scheduling_funclet(
    typing_ctx: &mut TypingContext,
    funclet_being_scheduled: &ValueFunclet,
    scheduling_funclet: &ast::scheduling::SchedulingFunclet,
) -> ASMSchedulingFunclet
{
    let mut returned_variable = None;
    let mut nodes: Vec<Option<asm::NamedNode>> = Vec::new();
    for (_stmt_info, stmt_kind) in scheduling_funclet.statements.iter() {
        match stmt_kind {
            ast::scheduling::StmtKind::Let(x, e) => {
                let combo = schedule_expr_to_node_combo(e, funclet_being_scheduled);
                let mut combo_vec = combo.to_named_nodes(x);
                nodes.append(&mut combo_vec);
            },
            ast::scheduling::StmtKind::Return(x) => {
                returned_variable = Some(asm::NodeId(x.clone()))
            },
        }
    }

    let tail_edge = asm::TailEdge::Return { return_values: Some(vec![returned_variable]) };

   
    let mut convert_type = |typ: &ast::scheduling::Type| match typing_ctx
        .convert_and_add_scheduling_type(typ.clone(), funclet_being_scheduled)
    {
        Err(why) => panic!("Error at {:?}: {}", scheduling_funclet.info, why),
        Ok(s) => asm::TypeId::Local(s),
    };

    let timeline_rmi = |node_name: &str| match scheduling_funclet.timeline_funclet.clone() {
        None => panic!("No timeline funclet to reference for scheduling funclet"),
        Some(tf) => asm::RemoteNodeId {
            funclet_name: Some(asm::FuncletId(tf.to_string())),
            node_name: Some(asm::NodeId(node_name.to_string())),
        },
    };

    let header = asm::FuncletHeader {
        name: asm::FuncletId(scheduling_funclet.name.to_string()),
        args: scheduling_funclet
            .input
            .iter()
            .map(|(name, typ)| asm::FuncletArgument {
                name: Some(asm::NodeId(name.to_string())),
                typ: convert_type(typ),
                tags: Vec::new(),
            })
            .collect(),
        ret: vec![asm::FuncletArgument {
            // TODO hacky default value here
            name: Some(asm::NodeId("out".to_string())),
            typ: convert_type(&scheduling_funclet.output),
            tags: Vec::new(),
        }],
        binding: asm::FuncletBinding::ScheduleBinding(asm::ScheduleBinding {
            value: Some(funclet_being_scheduled.0.header.name.clone()),
            timeline: scheduling_funclet.timeline_funclet.clone().map(|s| asm::FuncletId(s)),
            spatial: scheduling_funclet.spatial_funclet.clone().map(|s| asm::FuncletId(s)),
            // XXX this part is also a hack!!!! timeline arg isn't always gonna be called
            // "e"
            implicit_tags: Some((
                asm::Tag::Input(timeline_rmi("e")),
                asm::Tag::Output(timeline_rmi("e")),
            )),
        }),
    };

    ASMSchedulingFunclet(make_asm_funclet(
        ir::FuncletKind::ScheduleExplicit,
        header,
        nodes,
        tail_edge,
    ))
}
