#![allow(clippy::module_name_repetitions)]
use std::collections::BTreeSet;

use crate::{
    enum_cast,
    lower::data_type_to_local_type,
    parse::ast::{ArgsOrEnc, Binop, DataType, NestedExpr, SchedExpr, SchedFuncCall, Tags, Uop},
};
use caiman::assembly::ast as asm;
pub use caiman::assembly::ast::Hole;

use crate::{
    error::Info,
    parse::ast::{FullType, Name, SchedStmt, SchedTerm},
};

use super::RET_VAR;

/// High-level caiman IR, excluding tail edges.
///
/// This is an intermediary representation for going from the frontend to assembly.
/// This representation enforces functions to be flattened and split into basic
/// blocks connected by tail edges. These transformations
/// occur before passing the CFG to lowering. I got tired of writing
/// assertions and deep pattern matches so I switched to a different representations
/// which enforces the flattening and splitting assumptions.
#[derive(Debug)]
pub enum HirBody {
    // TODO: encodings
    /// A data movement into a mutable variable
    RefStore {
        info: Info,
        lhs_tags: Option<Tags>,
        lhs: Name,
        rhs: SchedTerm,
    },
    /// Load from a reference into a new variable
    RefLoad {
        dest: Name,
        typ: DataType,
        src: Name,
    },
    /// Declaration of an immutable variable
    ConstDecl {
        info: Info,
        lhs: Name,
        lhs_tag: Hole<FullType>,
        rhs: SchedTerm,
    },
    /// Declaration of a mutable variable (reference)
    VarDecl {
        info: Info,
        lhs: Name,
        lhs_tag: Hole<FullType>,
        rhs: Option<SchedTerm>,
    },
    Hole(Info),
    /// Built-in operation (performs a const decl)
    Op {
        info: Info,
        dest: Name,
        dest_tag: Hole<FullType>,
        op: HirOp,
        args: Vec<SchedTerm>,
    },
    InAnnotation(Info, Vec<(String, Tags)>),
    OutAnnotation(Info, Vec<(String, Tags)>),
}

/// A high level IR operation.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[allow(clippy::upper_case_acronyms)]
pub enum HirOp {
    /// an unlowered binary operation
    Binary(Binop),
    /// an unlowered unary operation
    #[allow(dead_code)]
    Unary(Uop),
    /// a lowered operation into an external call
    FFI(Name),
}

impl HirOp {
    /// Lowers a HIR operation into the name of the external function to call.
    /// Panics if the operation is not lowered.
    pub fn lower(&self) -> Name {
        match self {
            Self::Binary(_) | Self::Unary(_) => panic!("Cannot lower unlowered operation"),
            Self::FFI(name) => name.clone(),
        }
    }
}

/// An internal function call in the high-level IR.
#[derive(Clone, Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct HirFuncCall {
    pub target: String,
    pub args: Vec<String>,
    pub tag: Option<Tags>,
}

impl TryFrom<SchedFuncCall> for HirFuncCall {
    type Error = ();
    fn try_from(value: SchedFuncCall) -> Result<Self, Self::Error> {
        if let NestedExpr::Term(SchedTerm::Var { name, .. }) = *value.target {
            if let ArgsOrEnc::Args(args) = *value.args {
                let args = args
                    .into_iter()
                    .map(|a| {
                        enum_cast!(
                            SchedTerm::Var { name, .. },
                            name,
                            enum_cast!(SchedExpr::Term, a)
                        )
                    })
                    .collect();
                return Ok(Self {
                    target: name,
                    args,
                    tag: value.tag,
                });
            }
        }
        panic!("Invalid internal function call")
    }
}

/// A terminator of a basic block.
/// We use a seperate `Terminator` rather than a `SchedStmt` or `Hir` to allow moving data
/// from the `SchedStmt` and to ensure type safety.
#[derive(Clone, Debug)]
pub enum Terminator {
    /// A call to an internal function with a list of destinations to store the
    /// return values in.
    Call(Vec<(String, Option<FullType>)>, HirFuncCall),
    /// A select statement with a guard node. If the guard is true
    /// we transition to the `true_branch` of the outgoing edge of this block
    /// in the CFG. Otherwise, we transition to the `false_branch`.
    Select(String, Option<Tags>),
    /// A return statement with an optional node.
    /// Modeled as an assignment to the special `_out` variable and transition to
    /// the final basic block.
    Return(Option<String>),
    /// The final return statement in the final basic block. This is **NOT**
    /// a return statement in the frontend, but rather a special return statement
    /// for the canonical CFG.
    FinalReturn,
    /// No terminator, continue to the next block. A `None` terminator is just
    /// a temporary value until live vars and tag analysis can be done to know
    /// what the output variables are for the `Next` terminator
    None,
    /// No terminator, continue to next block with the specified returns
    Next(Vec<String>),
}

/// How a variable is used in a statement.
#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub enum UseType {
    Write,
    Read,
}

pub type Args = Vec<(String, Option<asm::TypeId>)>;

/// A generalized HIR instruction which is either a body statement or a terminator.
pub trait Hir {
    /// Get the variables used by this statement.
    /// Mutates the given set by appending the variables used to it.
    fn get_uses(&self, res: &mut BTreeSet<String>);

