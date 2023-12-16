use std::collections::{HashMap, HashSet};

use crate::{
    lower::BOOL_FFI_TYPE,
    parse::ast::{
        Binop, ClassMembers, DataType, FloatSize, IntSize, SchedStmt, SpecExpr, SpecLiteral,
        SpecStmt, SpecTerm, TopLevel,
    },
};
use caiman::assembly::ast as asm;
use caiman::ir;

use super::{binop_to_str, data_type_to_ffi, data_type_to_ffi_type};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The type of a spec.
pub enum SpecType {
    Value,
    Timeline,
    Spatial,
}

/// A global context for a caiman program. This contains information about constants,
/// type aliases, and function signatures.
pub struct Context {
    pub specs: HashMap<String, SpecType>,
    pub type_decls: Vec<asm::Declaration>,
    /// Signatures of value specs. Map from spec name to (input types, output types).
    pub signatures: HashMap<String, (Vec<DataType>, Vec<DataType>)>,
    /// Map from spec name to a map from variable name to type.
    pub value_types: HashMap<String, HashMap<String, DataType>>,
    /// Map from sched function name to a map from variable name to type.
    /// The variables in this map are the ones present at the source level ONLY.
    pub sched_types: HashMap<String, HashMap<String, DataType>>,
}

fn gen_type_decls(_tl: &[TopLevel]) -> Vec<asm::Declaration> {
    // collect used types
    vec![
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("BufferSpace"),
            data: asm::LocalTypeInfo::BufferSpace,
        })),
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("Event"),
            data: asm::LocalTypeInfo::Event,
        })),
        asm::Declaration::TypeDecl(asm::TypeDecl::FFI(asm::FFIType::I64)),
        asm::Declaration::TypeDecl(asm::TypeDecl::FFI(BOOL_FFI_TYPE)),
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("bool"),
            data: asm::LocalTypeInfo::NativeValue {
                storage_type: asm::TypeId::FFI(BOOL_FFI_TYPE),
            },
        })),
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("&bool"),
            data: asm::LocalTypeInfo::Ref {
                storage_type: asm::TypeId::FFI(BOOL_FFI_TYPE),
                storage_place: ir::Place::Local,
                buffer_flags: ir::BufferFlags::new(),
            },
        })),
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("i64"),
            data: asm::LocalTypeInfo::NativeValue {
                storage_type: asm::TypeId::FFI(asm::FFIType::I64),
            },
        })),
        asm::Declaration::TypeDecl(asm::TypeDecl::Local(asm::LocalType {
            name: String::from("&i64"),
            data: asm::LocalTypeInfo::Ref {
                storage_type: asm::TypeId::FFI(asm::FFIType::I64),
                storage_place: ir::Place::Local,
                buffer_flags: ir::BufferFlags::new(),
            },
        })),
    ]
}

/// Returns the output type of a binary operation.
fn op_output_type(op: Binop, op_l: &DataType, op_r: &DataType) -> DataType {
    match op {
        // TODO: argument promotion
        Binop::Add | Binop::Sub | Binop::Mul | Binop::Div => {
            assert!(matches!(op_l, DataType::Int(_) | DataType::Float(_)));
            assert!(matches!(op_r, DataType::Int(_) | DataType::Float(_)));
            op_l.clone()
        }
        Binop::Lt
        | Binop::Gt
        | Binop::Geq
        | Binop::Leq
        | Binop::Eq
        | Binop::Neq
        | Binop::Land
        | Binop::Lor => DataType::Bool,
        Binop::And
        | Binop::Or
        | Binop::Xor
        | Binop::AShr
        | Binop::Shr
        | Binop::Shl
        | Binop::Mod => {
            assert!(matches!(op_l, DataType::Int(_)));
            assert!(matches!(op_r, DataType::Int(_)));
            op_l.clone()
        }
        Binop::Dot | Binop::Range | Binop::Index | Binop::Cons => todo!(),
    }
}

/// A typed binary operation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TypedBinop {
    op: Binop,
    /// The type of the left operand.
    op_l: DataType,
    /// The type of the right operand.
    op_r: DataType,
    /// The type of the result.
    ret: DataType,
}

