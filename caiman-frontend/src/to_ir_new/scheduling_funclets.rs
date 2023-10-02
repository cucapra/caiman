use std::collections::HashMap;

use super::funclet_util::make_asm_funclet;
use super::funclet_util::vf_node_with_name;
use super::function_classes::FunctionClassContext;
use super::typing;
use super::typing::TypingContext;
use super::value_funclets::ValueFunclet;
use super::scheduling_funclets_ir;
use crate::syntax::ast;
use crate::to_ir_new::label;
use caiman::assembly::ast as asm;
use caiman::ir;

pub struct ASMSchedulingFunclet(pub asm::Funclet);

pub fn lower_scheduling_funclets(
    function_class_ctx: &FunctionClassContext,
    typing_ctx: &mut TypingContext,
    value_funclets: &Vec<ValueFunclet>,
    program: &ast::Program,
) -> Vec<ASMSchedulingFunclet>
{
    // TODO use schewduling funclet IR stuff here (in loop)
    
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
                    let mut lowered_sfs = lower_scheduling_funclet(
                        function_class_ctx,
                        typing_ctx,
                        value_funclet,
                        sf,
                    );
                    asm_scheduling_funclets.append(&mut lowered_sfs);
                }
            },

            _ => (),
        }
    }
    asm_scheduling_funclets
}

struct SchedulingContext
{
    map: HashMap<String, Option<String>>,
}

impl SchedulingContext
{
    fn add(&mut self, x: String, e: &ast::scheduling::Expr)
    {
        let (_info, kind) = e;
        use ast::scheduling as sch;
        match kind {
            sch::Hole::Filled(sch::ExprKind::Simple { value_var, .. }) => {
                self.map.insert(x, Some(value_var.clone()));
            },
            sch::Hole::Vacant => {
                self.map.insert(x, None);
            },
        }
    }
}

struct Operation
{
    value_funclet_name: Option<asm::FuncletId>,
    base_node: Option<asm::NodeId>,
    kind: ast::scheduling::FullSchedulable,
}

enum SchedulingNodeCombo
{
    LocalValue
    {
        operation: Option<Operation>, storage_type: Option<asm::TypeId>
    },
}

impl SchedulingNodeCombo
{
    fn to_named_nodes(self, name: &str) -> Vec<Option<asm::NamedNode>>
    {
        match self {
            SchedulingNodeCombo::LocalValue { operation, storage_type } => {
                let place = Some(ir::Place::Local);

                let make_rmi = |funclet, node| Some(asm::RemoteNodeId { funclet, node });

                let alloc_temp_name = asm::NodeId(name.to_string());

                //let operation = std::mem::take(&mut alloc_do.alloc_temp_operation);
                let alloc_temp_node =
                    asm::Node::AllocTemporary { place, /* operation, */ storage_type };

                let encode_do_name = asm::NodeId(name.to_string() + "_ENCODEDO");

                let encode_do_node = if let Some(op) = operation {
                    match op.kind {
                        ast::scheduling::FullSchedulable::Primitive => {
                            let op = make_rmi(op.value_funclet_name, op.base_node);
                            asm::Node::LocalDoBuiltin {
                                operation: Some(asm::Quotient::Node(op)),
                                inputs: Some(vec![]),
                                outputs: Some(vec![Some(alloc_temp_name.clone())]),
                            }
                        },
                        ast::scheduling::FullSchedulable::CallExternal(f, locs) => {
                            let call_node =
                                op.base_node.as_ref().map(|n| label::label_call_node(n));
                            let op = make_rmi(op.value_funclet_name, call_node);
                            let inputs = locs
                                .into_iter()
                                .map(|a| a.to_option_move().map(|s| asm::NodeId(s)))
                                .collect();
                            let f_id = f.to_option().map(|s| s.to_string());
                            asm::Node::LocalDoExternal {
                                operation: Some(asm::Quotient::Node(op)),
                                external_function_id: f_id.map(|s| asm::ExternalFunctionId(s)),
                                inputs: Some(inputs),
                                outputs: Some(vec![Some(alloc_temp_name.clone())]),
                            }
                        },
                        ast::scheduling::FullSchedulable::Call(f, locs) => {
                            // Probs also local but with more args and other stuff different
                            // But i guess if it's local then the function name isn't needed?
                            // don't rly have example to work off of :'(
                            todo!()
                        },
                    }
                } else {
                    asm::Node::LocalDoBuiltin { operation: None, inputs: None, outputs: None }
                };

                let alloc_temp_nn =
                    Some(asm::NamedNode { name: Some(alloc_temp_name), node: alloc_temp_node });
                let encode_do_nn =
                    Some(asm::NamedNode { name: Some(encode_do_name), node: encode_do_node });
                vec![alloc_temp_nn, encode_do_nn]
            },
        }
    }
}

