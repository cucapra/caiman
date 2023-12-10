use crate::{
    enum_cast,
    parse::ast::{ClassMembers, DataType, NestedExpr, SpecLiteral, SpecStmt, SpecTerm, TopLevel},
};
use caiman::{assembly::ast as asm, ir};

use super::{binop_to_str, data_type_to_local_type};

/// Lower a spec term into a caiman assembly node.
fn lower_spec_term(t: SpecTerm) -> asm::Node {
    match t {
        SpecTerm::Lit { lit, .. } => match lit {
            SpecLiteral::Int(v) => asm::Node::Constant {
                // TODO: different int widths
                value: Some(v),
                type_id: Some(asm::TypeId::Local(String::from("i64"))),
            },
            SpecLiteral::Bool(v) => asm::Node::Constant {
                value: Some(if v {
                    String::from("1")
                } else {
                    String::from("0")
                }),
                type_id: Some(asm::TypeId::Local(String::from("bool"))),
            },
            SpecLiteral::Float(v) => asm::Node::Constant {
                value: Some(v),
                type_id: Some(asm::TypeId::Local(String::from("f64"))),
            },
            _ => todo!(),
        },
        SpecTerm::Call { .. } => panic!("Unexpected call"),
        // we can probably do a local copy propagation here
        SpecTerm::Var { .. } => todo!(),
    }
}