/// Collects all names defined in a given spec.
fn collect_spec_names(stmts: &Vec<SpecStmt>) -> HashSet<String> {
    let mut res = HashSet::new();
    for stmt in stmts {
        match stmt {
            SpecStmt::Assign { lhs, .. } => {
                for (name, _) in lhs {
                    res.insert(name.clone());
                }
            }
            SpecStmt::Returns(..) => (),
        }
    }
    res
}

/// Collects all types of variables used in a given statement for an assignment
/// `lhs :- t`
fn collect_spec_assign_term(
    t: &SpecTerm,
    lhs: &[(String, Option<DataType>)],
    types: &mut HashMap<String, DataType>,
    signatures: &HashMap<String, (Vec<DataType>, Vec<DataType>)>,
) {
    match t {
        SpecTerm::Lit { lit, .. } => match lit {
            SpecLiteral::Int(_) => {
                types.insert(lhs[0].0.clone(), DataType::Int(IntSize::I64));
            }
            SpecLiteral::Bool(_) => {
                types.insert(lhs[0].0.clone(), DataType::Bool);
            }
            SpecLiteral::Float(_) => {
                types.insert(lhs[0].0.clone(), DataType::Float(FloatSize::F64));
            }
            _ => todo!(),
        },
        SpecTerm::Var { .. } => todo!(),
        SpecTerm::Call { function, .. } => {
            if let SpecExpr::Term(SpecTerm::Var { name, .. }) = &**function {
                for ((name, annot), typ) in lhs.iter().zip(signatures.get(name).unwrap().1.iter()) {
                    if let Some(a) = annot {
                        assert_eq!(a, typ);
                    }
                    types.insert(name.clone(), typ.clone());
                }
            }
        }
    }
}

/// Collects all types of variables used in a given statement for an assignment
/// `lhs :- if_true if guard if_false`.
///
/// Returns `true` if the collection failed and should be retried at the next iteration.
///
/// # Panics
/// Panics if the statement is not lowered or it uses a variable that is
/// undefined (i.e. not present in `names`).
fn collect_spec_assign_if(
    lhs: &[(String, Option<DataType>)],
    if_true: &SpecExpr,
    if_false: &SpecExpr,
    guard: &SpecExpr,
    types: &mut HashMap<String, DataType>,
    names: &HashSet<String>,
) -> bool {
    if let (
        SpecExpr::Term(SpecTerm::Var { name: name1, .. }),
        SpecExpr::Term(SpecTerm::Var { name: name2, .. }),
        SpecExpr::Term(SpecTerm::Var { name: guard, .. }),
    ) = (if_true, if_false, guard)
    {
        if !types.contains_key(guard) || !types.contains_key(name1) || !types.contains_key(name2) {
            assert!(names.contains(guard), "Undefined node: {guard}");
            assert!(names.contains(name1), "Undefined node: {name1}");
            assert!(names.contains(name2), "Undefined node: {name2}");
            return true;
        }
        assert_eq!(types[guard], DataType::Bool);
        assert_eq!(
            types[name1], types[name2],
            "Conditional types must be equal"
        );
        types.insert(lhs[0].0.clone(), types[name1].clone());
    } else {
        panic!("Not lowered")
    }
    false
}

/// Collects all types of variables used in a given statement for an assignment
/// `lhs :- op_l op op_r`.
///
/// Returns `true` if the collection failed and should be retried at the next iteration.
///
/// # Panics
/// Panics if the statement is not lowered or it uses a variable that is
/// undefined (i.e. not present in `names`).
fn collect_spec_assign_bop(
    op_l: &SpecExpr,
    op_r: &SpecExpr,
    op: Binop,
    types: &mut HashMap<String, DataType>,
    externs: &mut HashSet<TypedBinop>,
    lhs: &[(String, Option<DataType>)],
    names: &HashSet<String>,
) -> bool {
    if let (
        SpecExpr::Term(SpecTerm::Var { name: name1, .. }),
        SpecExpr::Term(SpecTerm::Var { name: name2, .. }),
    ) = (op_l, op_r)
    {
        if !types.contains_key(name1) || !types.contains_key(name2) {
            assert!(names.contains(name1), "Undefined node: {name1}");
            assert!(names.contains(name2), "Undefined node: {name2}");
            return true;
        }
        let ret = op_output_type(op, &types[name1], &types[name2]);
        types.insert(lhs[0].0.clone(), ret.clone());
        externs.insert(TypedBinop {
            op,
            op_l: types[name1].clone(),
            op_r: types[name2].clone(),
            ret,
        });
    } else {
        panic!("Not lowered")
    }
    false
}

