use crate::{
    enum_cast,
    lower::IN_STEM,
    parse::ast::{
        Binop, ClassMembers, DataType, NestedExpr, SpecExpr, SpecLiteral, SpecStmt, SpecTerm,
        TemplateArgs, TopLevel,
    },
    typing::{Context, SpecInfo},
};
use caiman::explication::Hole;
use caiman::{
    assembly::ast::{self as asm},
    ir,
};

use super::{binop_to_str, tuple_id};

/// Lower a spec term into a caiman assembly node.
fn lower_spec_term(t: SpecTerm) -> asm::Node {
    match t {
        SpecTerm::Lit { lit, .. } => match lit {
            SpecLiteral::Int(v) => asm::Node::Constant {
                // TODO: different int widths
                value: Hole::Filled(v),
                type_id: Hole::Filled(asm::TypeId(String::from("i64"))),
            },
            SpecLiteral::Bool(v) => asm::Node::Constant {
                value: Hole::Filled(if v {
                    String::from("1")
                } else {
                    String::from("0")
                }),
                type_id: Hole::Filled(asm::TypeId(String::from("bool"))),
            },
            SpecLiteral::Float(v) => asm::Node::Constant {
                value: Hole::Filled(v),
                type_id: Hole::Filled(asm::TypeId(String::from("f64"))),
            },
            _ => todo!(),
        },
        SpecTerm::Call { .. } => panic!("Unexpected call"),
        // we can probably do a local copy propagation here
        SpecTerm::Var { .. } => todo!(),
    }
}

/// Special encode begin function name
const ENCODE_BEGIN: &str = "encode_event";
/// Special submit event function name
const SUBMIT: &str = "submit_event";
/// Special sync event function name
const SYNC: &str = "sync_event";

/// Lowers a call to the special `encode_begin` function into caiman assembly.
fn lower_spec_encode_event(
    args: Vec<NestedExpr<SpecTerm>>,
    lhs: Vec<String>,
) -> Vec<Hole<asm::Command>> {
    let tuple_id = asm::NodeId(tuple_id(&lhs));
    assert!(!args.is_empty());
    let mut res = vec![Hole::Filled(asm::Command::Node(asm::NamedNode {
        node: asm::Node::EncodingEvent {
            local_past: Hole::Filled(asm::NodeId(enum_cast!(
                SpecTerm::Var { name, .. },
                name,
                enum_cast!(NestedExpr::Term, &args[0]).clone()
            ))),
            remote_local_pasts: Hole::Filled(
                args.into_iter()
                    .skip(1)
                    .map(|x| {
                        Hole::Filled(asm::NodeId(enum_cast!(
                            SpecTerm::Var { name, .. },
                            name,
                            enum_cast!(NestedExpr::Term, x)
                        )))
                    })
                    .collect(),
            ),
        },
        name: Some(tuple_id.clone()),
    }))];
    for (i, lhs) in lhs.into_iter().enumerate() {
        res.push(Hole::Filled(asm::Command::Node(asm::NamedNode {
            name: Some(asm::NodeId(lhs)),
            node: asm::Node::ExtractResult {
                node_id: Hole::Filled(tuple_id.clone()),
                index: Hole::Filled(i),
            },
        })));
    }
    res
}

/// Lowers a spec submission event into caiman assembly.
fn lower_spec_submission_event(
    args: &[NestedExpr<SpecTerm>],
    dests: &[String],
) -> Vec<Hole<asm::Command>> {
    assert_eq!(args.len(), 1);
    assert_eq!(dests.len(), 1);
    vec![Hole::Filled(asm::Command::Node(asm::NamedNode {
        name: Some(asm::NodeId(dests[0].clone())),
        node: asm::Node::SubmissionEvent {
            local_past: Hole::Filled(asm::NodeId(enum_cast!(
                SpecTerm::Var { name, .. },
                name,
                enum_cast!(SpecExpr::Term, &args[0]).clone()
            ))),
        },
    }))]
}
/// Lowers a call to the special `sync` function into caiman assembly.
fn lower_spec_sync_event(
    args: &[NestedExpr<SpecTerm>],
    dests: &[String],
) -> Vec<Hole<asm::Command>> {
    assert_eq!(args.len(), 2);
    assert_eq!(dests.len(), 1);
    vec![Hole::Filled(asm::Command::Node(asm::NamedNode {
        name: Some(asm::NodeId(dests[0].clone())),
        node: asm::Node::SynchronizationEvent {
            local_past: Hole::Filled(asm::NodeId(enum_cast!(
                SpecTerm::Var { name, .. },
                name,
                enum_cast!(SpecExpr::Term, &args[0]).clone()
            ))),
            remote_local_past: Hole::Filled(asm::NodeId(enum_cast!(
                SpecTerm::Var { name, .. },
                name,
                enum_cast!(SpecExpr::Term, &args[1]).clone()
            ))),
        },
    }))]
}