/// Lowers a spec call into caiman assembly call and extract.
fn lower_spec_call(
    lhs: String,
    function: &NestedExpr<SpecTerm>,
    args: Vec<NestedExpr<SpecTerm>>,
) -> Vec<asm::Hole<asm::Command>> {
    // TODO: multiple returns
    let function = enum_cast!(
        SpecTerm::Var { name, .. },
        name,
        enum_cast!(NestedExpr::Term, function)
    )
    .clone();
    let tuple_id = format!("_t{lhs}");
    vec![
        Some(asm::Command::Node(asm::NamedNode {
            name: Some(asm::NodeId(tuple_id.clone())),
            node: asm::Node::CallFunctionClass {
                function_id: Some(asm::FunctionClassId(function)),
                arguments: Some(
                    args.into_iter()
                        .map(|x| {
                            let t = enum_cast!(NestedExpr::Term, x);
                            let v = enum_cast!(SpecTerm::Var { name, .. }, name, t);
                            Some(asm::NodeId(v))
                        })
                        .collect(),
                ),
            },
        })),
        Some(asm::Command::Node(asm::NamedNode {
            name: Some(asm::NodeId(lhs)),
            node: asm::Node::ExtractResult {
                node_id: Some(asm::NodeId(tuple_id)),
                index: Some(0),
            },
        })),
    ]
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

/// Lowers a flattened spec assignment into an assembly command.
/// This will convert things like additions and conditionals into assembly constructrs
/// like external function calls and select nodes.
/// # Panics
/// Panics if the rhs expression is not flattened
/// (i.e. contains a nested expression, or constants outside of direct assignments)
fn lower_spec_assign(lhs: String, e: NestedExpr<SpecTerm>) -> Vec<asm::Hole<asm::Command>> {
    match e {
        NestedExpr::Conditional {
            if_true,
            if_false,
            guard,
            ..
        } => {
            let guard_id = term_to_name(*guard);
            let true_id = term_to_name(*if_true);
            let false_id = term_to_name(*if_false);
            vec![Some(asm::Command::Node(asm::NamedNode {
                name: Some(asm::NodeId(lhs)),
                node: asm::Node::Select {
                    condition: Some(asm::NodeId(guard_id)),
                    true_case: Some(asm::NodeId(true_id)),
                    false_case: Some(asm::NodeId(false_id)),
                },
            }))]
        }
        NestedExpr::Term(SpecTerm::Call { function, args, .. }) => {
            lower_spec_call(lhs, &function, args)
        }
        NestedExpr::Term(t) => vec![Some(asm::Command::Node(asm::NamedNode {
            name: Some(asm::NodeId(lhs)),
            node: lower_spec_term(t),
        }))],
        NestedExpr::Binop {
            op,
            lhs: op_lhs,
            rhs: op_rhs,
            ..
        } => {
            let op_lhs = term_to_name(*op_lhs);
            let op_rhs = term_to_name(*op_rhs);
            let temp = format!("_{lhs}_t");
            vec![
                Some(asm::Command::Node(asm::NamedNode {
                    name: Some(asm::NodeId(temp.clone())),
                    node: asm::Node::CallFunctionClass {
                        function_id: Some(asm::FunctionClassId(binop_to_str(op).to_string())),
                        arguments: Some(vec![Some(asm::NodeId(op_lhs)), Some(asm::NodeId(op_rhs))]),
                    },
                })),
                Some(asm::Command::Node(asm::NamedNode {
                    name: Some(asm::NodeId(lhs)),
                    node: asm::Node::ExtractResult {
                        node_id: Some(asm::NodeId(temp)),
                        index: Some(0),
                    },
                })),
            ]
        }
        NestedExpr::Uop { .. } => todo!(),
    }
}
/// Lower a list of spec statements into a list of assembly commands.
fn lower_spec_stmts(stmts: Vec<SpecStmt>) -> Vec<Option<asm::Command>> {
    let mut res = vec![];
    for stmt in stmts {
        match stmt {
            SpecStmt::Assign { mut lhs, rhs, .. } => {
                assert_eq!(lhs.len(), 1);
                res.extend(lower_spec_assign(lhs.swap_remove(0).0, rhs));
            }
            SpecStmt::Returns(_, e) => {
                let v = term_to_name(e);
                res.push(Some(asm::Command::TailEdge(asm::TailEdge::Return {
                    return_values: Some(vec![Some(asm::NodeId(v))]),
                })));
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
fn lower_spec_funclet(
    name: String,
    input: Vec<(String, DataType)>,
    output: Option<(Option<String>, DataType)>,
    statements: Vec<SpecStmt>,
    class_name: Option<&str>,
) -> (asm::FuncletHeader, Vec<Option<asm::Command>>) {
    (
        asm::FuncletHeader {
            name: asm::FuncletId(name),
            args: input
                .into_iter()
                .map(|x| {
                    let (name, dt) = x;
                    asm::FuncletArgument {
                        name: Some(asm::NodeId(name)),
                        typ: data_type_to_local_type(&dt),
                        tags: Vec::new(),
                    }
                })
                .collect(),
            ret: output
                .into_iter()
                .map(|x| {
                    let (name, dt) = x;
                    asm::FuncletArgument {
                        name: name.map(asm::NodeId),
                        typ: data_type_to_local_type(&dt),
                        tags: Vec::new(),
                    }
                })
                .collect(),
            binding: class_name.map_or(asm::FuncletBinding::None, |name| {
                asm::FuncletBinding::ValueBinding(asm::FunctionClassBinding {
                    default: false,
                    function_class: asm::FunctionClassId(name.to_string()),
                })
            }),
        },
        lower_spec_stmts(statements),
    )
}

/// Lower an external value function
/// # Panics
/// Panics if the function is not a value funclet
pub fn lower_val_funclet(f: ClassMembers, class_name: &str) -> asm::Funclet {
    if let ClassMembers::ValueFunclet {
        name,
        input,
        output,
        statements,
        ..
    } = f
    {
        let (header, commands) =
            lower_spec_funclet(name, input, output, statements, Some(class_name));
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
pub fn lower_timeline_funclet(f: TopLevel) -> asm::Funclet {
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
    let (header, commands) = lower_spec_funclet(name, input, Some(output), statements, None);
    asm::Funclet {
        kind: ir::FuncletKind::Timeline,
        header,
        commands,
    }
}

/// Lower a spatial funclet into a caiman assembly funclet.
/// # Panics
/// Panics if the function is not a spatial funclet
pub fn lower_spatial_funclet(f: TopLevel) -> asm::Funclet {
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
    let (header, commands) = lower_spec_funclet(name, input, Some(output), statements, None);
    asm::Funclet {
        kind: ir::FuncletKind::Spatial,
        header,
        commands,
    }
}
