use crate::{
    enum_cast,
    parse::ast::{
        ClassMembers, DataType, NestedExpr, SpecExpr, SpecLiteral, SpecStmt, SpecTerm, TopLevel,
    },
};
use caiman::{assembly::ast as asm, ir};

use super::{data_type_to_type, BOOL_FFI_TYPE};

/// Lower a spec term into a caiman assembly node.
fn lower_spec_term(t: SpecTerm) -> asm::Node {
    match t {
        SpecTerm::Lit { lit, .. } => match lit {
            SpecLiteral::Int(v) => asm::Node::Constant {
                // TODO: different int widths
                value: Some(v),
                type_id: Some(asm::TypeId::FFI(asm::FFIType::I64)),
            },
            SpecLiteral::Bool(v) => asm::Node::Constant {
                value: Some(if v {
                    String::from("1")
                } else {
                    String::from("0")
                }),
                type_id: Some(asm::TypeId::FFI(BOOL_FFI_TYPE)),
            },
            SpecLiteral::Float(v) => asm::Node::Constant {
                value: Some(v),
                type_id: Some(asm::TypeId::FFI(asm::FFIType::F64)),
            },
            _ => unimplemented!(),
        },
        SpecTerm::Call { function, args, .. } => match function.as_ref() {
            SpecExpr::Term(SpecTerm::Var { name, .. }) => asm::Node::CallFunctionClass {
                function_id: Some(asm::FunctionClassId(name.to_string())),
                arguments: Some(
                    args.into_iter()
                        .map(|x| {
                            Some(asm::NodeId(enum_cast!(
                                SpecTerm::Var { name, .. },
                                name,
                                enum_cast!(NestedExpr::Term, x)
                            )))
                        })
                        .collect(),
                ),
            },
            _ => panic!("Not flattened"),
        },
        // we can probably do a local copy propagation here
        SpecTerm::Var { .. } => todo!(),
    }
}

/// Lower a list of spec statements into a list of assembly commands.
fn lower_spec_stmts(stmts: Vec<SpecStmt>) -> Vec<Option<asm::Command>> {
    let mut res = vec![];
    for stmt in stmts {
        match stmt {
            SpecStmt::Assign { mut lhs, rhs, .. } => {
                assert_eq!(lhs.len(), 1);
                res.push(Some(asm::Command::Node(asm::NamedNode {
                    name: Some(asm::NodeId(lhs.swap_remove(0).0)),
                    node: lower_spec_term(enum_cast!(NestedExpr::Term, rhs)),
                })));
            }
            SpecStmt::Returns(_, e) => {
                let t = enum_cast!(NestedExpr::Term, e);
                let v = enum_cast!(SpecTerm::Var { name, .. }, name, t);
                res.push(Some(asm::Command::TailEdge(asm::TailEdge::Return {
                    return_values: Some(vec![Some(asm::NodeId(v))]),
                })));
            }
        }
    }
    res
}

/// Lower a spec funclet into a caiman assembly funclet.
fn lower_spec_funclet(
    name: String,
    input: Vec<(String, DataType)>,
    output: Option<(Option<String>, DataType)>,
    statements: Vec<SpecStmt>,
    class_name: &str,
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
                        typ: data_type_to_type(&dt),
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
                        typ: data_type_to_type(&dt),
                        tags: Vec::new(),
                    }
                })
                .collect(),
            binding: asm::FuncletBinding::ValueBinding(asm::FunctionClassBinding {
                default: false,
                function_class: asm::FunctionClassId(class_name.to_string()),
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
        let (header, commands) = lower_spec_funclet(name, input, output, statements, class_name);
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
    let (header, commands) = lower_spec_funclet(name, input, Some(output), statements, "");
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
    let (header, commands) = lower_spec_funclet(name, input, Some(output), statements, "");
    asm::Funclet {
        kind: ir::FuncletKind::Spatial,
        header,
        commands,
    }
}
