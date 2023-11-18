use std::collections::HashSet;

use crate::{
    enum_cast,
    parse::ast::{SchedExpr, SchedFuncCall},
};
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
    Move {
        info: Info,
        lhs: Name,
        rhs: SchedTerm,
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
    /// Built-in operation
    #[allow(dead_code)]
    Op {
        info: Info,
        dest: Name,
        op: Name,
        args: Vec<SchedTerm>,
    },
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
    Select(String),
    /// A return statement with an optional node.
    /// Modeled as an assignment to the special `_out` variable and transition to
    /// the final basic block.
    Return(Option<String>),
    /// The final return statement in the final basic block. This is **NOT**
    /// a return statement in the frontend, but rather a special return statement
    /// for the canonical CFG.
    FinalReturn,
    /// No terminator, continue to the next block
    None,
}

/// A reference to an instruction in the high-level IR.
/// Either a tail edge (terminator) or a statement.
///
/// Hir body and terminators are owned by the basic block they are in.
#[allow(clippy::module_name_repetitions)]
pub enum HirInstr<'a> {
    Stmt(&'a Hir),
    Tail(&'a Terminator),
}

impl Hir {
    pub fn new(stmt: SchedStmt) -> Self {
        // TODO: operations
        match stmt {
            SchedStmt::Assign { info, lhs, rhs } => {
                let rhs = enum_cast!(SchedExpr::Term, rhs);
                Self::Move { info, lhs, rhs }
            }
            SchedStmt::Decl {
                info,
                lhs,
                expr: Some(expr),
                is_const: true,
            } => {
                let rhs = enum_cast!(SchedExpr::Term, expr);
                Self::ConstDecl {
                    info,
                    lhs: lhs[0].0.clone(),
                    lhs_tag: lhs[0].1.clone(),
                    rhs,
                }
            }
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
        }
    }

    /// Get the variables used by this statement.
    /// Mutates the given vector by appending the variables used to it.
    pub fn get_uses(&self, res: &mut HashSet<String>) {
        match self {
            Self::ConstDecl { rhs, .. } => {
                term_get_uses(rhs, res);
            }
            Self::VarDecl { rhs, .. } => {
                if let Some(rhs) = rhs {
                    term_get_uses(rhs, res);
                }
            }
            Self::Move { lhs, rhs, .. } => {
                term_get_uses(rhs, res);
                // Viewing this as a store to a reference, then the destination
                // is a use
                res.insert(lhs.clone());
            }
            Self::Hole(..) => (),
            Self::Op { args, .. } => {
                for arg in args {
                    term_get_uses(arg, res);
                }
            }
        }
    }

    /// Get the name of the variable defined by this statement, if any.
    /// A `Move` is not considered to have a `def` because it is updating a
    /// reference.
    pub fn get_def(&self) -> Option<String> {
        match self {
            Self::ConstDecl { lhs, .. } | Self::VarDecl { lhs, .. } => Some(lhs.clone()),
            // TODO: re-evaluate the move instruction.
            // Viewing it as a write to a reference, then it had no defs
            Self::Hole(..) | Self::Move { .. } => None,
            Self::Op { dest, .. } => Some(dest.clone()),
        }
    }
}

/// Convert a list of `SchedStmts` to a list of Hirs
#[allow(clippy::module_name_repetitions)]
pub fn stmts_to_hir(stmts: Vec<SchedStmt>) -> Vec<Hir> {
    stmts.into_iter().map(Hir::new).collect()
}

/// Get the uses in a `SchedTerm`
fn term_get_uses(t: &SchedTerm, res: &mut HashSet<String>) {
    match t {
        SchedTerm::Var { name, .. } => {
            res.insert(name.clone());
        }
        SchedTerm::Hole(..) | SchedTerm::Lit { .. } => (),
        SchedTerm::Call(..) => todo!(),
    }
}