/// Collects all extern operations used in a given spec and collects all types
/// of variables used in the spec.
/// # Arguments
/// * `stmts` - the statements to scan
/// * `externs` - a set of all extern operations used in `stmts`. This is updated
/// as we scan `stmts` for all new extern operations.
/// * `types` - a map from variable names to their types. This is updated as
/// we scan `stmts` for all new variables.
/// * `signatures` - a map from spec names to their signatures
fn collect_spec(
    stmts: &Vec<SpecStmt>,
    externs: &mut HashSet<TypedBinop>,
    types: &mut HashMap<String, DataType>,
    signatures: &HashMap<String, (Vec<DataType>, Vec<DataType>)>,
) {
    let names = collect_spec_names(stmts);
    let mut skipped = true;
    // specs are unordered, so iterate until no change.
    while skipped {
        skipped = false;
        for stmt in stmts {
            match stmt {
                SpecStmt::Assign { lhs, rhs, .. } => match rhs {
                    SpecExpr::Term(t) => collect_spec_assign_term(t, lhs, types, signatures),
                    SpecExpr::Conditional {
                        if_true,
                        guard,
                        if_false,
                        ..
                    } => {
                        if collect_spec_assign_if(lhs, if_true, if_false, guard, types, &names) {
                            skipped = true;
                            continue;
                        }
                    }
                    SpecExpr::Binop {
                        op,
                        lhs: op_l,
                        rhs: op_r,
                        ..
                    } => {
                        if collect_spec_assign_bop(op_l, op_r, *op, types, externs, lhs, &names) {
                            skipped = true;
                            continue;
                        }
                    }
                    SpecExpr::Uop { .. } => todo!(),
                },
                SpecStmt::Returns(..) => (),
            }
        }
    }
}

/// Collects a mapping between high-level variables and their types.
fn collect_sched_types(stmts: &Vec<SchedStmt>, types: &mut HashMap<String, DataType>) {
    for s in stmts {
        match s {
            SchedStmt::Decl { lhs, .. } => {
                for (name, tag) in lhs {
                    tag.as_ref()
                        .map(|t| types.insert(name.clone(), t.base.base.clone()));
                }
            }
            SchedStmt::Block(_, stmts) => {
                collect_sched_types(stmts, types);
            }
            _ => (),
        }
    }
}

/// Generates a list of extern declarations needed for a given program.
fn gen_extern_decls(
    tl: &[TopLevel],
    signatures: &HashMap<String, (Vec<DataType>, Vec<DataType>)>,
) -> (
    Vec<asm::Declaration>,
    HashMap<String, HashMap<String, DataType>>,
) {
    let mut existing_externs = HashSet::new();
    let mut all_types = HashMap::new();
    // TODO: do we need to scan schedules?
    for decl in tl {
        if let TopLevel::FunctionClass { members, .. } = decl {
            for m in members {
                if let ClassMembers::ValueFunclet {
                    statements, input, ..
                } = m
                {
                    let mut types = HashMap::new();
                    for (name, typ) in input {
                        types.insert(name.clone(), typ.clone());
                    }
                    collect_spec(statements, &mut existing_externs, &mut types, signatures);
                    all_types.insert(m.get_name(), types);
                }
            }
        }
    }
    (get_extern_decls(&existing_externs), all_types)
}