/// Lowers a spec call into caiman assembly call and extract.
/// Special functions are handled separately
fn lower_spec_call(
    lhs: Vec<String>,
    function: &NestedExpr<SpecTerm>,
    mut args: Vec<NestedExpr<SpecTerm>>,
    templates: Option<TemplateArgs>,
) -> Vec<Hole<asm::Command>> {
    let function = enum_cast!(
        SpecTerm::Var { name, .. },
        name,
        enum_cast!(NestedExpr::Term, function)
    )
    .clone();
    if let Some(TemplateArgs::Vals(mut vs)) = templates {
        vs.append(&mut args);
        args = vs;
    }
    match function.as_str() {
        ENCODE_BEGIN => lower_spec_encode_event(args, lhs),
        SUBMIT => lower_spec_submission_event(&args, &lhs),
        SYNC => lower_spec_sync_event(&args, &lhs),
        _ => {
            let tuple_id = asm::NodeId(tuple_id(&lhs));
            let mut r = vec![Hole::Filled(asm::Command::Node(asm::NamedNode {
                name: Some(tuple_id.clone()),
                node: asm::Node::CallFunctionClass {
                    function_id: Hole::Filled(asm::FunctionClassId(function)),
                    arguments: Hole::Filled(
                        args.into_iter()
                            .map(|x| {
                                let t = enum_cast!(NestedExpr::Term, x);
                                let v = enum_cast!(SpecTerm::Var { name, .. }, name, t);
                                Hole::Filled(asm::NodeId(v))
                            })
                            .collect(),
                    ),
                },
            }))];
            for (i, name) in lhs.into_iter().enumerate() {
                r.push(Hole::Filled(asm::Command::Node(asm::NamedNode {
                    name: Some(asm::NodeId(name)),
                    node: asm::Node::ExtractResult {
                        node_id: Hole::Filled(tuple_id.clone()),
                        index: Hole::Filled(i),
                    },
                })));
            }
            r
        }
    }
}

/// Converts a term to its node name, assuming that the term is a variable
/// # Panics
/// Panics if the term is not a variable
fn term_to_name(t: NestedExpr<SpecTerm>) -> String {
    enum_cast!(
        SpecTerm::Var { name, .. },
        name,
        enum_cast!(NestedExpr::Term, t)
    )
}

/// Lowers a binary operation into a caiman assembly call and extract.
fn lower_binop(
    dest: String,
    op: Binop,
    op_lhs: NestedExpr<SpecTerm>,
    op_rhs: NestedExpr<SpecTerm>,
    type_ctx: &SpecInfo,
) -> Vec<Hole<asm::Command>> {
    let op_lhs = term_to_name(op_lhs);
    let op_rhs = term_to_name(op_rhs);
    let temp = tuple_id(&[dest.clone()]);
    vec![
        Hole::Filled(asm::Command::Node(asm::NamedNode {
            name: Some(asm::NodeId(temp.clone())),
            node: asm::Node::CallFunctionClass {
                function_id: Hole::Filled(asm::FunctionClassId(binop_to_str(
                    op,
                    &format!("{:#}", type_ctx.types.get(&op_lhs).unwrap()),
                    &format!("{:#}", type_ctx.types.get(&op_rhs).unwrap()),
                ))),
                arguments: Hole::Filled(vec![
                    Hole::Filled(asm::NodeId(op_lhs)),
                    Hole::Filled(asm::NodeId(op_rhs)),
                ]),
            },
        })),
        Hole::Filled(asm::Command::Node(asm::NamedNode {
            name: Some(asm::NodeId(dest)),
            node: asm::Node::ExtractResult {
                node_id: Hole::Filled(asm::NodeId(temp)),
                index: Hole::Filled(0),
            },
        })),
    ]
}

