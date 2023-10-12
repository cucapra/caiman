use std::collections::HashMap;

use super::funclet_util::{self, make_asm_funclet, vf_node_with_name};
use super::function_classes::FunctionClassContext;
use super::scheduling_funclets_ir;
use super::scheduling_funclets_ir::SplitStmt;
use super::scheduling_funclets_ir::UnsplitStmt;
use super::typing;
use super::typing::TypingContext;
use super::value_funclets::ValueFunclet;
use crate::error::Info;
use crate::syntax::ast;
use crate::syntax::ast::scheduling::Hole;
use crate::to_ir_new::label;
use caiman::assembly::ast as asm;
use caiman::ir;

pub struct ASMSchedulingFunclet(pub asm::Funclet);

struct SchedulingContext
{
    map: HashMap<String, Option<String>>,
}

pub fn lower_scheduling_funclets(
    function_class_ctx: &FunctionClassContext,
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

fn lower_scheduling_funclet(
    function_class_ctx: &FunctionClassContext,
    typing_ctx: &mut TypingContext,
    funclet_being_scheduled: &ValueFunclet,
    scheduling_funclet: &ast::scheduling::SchedulingFunclet,
) -> Vec<ASMSchedulingFunclet>
{
    let total_funclet = scheduling_funclets_ir::ast_to_total_funclet(
        function_class_ctx,
        funclet_being_scheduled,
        scheduling_funclet,
    );

    let bodies: Vec<(Vec<Option<asm::NamedNode>>, asm::TailEdge)> = total_funclet
        .split_funclets
        .iter()
        .map(|funclet| {
            let scheduling_context = build_scheduling_context(funclet);
            let mut nodes: Vec<Option<asm::NamedNode>> = Vec::new();
            for stmt in funclet.stmts.iter() {
                add_stmt(&mut nodes, stmt, &total_funclet.name);
            }
            let tail_edge = handle_tail_edge(
                &funclet.tail_edge,
                &mut nodes,
                scheduling_context,
                funclet_being_scheduled,
                function_class_ctx,
            );
            (nodes, tail_edge)
        })
        .collect();

    let headers = make_headers(
        scheduling_funclet.info,
        function_class_ctx,
        typing_ctx,
        funclet_being_scheduled,
        &total_funclet,
    );

    bodies
        .into_iter()
        .zip(headers.into_iter())
        .map(|((nodes, tail_edge), header)| {
            ASMSchedulingFunclet(make_asm_funclet(
                ir::FuncletKind::ScheduleExplicit,
                header,
                nodes,
                tail_edge,
            ))
        })
        .collect()
}

fn make_funclet_name(total_name: &str, index: usize) -> asm::FuncletId
{
    if index == 0 {
        asm::FuncletId(total_name.to_string())
    } else {
        asm::FuncletId(format!("{}{}", total_name, index))
    }
}

fn name_node(name: &str, node: asm::Node) -> asm::NamedNode
{
    asm::NamedNode { name: Some(asm::NodeId(name.to_string())), node }
}

fn add_stmt(
    nodes: &mut Vec<Option<asm::NamedNode>>,
    stmt: &Hole<scheduling_funclets_ir::SplitStmt>,
    total_name: &str,
)
{
    let filled_stmt = if let Hole::Filled(filled_stmt) = stmt {
        filled_stmt
    } else {
        nodes.push(None);
        return;
    };

    match filled_stmt {
        SplitStmt::Unsplit(ustmt) => match ustmt {
            scheduling_funclets_ir::UnsplitStmt::Let { x, e: Hole::Filled(e_filled) } => {
                let mut v = translate_ir_expr(e_filled.clone(), x);
                nodes.append(&mut v);
            },
            _ => panic!("Unsure what to do when expression is a hole"),
        },
        SplitStmt::InlineDefaultJoin(index) => {
            let default_name = "defaultjoin~";
            let inline_name = "inlinejoin~";
            let next_funclet = make_funclet_name(total_name, *index);

            let default_join = asm::Node::DefaultJoin;
            let default_join_nn = name_node(default_name, default_join);

            let inline_join = asm::Node::InlineJoin {
                funclet: Some(next_funclet),
                // TODO args!!!
                captures: Some(vec![]),
                continuation: default_join_nn.name.clone(),
            };
            let inline_join_nn = name_node(inline_name, inline_join);

            nodes.push(Some(default_join_nn));
            nodes.push(Some(inline_join_nn));
        },
    }
}

fn translate_ir_expr(e: scheduling_funclets_ir::Expr, name: &str) -> Vec<Option<asm::NamedNode>>
{
    let storage_type = e.storage_type;

    let place = Some(ir::Place::Local);

    let make_rmi = |funclet, node| Some(asm::RemoteNodeId { funclet, node });

    let alloc_temp_name = asm::NodeId(name.to_string());

    //let operation = std::mem::take(&mut alloc_do.alloc_temp_operation);
    let alloc_temp_node = asm::Node::AllocTemporary { place, /* operation, */ storage_type };

    let encode_do_name = asm::NodeId(name.to_string() + "_ENCODEDO");

    let encode_do_node = if let Some(op) = e.operation {
        match op.kind {
            ast::scheduling::FullSchedulable::Primitive => {
                let op = make_rmi(op.value_funclet_name, op.value_funclet_node);
                asm::Node::LocalDoBuiltin {
                    operation: Some(asm::Quotient::Node(op)),
                    inputs: Some(vec![]),
                    outputs: Some(vec![Some(alloc_temp_name.clone())]),
                }
            },
            ast::scheduling::FullSchedulable::CallExternal(f, locs) => {
                let call_node = op.value_funclet_node.as_ref().map(|n| label::label_call_node(n));
                let op = make_rmi(op.value_funclet_name, call_node);
                let inputs =
                    locs.into_iter().map(|a| a.to_option_move().map(|s| asm::NodeId(s))).collect();
                let f_id = f.to_option().map(|s| s.to_string());
                asm::Node::LocalDoExternal {
                    operation: Some(asm::Quotient::Node(op)),
                    external_function_id: f_id.map(|s| asm::ExternalFunctionId(s)),
                    inputs: Some(inputs),
                    outputs: Some(vec![Some(alloc_temp_name.clone())]),
                }
            },
            ast::scheduling::FullSchedulable::Call(_, _) => {
                panic!(
                    "Internal call should not be an expr at the 'schedule IR -> ASM' stage, it \
                     should have been turned into a tail edge already."
                )
            },
        }
    } else {
        asm::Node::LocalDoBuiltin { operation: None, inputs: None, outputs: None }
    };

    let alloc_temp_nn =
        Some(asm::NamedNode { name: Some(alloc_temp_name), node: alloc_temp_node });
    let encode_do_nn = Some(asm::NamedNode { name: Some(encode_do_name), node: encode_do_node });
    vec![alloc_temp_nn, encode_do_nn]
}

fn build_scheduling_context(funclet: &scheduling_funclets_ir::SplitFunclet) -> SchedulingContext
{
    let mut scheduling_ctx = SchedulingContext { map: HashMap::new() };
    for stmt in funclet.stmts.iter() {
        match stmt {
            Hole::Filled(SplitStmt::Unsplit(UnsplitStmt::Let {
                x,
                e: Hole::Filled(e_filled),
            })) => {
                let n =
                    e_filled.operation.clone().and_then(|op| op.value_funclet_node.map(|n| n.0));
                scheduling_ctx.map.insert(x.to_string(), n);
            },
            _ => (),
        }
    }
    scheduling_ctx
}

fn handle_tail_edge(
    tail_edge: &scheduling_funclets_ir::TailEdge,
    nodes: &mut Vec<Option<asm::NamedNode>>,
    scheduling_ctx: SchedulingContext,
    funclet_being_scheduled: &ValueFunclet,
    function_class_ctx: &FunctionClassContext,
) -> asm::TailEdge
{
    match tail_edge {
        scheduling_funclets_ir::TailEdge::Return(x) => {
            let value_x_opt = if let Some(value_x_opt) = scheduling_ctx.map.get(x) {
                value_x_opt
            } else {
                // Is an argument and not local variable, so no need to read ref
                let x_ret = Some(asm::NodeId(x.to_string()));
                return asm::TailEdge::Return { return_values: Some(vec![x_ret]) };
            };

            let returned_variable_name = Some(asm::NodeId("result".to_string()));

            let storage_type = value_x_opt.as_ref().and_then(|value_x| {
                let x_node = vf_node_with_name(funclet_being_scheduled, value_x)
                    .unwrap_or_else(|| panic!("Unknown scheduled variable {}", value_x));
                typing::type_of_asm_node(&x_node.node, funclet_being_scheduled, function_class_ctx)
            });
            let read_ref =
                asm::Node::ReadRef { storage_type, source: Some(asm::NodeId(x.clone())) };
            let read_ref_named =
                Some(asm::NamedNode { name: returned_variable_name.clone(), node: read_ref });
            nodes.push(read_ref_named);

            asm::TailEdge::Return { return_values: Some(vec![returned_variable_name]) }
        },

        scheduling_funclets_ir::TailEdge::ScheduleCall { callee_funclet_id } => {
            let callee_class = function_class_ctx.get(callee_funclet_id);
            let callee_node = callee_class.and_then(|f| {
                funclet_util::find_function_call(funclet_being_scheduled, &f.0)
                    .and_then(|nn| nn.clone().name)
            });

            asm::TailEdge::ScheduleCall {
                value_operation: Some(asm::Quotient::Node(Some(asm::RemoteNodeId {
                    funclet: Some(funclet_being_scheduled.0.header.name.clone()),
                    node: callee_node,
                }))),

                // Unsure what to put here
                timeline_operation: Some(asm::Quotient::None),
                spatial_operation: Some(asm::Quotient::None),

                callee_funclet_id: Some(asm::FuncletId(callee_funclet_id.to_string())),

                // TODO not this
                callee_arguments: Some(vec![]),

                // XXX unsure if this works in all cases
                continuation_join: nodes[nodes.len() - 1].clone().and_then(|nn| nn.name),
            }
        },
    }
}

fn make_headers(
    info: Info,
    function_class_ctx: &FunctionClassContext,
    typing_ctx: &mut TypingContext,
    funclet_being_scheduled: &ValueFunclet,
    total_funclet: &scheduling_funclets_ir::TotalFunclet,
) -> Vec<asm::FuncletHeader>
{
    let mut convert_type = |typ: &ast::scheduling::Type| match typing_ctx
        .convert_and_add_scheduling_type(typ.clone(), funclet_being_scheduled, function_class_ctx)
    {
        Err(why) => panic!("Error at {:?}: {}", info, why),
        Ok(s) => asm::TypeId::Local(s),
    };

    let mut headers = Vec::new();

    for (i, funclet) in total_funclet.split_funclets.iter().enumerate() {
        let header = asm::FuncletHeader {
            name: make_funclet_name(&total_funclet.name, i),
            args: funclet
                .inputs
                .iter()
                .map(|(name, typ)| asm::FuncletArgument {
                    name: Some(asm::NodeId(name.to_string())),
                    typ: convert_type(typ),
                    tags: Vec::new(),
                })
                .collect(),
            ret: vec![asm::FuncletArgument {
                name: Some(asm::NodeId(funclet.output.0.to_string())),
                typ: convert_type(&funclet.output.1),
                tags: Vec::new(),
            }],
            binding: asm::FuncletBinding::ScheduleBinding(asm::ScheduleBinding {
                value: Some(funclet_being_scheduled.0.header.name.clone()),
                // XXX Lots of hacks here! Should be differing timeline/spatial funclets
                timeline: total_funclet.timeline_funclet.clone().map(|s| asm::FuncletId(s)),
                spatial: total_funclet.spatial_funclet.clone().map(|s| asm::FuncletId(s)),
                implicit_tags: Some((
                    asm::Tag { quot: asm::Quotient::None, flow: ir::Flow::Have },
                    asm::Tag { quot: asm::Quotient::None, flow: ir::Flow::Have },
                )),
            }),
        };
        headers.push(header);
    }

    headers
}

/*
-------------------------------------------------------------------------------
********************************************************************************
* OLD CODE BELOW!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
********************************************************************************
-------------------------------------------------------------------------------
*/
/*
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
    //if scheduling_funclet.name == "bar" {
    let _ = scheduling_funclets_ir::ast_to_total_funclet(
        function_class_ctx,
        //typing_ctx,
        funclet_being_scheduled,
        scheduling_funclet,
    );
    //}
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
*/