fn schedule_expr_to_node_combo(
    expr: &ast::scheduling::Expr,
    funclet_being_scheduled: &ValueFunclet,
    function_class_ctx: &FunctionClassContext,
) -> Option<SchedulingNodeCombo>
{
    let (info, kind) = expr;
    let (value_var, full) = match kind {
        ast::scheduling::Hole::Filled(ast::scheduling::ExprKind::Simple { value_var, full }) => {
            (value_var, full)
        },
        _ => return None,
    };

    // TODO obviously hole-able stuff should be possible here (that would be a case where search
    // for var fails)
    let vf_node = vf_node_with_name(funclet_being_scheduled, &value_var)
        .unwrap_or_else(|| panic!("Scheduling an unknown variable {} at {:?}", value_var, info));
    let operation = Some(Operation {
        value_funclet_name: Some(funclet_being_scheduled.0.header.name.clone()),
        base_node: vf_node.name.clone(),
        kind: full.clone(),
    });
    let storage_type =
        typing::type_of_asm_node(&vf_node.node, funclet_being_scheduled, function_class_ctx);
    Some(SchedulingNodeCombo::LocalValue { operation, storage_type })
}

fn schedule_stmt_kind(
    nodes: &mut Vec<Option<asm::NamedNode>>,
    scheduling_ctx: &mut SchedulingContext,
    funclet_being_scheduled: &ValueFunclet,
    function_class_ctx: &FunctionClassContext,
    stmt: &ast::scheduling::Stmt,
)
{
    let (info, stmt_kind) = stmt;
    let filled_stmt_kind = if let ast::scheduling::Hole::Filled(filled_stmt_kind) = stmt_kind {
        filled_stmt_kind
    } else {
        nodes.push(None);
        return;
    };

    match filled_stmt_kind {
        ast::scheduling::StmtKind::Let(x, e) => {
            if let Some(combo) =
                schedule_expr_to_node_combo(e, funclet_being_scheduled, function_class_ctx)
            {
                scheduling_ctx.add(x.clone(), &e);

                let mut combo_vec = combo.to_named_nodes(x);
                nodes.append(&mut combo_vec);
            } else {
            }
        },
        ast::scheduling::StmtKind::Return(x) => {
            let value_x_opt = scheduling_ctx
                .map
                .get(x)
                .unwrap_or_else(|| panic!("Returning an unknown variable {} at {:?}", x, info));

            let storage_type = value_x_opt.as_ref().and_then(|value_x| {
                let x_node = vf_node_with_name(funclet_being_scheduled, value_x)
                    .unwrap_or_else(|| panic!("Unknown scheduled variable {}", value_x));
                typing::type_of_asm_node(&x_node.node, funclet_being_scheduled, function_class_ctx)
            });
            let read_ref =
                asm::Node::ReadRef { storage_type, source: Some(asm::NodeId(x.clone())) };
            let read_ref_named =
                Some(asm::NamedNode { name: returned_variable_name(), node: read_ref });
            nodes.push(read_ref_named)
        },
    }
}

fn returned_variable_name() -> Option<asm::NodeId> { Some(asm::NodeId("result".to_string())) }

fn lower_scheduling_funclet(
    function_class_ctx: &FunctionClassContext,
    typing_ctx: &mut TypingContext,
    funclet_being_scheduled: &ValueFunclet,
    scheduling_funclet: &ast::scheduling::SchedulingFunclet,
) -> Vec<ASMSchedulingFunclet>
{
    // TODO  this is used for testing 
    /*let _ = scheduling_funclets_ir::ast_to_total_funclet(
        function_class_ctx,
        //typing_ctx,
        funclet_being_scheduled,
        scheduling_funclet,
    );*/
    // end

    let mut scheduling_ctx = SchedulingContext { map: HashMap::new() };

    let mut nodes: Vec<Option<asm::NamedNode>> = Vec::new();

    for stmt in scheduling_funclet.statements.iter() {
        schedule_stmt_kind(
            &mut nodes,
            &mut scheduling_ctx,
            funclet_being_scheduled,
            function_class_ctx,
            stmt,
        )
    }

    let tail_edge = asm::TailEdge::Return { return_values: Some(vec![returned_variable_name()]) };

    let mut convert_type = |typ: &ast::scheduling::Type| match typing_ctx
        .convert_and_add_scheduling_type(typ.clone(), funclet_being_scheduled, function_class_ctx)
    {
        Err(why) => panic!("Error at {:?}: {}", scheduling_funclet.info, why),
        Ok(s) => asm::TypeId::Local(s),
    };

    let timeline_rmi = |node_name: &str| match scheduling_funclet.timeline_funclet.clone() {
        None => panic!("No timeline funclet to reference for scheduling funclet"),
        Some(tf) => asm::RemoteNodeId {
            funclet: Some(asm::FuncletId(tf.to_string())),
            node: Some(asm::NodeId(node_name.to_string())),
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
            // XXX this part is also a hack!!!! obvs
            implicit_tags: Some((
                asm::Tag { quot: asm::Quotient::None, flow: ir::Flow::Have },
                asm::Tag { quot: asm::Quotient::None, flow: ir::Flow::Have },
            )),
        }),
    };

    vec![ASMSchedulingFunclet(make_asm_funclet(
        ir::FuncletKind::ScheduleExplicit,
        header,
        nodes,
        tail_edge,
    ))]
}