/// Lowers a flattened spec assignment into an assembly command.
/// This will convert things like additions and conditionals into assembly constructrs
/// like external function calls and select nodes.
/// # Panics
/// Panics if the rhs expression is not flattened
/// (i.e. contains a nested expression, or constants outside of direct assignments)
fn lower_spec_assign(
    mut lhs: Vec<String>,
    e: NestedExpr<SpecTerm>,
    global_ctx: &Context,
    spec_name: &str,
) -> Vec<Hole<asm::Command>> {
    match e {
        NestedExpr::Conditional {
            if_true,
            if_false,
            guard,
            ..
        } => {
            assert_eq!(lhs.len(), 1);
            let guard_id = term_to_name(*guard);
            let true_id = term_to_name(*if_true);
            let false_id = term_to_name(*if_false);
            vec![Hole::Filled(asm::Command::Node(asm::NamedNode {
                name: Some(asm::NodeId(lhs.swap_remove(0))),
                node: asm::Node::Select {
                    condition: Hole::Filled(asm::NodeId(guard_id)),
                    true_case: Hole::Filled(asm::NodeId(true_id)),
                    false_case: Hole::Filled(asm::NodeId(false_id)),
                },
            }))]
        }
        NestedExpr::Term(SpecTerm::Call {
            function,
            args,
            templates,
            ..
        }) => lower_spec_call(lhs, &function, args, templates),
        NestedExpr::Term(t) => {
            assert_eq!(lhs.len(), 1);
            let node = lower_spec_term(t);
            vec![Hole::Filled(asm::Command::Node(asm::NamedNode {
                name: Some(asm::NodeId(lhs.swap_remove(0))),
                node,
            }))]
        }
        NestedExpr::Binop {
            op,
            lhs: op_lhs,
            rhs: op_rhs,
            ..
        } => {
            assert_eq!(lhs.len(), 1);
            lower_binop(
                lhs.swap_remove(0),
                op,
                *op_lhs,
                *op_rhs,
                &global_ctx.specs[spec_name],
            )
        }
        NestedExpr::Uop { .. } => todo!(),
    }
}
/// Lower a list of spec statements into a list of assembly commands by appending
/// the commands to the given vector `res`.
fn lower_spec_stmts(
    stmts: Vec<SpecStmt>,
    ctx: &Context,
    spec_name: &str,
    mut res: Vec<Hole<asm::Command>>,
) -> Vec<Hole<asm::Command>> {
    for stmt in stmts {
        match stmt {
            SpecStmt::Assign { lhs, rhs, .. } => {
                res.extend(lower_spec_assign(
                    lhs.into_iter().map(|x| x.0).collect(),
                    rhs,
                    ctx,
                    spec_name,
                ));
            }
            SpecStmt::Returns(_, e) => {
                if let SpecExpr::Term(SpecTerm::Lit {
                    lit: SpecLiteral::Tuple(names),
                    ..
                }) = e
                {
                    res.push(Hole::Filled(asm::Command::TailEdge(
                        asm::TailEdge::Return {
                            return_values: Hole::Filled(
                                names
                                    .into_iter()
                                    .map(|x| Hole::Filled(asm::NodeId(term_to_name(x))))
                                    .collect(),
                            ),
                        },
                    )));
                } else {
                    let v = term_to_name(e);
                    res.push(Hole::Filled(asm::Command::TailEdge(
                        asm::TailEdge::Return {
                            return_values: Hole::Filled(vec![Hole::Filled(asm::NodeId(v))]),
                        },
                    )));
                }
            }
        }
    }
    res
}

