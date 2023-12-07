use std::collections::BTreeSet;

use crate::{
    enum_cast,
    lower::{binop_to_str, data_type_to_local_type},
    parse::ast::{DataType, SchedExpr, SchedFuncCall, Tags},
};
use caiman::assembly::ast as asm;
pub use caiman::assembly::ast::Hole;

use crate::{
    error::Info,
    parse::ast::{FullType, Name, SchedStmt, SchedTerm},
};

/// High-level caiman IR, excluding tail edges.
///
/// This is an intermediary representation for going from the frontend to assembly.
/// This representation enforces functions to be flattened and split into basic
/// blocks connected by tail edges. These transformations
/// occur before passing the CFG to lowering. I got tired of writing
/// assertions and deep pattern matches so I switched to a different representations
/// which enforces the flattening and splitting assumptions.
#[derive(Debug)]
pub enum Hir {
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
        op: Name,
        args: Vec<SchedTerm>,
    },
    InAnnotation(Info, Vec<(String, Tags)>),
    OutAnnotation(Info, Vec<(String, Tags)>),
}

/// A terminator of a basic block.
/// We use a seperate `Terminator` rather than a `SchedStmt` or `Hir` to allow moving data
/// from the `SchedStmt` and to ensure type safety.
#[derive(Clone, Debug)]
pub enum Terminator {
    /// A call to a function with a list of arguments.
    #[allow(dead_code)]
    Call(SchedFuncCall),
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

/// A reference to an instruction in the high-level IR.
/// Either a tail edge (terminator) or a statement.
///
/// Hir body and terminators are owned by the basic block they are in.
#[allow(clippy::module_name_repetitions)]
pub enum HirInstr<'a> {
    Stmt(&'a mut Hir),
    Tail(&'a mut Terminator),
}

impl Hir {
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
                        op: binop_to_str(op).to_string(),
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

    /// Get the variables used by this statement.
    /// Mutates the given vector by appending the variables used to it.
    pub fn get_uses(&self, res: &mut BTreeSet<String>) {
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

    /// Get the name of the variable defined by this statement, if any.
    /// A `Move` is not considered to have a `def` because it is updating a
    /// reference.
    pub fn get_def(&self) -> Option<String> {
        match self {
            Self::ConstDecl { lhs, .. }
            | Self::VarDecl { lhs, .. }
            | Self::RefLoad { dest: lhs, .. } => Some(lhs.clone()),
            // TODO: re-evaluate the move instruction.
            // Viewing it as a write to a reference, then it had no defs
            Self::Hole(..)
            | Self::RefStore { .. }
            | Self::InAnnotation(..)
            | Self::OutAnnotation(..) => None,
            Self::Op { dest, .. } => Some(dest.clone()),
        }
    }

    /// Gets the local type of the variable defined by this statement, if any.
    pub fn get_def_local_type(&self) -> Option<asm::TypeId> {
        // TODO: flags
        match self {
            Self::ConstDecl { lhs_tag, .. } => lhs_tag
                .as_ref()
                .map(|tag| data_type_to_local_type(&tag.base.base)),
            Self::VarDecl { lhs_tag, .. } => lhs_tag
                .as_ref()
                .map(|tag| make_ref(data_type_to_local_type(&tag.base.base))),
            Self::RefLoad { typ, .. } => Some(data_type_to_local_type(typ)),
            Self::RefStore { .. }
            | Self::Hole(..)
            | Self::InAnnotation(..)
            | Self::OutAnnotation(..)
            | Self::Op { .. } => None,
        }
    }

    /// Renames all uses in this statement using the given function
    pub fn rename_uses(&mut self, f: &mut dyn FnMut(&str) -> String) {
        match self {
            Self::ConstDecl { rhs, .. }
            | Self::VarDecl { rhs: Some(rhs), .. }
            | Self::RefStore { rhs, .. } => {
                term_rename_uses(rhs, f);
            }
            Self::Op { args, .. } => {
                for arg in args {
                    term_rename_uses(arg, f);
                }
            }
            Self::RefLoad { src, .. } => {
                *src = f(src);
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
pub fn stmts_to_hir(stmts: Vec<SchedStmt>) -> Vec<Hir> {
    stmts.into_iter().map(Hir::new).collect()
}

/// Get the uses in a `SchedTerm`
pub fn term_get_uses(t: &SchedTerm, res: &mut BTreeSet<String>) {
    match t {
        SchedTerm::Var { name, .. } => {
            res.insert(name.clone());
        }
        SchedTerm::Hole(..) | SchedTerm::Lit { .. } => (),
        SchedTerm::Call(..) => todo!(),
    }
}

/// Renames all uses in a `SchedTerm` using the given function
pub fn term_rename_uses(t: &mut SchedTerm, f: &mut dyn FnMut(&str) -> String) {
    match t {
        SchedTerm::Var { name, .. } => *name = f(name),
        SchedTerm::Hole(..) | SchedTerm::Lit { .. } => (),
        SchedTerm::Call(..) => todo!(),
    }
}

/// Makes the base type of this type into a reference to the existing type
/// Does not check against references to references
pub(super) fn make_ref(typ: asm::TypeId) -> asm::TypeId {
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