/// Returns a list of extern declarations needed for a given expression.
fn get_extern_decls(existing_externs: &HashSet<TypedBinop>) -> Vec<asm::Declaration> {
    let mut res = vec![];
    for TypedBinop {
        op,
        op_l,
        op_r,
        ret,
    } in existing_externs
    {
        let op_name = binop_to_str(*op, &format!("{op_l:#}"), &format!("{op_r:#}")).to_string();
        res.extend(
            [
                asm::Declaration::FunctionClass(asm::FunctionClass {
                    name: asm::FunctionClassId(op_name.clone()),
                    input_types: vec![data_type_to_ffi_type(op_l), data_type_to_ffi_type(op_r)],
                    output_types: vec![data_type_to_ffi_type(ret)],
                }),
                asm::Declaration::ExternalFunction(asm::ExternalFunction {
                    name: op_name.clone(),
                    kind: asm::ExternalFunctionKind::CPUPure,
                    value_function_binding: asm::FunctionClassBinding {
                        default: false,
                        function_class: asm::FunctionClassId(op_name.clone()),
                    },
                    input_args: vec![
                        asm::ExternalArgument {
                            name: None,
                            ffi_type: data_type_to_ffi(op_l).unwrap(),
                        },
                        asm::ExternalArgument {
                            name: None,
                            ffi_type: data_type_to_ffi(op_r).unwrap(),
                        },
                    ],
                    output_types: vec![asm::ExternalArgument {
                        name: None,
                        ffi_type: data_type_to_ffi(ret).unwrap(),
                    }],
                }),
            ]
            .into_iter(),
        );
    }
    res
}

/// Creates a global context from a list of top-level declarations.
pub fn gen_context(tl: &[TopLevel]) -> Context {
    let mut ctx = Context {
        specs: HashMap::new(),
        type_decls: gen_type_decls(tl).into_iter().collect(),
        signatures: HashMap::new(),
        value_types: HashMap::new(),
        sched_types: HashMap::new(),
    };
    for decl in tl {
        match decl {
            TopLevel::SpatialFunclet { name, .. } => {
                ctx.specs.insert(name.to_string(), SpecType::Spatial);
            }
            TopLevel::TimelineFunclet { name, .. } => {
                ctx.specs.insert(name.to_string(), SpecType::Timeline);
            }
            TopLevel::FunctionClass {
                name: class_name,
                members,
                ..
            } => {
                let mut member_name = None;
                for m in members {
                    match m {
                        ClassMembers::ValueFunclet {
                            name,
                            input,
                            output,
                            ..
                        } => {
                            member_name = Some(name.to_string());
                            ctx.specs.insert(name.to_string(), SpecType::Value);
                            ctx.signatures.insert(
                                name.to_string(),
                                (
                                    input.iter().map(|x| x.1.clone()).collect(),
                                    output.as_ref().map_or_else(Vec::new, |x| vec![x.1.clone()]),
                                ),
                            );
                        }
                        ClassMembers::Extern {
                            name,
                            input,
                            output,
                            ..
                        } => {
                            member_name = Some(name.to_string());
                            ctx.signatures.insert(
                                name.to_string(),
                                (
                                    input.iter().map(|x| x.1.clone()).collect(),
                                    output.as_ref().map_or_else(Vec::new, |x| vec![x.1.clone()]),
                                ),
                            );
                        }
                    }
                }
                ctx.signatures.insert(
                    class_name.clone(),
                    ctx.signatures[&member_name.unwrap()].clone(),
                );
            }
            TopLevel::SchedulingFunc {
                name,
                input,
                output,
                statements,
                ..
            } => {
                let mut types = HashMap::new();
                for (name, typ) in input {
                    types.insert(name.clone(), typ.base.base.clone());
                }
                collect_sched_types(statements, &mut types);
                ctx.sched_types.insert(name.to_string(), types);
                ctx.signatures.insert(
                    name.to_string(),
                    (
                        input.iter().map(|x| x.1.base.base.clone()).collect(),
                        output
                            .as_ref()
                            .map_or_else(Vec::new, |x| vec![x.base.base.clone()]),
                    ),
                );
            }
            _ => (),
        }
    }
    let (extern_decls, types) = gen_extern_decls(tl, &ctx.signatures);
    ctx.type_decls.extend(extern_decls);
    ctx.value_types = types;
    ctx
}