    /// Get the name and type of the variables defined by this statement, if any.
    /// A def is a **NEW** variable, not a write to an existing variable.
    fn get_defs(&self) -> Option<Args>;

    /// Renames all uses in this statement using the given function which is
    /// passed the name of the variable and the type of use.
    fn rename_uses(&mut self, f: &mut dyn FnMut(&str, UseType) -> String);

    /// Get the variables used by this statement.
    fn get_use_set(&self) -> BTreeSet<String> {
        let mut res = BTreeSet::new();
        self.get_uses(&mut res);
        res
    }
}

impl Hir for Terminator {
    fn get_defs(&self) -> Option<Args> {
        match self {
            Self::Call(defs, ..) => Some(
                defs.iter()
                    .map(|(d, t)| {
                        (
                            d.clone(),
                            t.as_ref().map(|dt| data_type_to_local_type(&dt.base.base)),
                        )
                    })
                    .collect(),
            ),
            Self::Return(..) => Some(vec![(RET_VAR.to_string(), None)]),
            Self::Select(..) | Self::FinalReturn | Self::None | Self::Next(..) => None,
        }
    }

    fn get_uses(&self, uses: &mut BTreeSet<String>) {
        match self {
            Self::Call(_, call) => {
                uses.insert(call.target.clone());
                for arg in &call.args {
                    uses.insert(arg.clone());
                }
            }
            Self::Select(guard, ..) => {
                uses.insert(guard.clone());
            }
            Self::Return(Some(node)) => {
                uses.insert(node.clone());
            }
            Self::FinalReturn => {
                uses.insert(RET_VAR.to_string());
            }
            Self::Return(None) | Self::None | Self::Next(..) => (),
        }
    }

    fn rename_uses(&mut self, f: &mut dyn FnMut(&str, UseType) -> String) {
        match self {
            Self::Call(_, call) => {
                for arg in &mut call.args {
                    *arg = f(arg, UseType::Read);
                }
            }
            Self::Select(guard, ..) => {
                *guard = f(guard, UseType::Read);
            }
            Self::Return(Some(node)) => {
                *node = f(node, UseType::Read);
            }
            Self::FinalReturn | Self::Return(None) | Self::None | Self::Next(..) => (),
        }
    }
}

/// A reference to an instruction in the high-level IR.
/// Either a tail edge (terminator) or a statement.
///
/// Hir body and terminators are owned by the basic block they are in.
///
/// This enum provides the ability to access instruction specific
/// methods not present in the HIR trait or perform instruction
/// matching.
pub enum HirInstr<'a> {
    Stmt(&'a mut HirBody),
    Tail(&'a mut Terminator),
}

impl std::ops::Deref for HirInstr<'_> {
    type Target = dyn Hir;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Stmt(s) => *s,
            Self::Tail(t) => *t,
        }
    }
}

impl std::ops::DerefMut for HirInstr<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Stmt(s) => *s,
            Self::Tail(t) => *t,
        }
    }
}

impl HirBody {
    pub fn new(stmt: SchedStmt) -> Self {
        // TODO: operations
        match stmt {
            SchedStmt::Assign {
                info,
                tag,
                lhs,
                rhs,
            } => {
                let rhs = enum_cast!(SchedExpr::Term, rhs);
                Self::RefStore {
                    info,
                    lhs_tags: tag,
                    lhs,
                    rhs,
                }
            }
            SchedStmt::Decl {
                info,
                lhs,
                expr: Some(expr),
                is_const: true,
            } => match expr {
                SchedExpr::Term(rhs) => Self::ConstDecl {
                    info,
                    lhs: lhs[0].0.clone(),
                    lhs_tag: lhs[0].1.clone(),
                    rhs,
                },
                SchedExpr::Binop {
                    info,
                    op,
                    lhs: op_lhs,
                    rhs: op_rhs,
                } => {
                    let lhs_term = enum_cast!(SchedExpr::Term, op_lhs.as_ref());
                    let rhs_term = enum_cast!(SchedExpr::Term, op_rhs.as_ref());
                    Self::Op {
                        info,
                        dest: lhs[0].0.clone(),
                        dest_tag: lhs[0].1.clone(),
                        op: HirOp::Binary(op),
                        args: vec![lhs_term.clone(), rhs_term.clone()],
                    }
                }
                _ => todo!(),
            },
            SchedStmt::Decl {
                info,
                lhs,
                expr,
                is_const: false,
            } => {
                let rhs = expr.map(|x| enum_cast!(SchedExpr::Term, x));
                Self::VarDecl {
                    info,
                    lhs: lhs[0].0.clone(),
                    lhs_tag: lhs[0].1.clone(),
                    rhs,
                }
            }
            SchedStmt::Decl { .. } => panic!("Invalid declaration"),
            SchedStmt::Return(..)
            | SchedStmt::Block(..)
            | SchedStmt::If { .. }
            | SchedStmt::Call(..) => {
                panic!("Unexpected stmt")
            }
            SchedStmt::Hole(info) => Self::Hole(info),
            SchedStmt::InEdgeAnnotation { info, tags } => Self::InAnnotation(info, tags),
            SchedStmt::OutEdgeAnnotation { info, tags } => Self::OutAnnotation(info, tags),
        }
    }
}
impl Hir for HirBody {
    fn get_uses(&self, res: &mut BTreeSet<String>) {
        match self {
            Self::ConstDecl { rhs, .. } => {
                term_get_uses(rhs, res);
            }
            Self::VarDecl { rhs, .. } => {
                if let Some(rhs) = rhs {
                    term_get_uses(rhs, res);
                }
            }
            Self::RefStore { lhs, rhs, .. } => {
                term_get_uses(rhs, res);
                // Viewing this as a store to a reference, then the destination
                // is a use
                res.insert(lhs.clone());
            }
            Self::RefLoad { src, .. } => {
                res.insert(src.clone());
            }
            Self::Op { args, .. } => {
                for arg in args {
                    term_get_uses(arg, res);
                }
            }
            Self::InAnnotation(..) | Self::OutAnnotation(..) | Self::Hole(..) => (),
        }
    }