/// Lower a spec funclet into a caiman assembly funclet.
/// # Arguments
/// * `name` - The name of the funclet
/// * `input` - The input arguments of the funclet
/// * `output` - The output arguments of the funclet. Can be `None` for a void
///     returning funclet
/// * `statements` - The statements of the funclet
/// * `class_name` - The name of the class the funclet belongs to. Can be `None`
///    for a non-value spec funclet
/// * `ctx` - The global context
fn lower_spec_funclet(
    name: &str,
    input: Vec<(String, DataType)>,
    output: Vec<(Option<String>, DataType)>,
    statements: Vec<SpecStmt>,
    class_name: Option<&str>,
    ctx: &Context,
) -> (asm::FuncletHeader, Vec<Hole<asm::Command>>) {
    let phi_nodes: Vec<_> = input
        .iter()
        .enumerate()
        .map(|(idx, (name, _))| {
            Hole::Filled(asm::Command::Node(asm::NamedNode {
                name: Some(asm::NodeId(name.to_string())),
                node: asm::Node::Phi {
                    index: Hole::Filled(idx),
                },
            }))
        })
        .collect();
    (
        asm::FuncletHeader {
            name: asm::FuncletId(name.to_string()),
            args: input
                .into_iter()
                .map(|x| {
                    let (name, dt) = x;
                    asm::FuncletArgument {
                        name: Some(asm::NodeId(format!("{IN_STEM}{name}"))),
                        typ: dt.asm_type(),
                        tags: Vec::new(),
                    }
                })
                .collect(),
            ret: output
                .into_iter()
                .enumerate()
                .map(|(id, (name, dt))| asm::FuncletArgument {
                    name: Some(asm::NodeId(name.unwrap_or_else(|| format!("_out{id}")))),
                    typ: dt.asm_type(),
                    tags: Vec::new(),
                })
                .collect(),
            binding: class_name.map_or(asm::FuncletBinding::None, |name| {
                asm::FuncletBinding::SpecBinding(asm::FunctionClassBinding {
                    default: false,
                    function_class: asm::FunctionClassId(name.to_string()),
                })
            }),
        },
        lower_spec_stmts(statements, ctx, name, phi_nodes),
    )
}

/// Lower an external value function
/// # Panics
/// Panics if the function is not a value funclet
pub fn lower_val_funclet(f: ClassMembers, class_name: &str, ctx: &Context) -> asm::Funclet {
    if let ClassMembers::ValueFunclet {
        name,
        input,
        output,
        statements,
        ..
    } = f
    {
        let (header, commands) =
            lower_spec_funclet(&name, input, output, statements, Some(class_name), ctx);
        asm::Funclet {
            kind: ir::FuncletKind::Value,
            header,
            commands,
        }
    } else {
        panic!("Expected value functlet")
    }
}

/// Lower a timeline funclet into a caiman assembly funclet.
/// # Panics
/// Panics if the function is not a timeline funclet
pub fn lower_timeline_funclet(f: TopLevel, ctx: &Context) -> asm::Funclet {
    let (name, input, output, statements) = enum_cast!(
        TopLevel::TimelineFunclet {
            name,
            input,
            output,
            statements,
            ..
        },
        (name, input, output, statements),
        f
    );
    let (header, commands) = lower_spec_funclet(&name, input, vec![output], statements, None, ctx);
    asm::Funclet {
        kind: ir::FuncletKind::Timeline,
        header,
        commands,
    }
}

/// Lower a spatial funclet into a caiman assembly funclet.
/// # Panics
/// Panics if the function is not a spatial funclet
pub fn lower_spatial_funclet(f: TopLevel, ctx: &Context) -> asm::Funclet {
    let (name, input, output, statements) = enum_cast!(
        TopLevel::SpatialFunclet {
            name,
            input,
            output,
            statements,
            ..
        },
        (name, input, output, statements),
        f
    );
    let (header, commands) = lower_spec_funclet(&name, input, vec![output], statements, None, ctx);
    asm::Funclet {
        kind: ir::FuncletKind::Spatial,
        header,
        commands,
    }
}