    fn get_defs(&self) -> Option<Args> {
        match self {
            Self::ConstDecl { lhs, lhs_tag, .. } => {
                Some(vec![(
                    lhs.clone(),
                    lhs_tag
                        .as_ref()
                        .map(|tag| data_type_to_local_type(&tag.base.base)),
                )])
            }
            Self::VarDecl { lhs, lhs_tag, ..} => {
                Some(vec![(
                    lhs.clone(),
                    lhs_tag
                        .as_ref()
                        .map(|tag| make_ref(data_type_to_local_type(&tag.base.base))),
                )])
            }
            Self::RefLoad { dest: lhs, typ, .. } => {
                Some(vec![(lhs.clone(), Some(data_type_to_local_type(typ)))])
            }
            // TODO: re-evaluate the move instruction.
            // Viewing it as a write to a reference, then it had no defs
            Self::Hole(..)
            // RefStore doesn't have a def bc it's a store to a reference
            | Self::RefStore { .. }
            | Self::InAnnotation(..)
            | Self::OutAnnotation(..) => None,
            Self::Op { dest, dest_tag, .. } => Some(vec![(
                dest.clone(),
                dest_tag.as_ref().map(|tag| data_type_to_local_type(&tag.base.base)),
            )]),
        }
    }

    fn rename_uses(&mut self, f: &mut dyn FnMut(&str, UseType) -> String) {
        match self {
            Self::ConstDecl { rhs, .. } | Self::VarDecl { rhs: Some(rhs), .. } => {
                term_rename_uses(rhs, &mut |name| f(name, UseType::Read));
            }
            Self::RefStore { lhs, rhs, .. } => {
                term_rename_uses(rhs, &mut |name| f(name, UseType::Read));
                *lhs = f(lhs, UseType::Write);
            }
            Self::Op { args, .. } => {
                for arg in args {
                    term_rename_uses(arg, &mut |name| f(name, UseType::Read));
                }
            }
            Self::RefLoad { src, .. } => {
                *src = f(src, UseType::Read);
            }
            Self::Hole(..)
            | Self::InAnnotation(..)
            | Self::OutAnnotation(..)
            | Self::VarDecl { rhs: None, .. } => (),
        }
    }
}

/// Convert a list of `SchedStmts` to a list of Hirs
#[allow(clippy::module_name_repetitions)]
pub fn stmts_to_hir(stmts: Vec<SchedStmt>) -> Vec<HirBody> {
    stmts.into_iter().map(HirBody::new).collect()
}

/// Get the uses in a `SchedTerm`
fn term_get_uses(t: &SchedTerm, res: &mut BTreeSet<String>) {
    match t {
        SchedTerm::Var { name, .. } => {
            res.insert(name.clone());
        }
        SchedTerm::Hole(..) | SchedTerm::Lit { .. } => (),
        SchedTerm::Call(..) => todo!(),
    }
}

/// Renames all uses in a `SchedTerm` using the given function
fn term_rename_uses(t: &mut SchedTerm, f: &mut dyn FnMut(&str) -> String) {
    match t {
        SchedTerm::Var { name, .. } => *name = f(name),
        SchedTerm::Hole(..) | SchedTerm::Lit { .. } => (),
        SchedTerm::Call(..) => todo!(),
    }
}

/// Makes the base type of this type into a reference to the existing type
/// Does not check against references to references
fn make_ref(typ: asm::TypeId) -> asm::TypeId {
    match typ {
        asm::TypeId::Local(type_name) => asm::TypeId::Local(format!("&{type_name}")),
        asm::TypeId::FFI(_) => todo!(),
    }
}

/// Makes the base type of this type into a dereference to the existing type
/// Does not check against references to references
pub(super) fn make_deref(typ: &asm::TypeId) -> asm::TypeId {
    match typ {
        asm::TypeId::Local(type_name) => {
            assert_eq!(&type_name[0..1], "&");
            asm::TypeId::Local(type_name[1..].to_string())
        }
        asm::TypeId::FFI(_) => todo!(),
    }
}
