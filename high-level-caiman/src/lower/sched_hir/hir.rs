#![allow(clippy::module_name_repetitions)]
use std::{collections::{BTreeSet, HashMap}, rc::Rc};

use crate::{
    enum_cast,
    parse::ast::{ArgsOrEnc, Binop, DataType, NestedExpr, SchedExpr, SchedFuncCall, Tags, Uop, Tag, QuotientReference, FullType},
};
use caiman::assembly::ast as asm;
pub use caiman::assembly::ast::Hole;

use crate::{
    error::Info,
    parse::ast::{Name, SchedStmt, SchedTerm},
};

use super::{RET_VAR, Specs};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TripleTag {
    pub value: Option<Tag>,
    pub spatial: Option<Tag>,
    pub timeline: Option<Tag>,
    pub specs: Rc<Specs>
}

impl TripleTag {
    pub fn from_opt(tags: &Option<Tags>, specs: &Rc<Specs>) -> Self {
        tags.as_ref().map_or_else(|| Self::from_owned_opt(None, specs), |tags| Self::from_tags(tags, specs))
    }

    pub fn from_owned_opt(tags: Option<Tags>, specs: &Rc<Specs>) -> Self {
        tags.map_or_else(|| Self {
                value: None,
                spatial: None,
                timeline: None,
                specs: specs.clone()
            }, |tags| Self::from_tag_vec(tags, specs))
    }

    pub fn from_tag_vec(tags: Vec<Tag>, specs: &Rc<Specs>) -> Self {
        let mut value = None;
        let mut spatial = None;
        let mut timeline = None;
        for tag in tags {
            if let Tag { quot_var: Some(QuotientReference {
                spec_name, ..
            }), ..} = &tag {
                if &specs.value.0 == spec_name {
                    value = Some(tag.clone());
                } else if &specs.spatial.0 == spec_name{
                    spatial = Some(tag.clone());
                } else if &specs.timeline.0 == spec_name {
                    timeline = Some(tag.clone());
                }
            }
        }
        Self {
            value,
            spatial,
            timeline,
            specs: specs.clone()
        }
    }

    pub fn from_tags(tags: &[Tag], specs: &Rc<Specs>) -> Self {
        let mut value = None;
        let mut spatial = None;
        let mut timeline = None;
        for tag in tags {
            if let Tag { quot_var: Some(QuotientReference {
                spec_name, ..
            }), ..} = tag {
                if &specs.value.0 == spec_name {
                    value = Some(tag.clone());
                } else if &specs.spatial.0 == spec_name{
                    spatial = Some(tag.clone());
                } else if &specs.timeline.0 == spec_name {
                    timeline = Some(tag.clone());
                }
            }
        }
        Self {
            value,
            spatial,
            timeline,
            specs: specs.clone()
        }
    }

    pub fn from_fulltype(ft: &FullType, specs: &Rc<Specs>) -> Self {
        Self::from_tags(&ft.tags, specs)
    }

    pub fn from_fulltype_opt(ft: &Option<FullType>, specs: &Rc<Specs>) -> Self {
        ft.as_ref().map_or_else(|| Self::from_owned_opt(None, specs), |ft| Self::from_fulltype(ft, specs))
    }

    pub const fn is_any_specified(&self) -> bool {
        self.value.is_some() || self.spatial.is_some() || self.timeline.is_some()
    }
}

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
        lhs_tags: TripleTag,
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
        lhs_tag: TripleTag,
        rhs: SchedTerm,
    },
    /// Declaration of a mutable variable (reference)
    VarDecl {
        info: Info,
        lhs: Name,
        lhs_tag: TripleTag,
        rhs: Option<SchedTerm>,
    },
    Hole(Info),
    /// Built-in operation (performs a const decl)
    Op {
        info: Info,
        dest: Name,
        dest_tag: TripleTag,
        op: HirOp,
        args: Vec<SchedTerm>,
    },
    InAnnotation(Info, Vec<(String, TripleTag)>),
    OutAnnotation(Info, Vec<(String, TripleTag)>),
    Phi {
        dest: Name,
        /// Map from incoming block id to the incoming variable name 
        /// from that block
        inputs: HashMap<usize, Name>,
        /// original name of the variable
        original: Name,
    },
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
    pub tag: TripleTag,
}

impl HirFuncCall {
    pub fn new(value: SchedFuncCall, specs: &Rc<Specs>) ->Self {
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
                return Self {
                    target: name,
                    args,
                    tag: TripleTag::from_opt(&value.tag, specs),
                };
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
    /// return values in. The destinations are **NEW** variables, not writes to
    /// existing variables. This terminator is replaced by a `CaptureCall` terminator
    /// when analyses are complete.
    Call(Vec<(String, TripleTag)>, HirFuncCall),
    /// A call to an internal function with a list of destinations to store the
    /// return values in and a list of variables to capture. The destinations are
    /// **NEW** variables, not writes to existing variables.
    CaptureCall {
        dests: Vec<(String, TripleTag)>,
        call: HirFuncCall,
        captures: BTreeSet<String>,
    },
    /// A select statement with a guard node. If the guard is true
    /// we transition to the `true_branch` of the outgoing edge of this block
    /// in the CFG. Otherwise, we transition to the `false_branch`.
    Select {
        dests: Vec<(String, TripleTag)>,
        guard: String,
        tag: TripleTag,
    },
    /// A return statement which returns values to the parent scope, **NOT** out
    /// of the function. This is modeled as an assignment to the destination variables.
    /// For returning from the function, the destination variables are
    /// `_out0`, `_out1`, etc.
    Return {
        dests: Vec<(String, TripleTag)>,
        rets: Vec<String>,
    },
    /// The final return statement in the final basic block. This is **NOT**
    /// a return statement in the frontend, but rather a special return statement
    /// for final block in the canonical CFG. Takes an argument which is
    /// how many return values there are. Essentially returns `_out0` to `_out{n-1}`
    FinalReturn(usize),
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

/// A generalized HIR instruction which is either a body statement or a terminator.
pub trait Hir {
    /// Get the variables used by this statement.
    /// Mutates the given set by appending the variables used to it.
    fn get_uses(&self, res: &mut BTreeSet<String>);

    /// Get the name and type of the variables defined by this statement, if any.
    /// A def is a **NEW** variable, not a write to an existing variable.
    fn get_defs(&self) -> Option<Vec<String>>;

    /// Renames all uses in this statement using the given function which is
    /// passed the name of the variable and the type of use.
    fn rename_uses(&mut self, f: &mut dyn FnMut(&str, UseType) -> String);

    /// Renames all defs in this statement using the given function which is
    /// passed the name of the variable.
    fn rename_defs(&mut self, f: &mut dyn FnMut(&str) -> String);

    /// Get the variables used by this statement.
    fn get_use_set(&self) -> BTreeSet<String> {
        let mut res = BTreeSet::new();
        self.get_uses(&mut res);
        res
    }
}

impl Hir for Terminator {
    fn get_defs(&self) -> Option<Vec<String>> {
        match self {
            Self::Call(defs, ..)
            | Self::CaptureCall { dests: defs, .. }
            | Self::Return { dests: defs, .. } => {
                Some(defs.iter().map(|(d, _)| d.clone()).collect())
            }
            // we don't consider the defs of a select to be defs of this terminator,
            // but rather they are the defs of the left and right funclets
            Self::FinalReturn(_) | Self::Select { .. } | Self::None | Self::Next(..) => None,
        }
    }

    fn get_uses(&self, uses: &mut BTreeSet<String>) {
        match self {
            Self::Call(_, call) | Self::CaptureCall { call, .. } => {
                for arg in &call.args {
                    uses.insert(arg.clone());
                }
            }
            Self::Select { guard, .. } => {
                uses.insert(guard.clone());
            }
            Self::Return { rets, .. } | Self::Next(rets) => {
                for node in rets {
                    uses.insert(node.clone());
                }
            }
            Self::FinalReturn(n) => {
                for i in 0..*n {
                    uses.insert(format!("{RET_VAR}{i}"));
                }
            }
            Self::None => (),
        }
    }

    fn rename_uses(&mut self, f: &mut dyn FnMut(&str, UseType) -> String) {
        match self {
            Self::Call(_, call) | Self::CaptureCall { call, .. } => {
                for arg in &mut call.args {
                    *arg = f(arg, UseType::Read);
                }
            }
            Self::Select { guard, .. } => {
                *guard = f(guard, UseType::Read);
            }
            Self::Return { rets, .. } | Self::Next(rets) => {
                for node in rets {
                    *node = f(node, UseType::Read);
                }
            }
            Self::FinalReturn(_) | Self::None => (),
        }
    }

    fn rename_defs(&mut self, f: &mut dyn FnMut(&str) -> String) {
        match self {
            Self::Call(defs, ..)
            | Self::CaptureCall { dests: defs, .. }
            | Self::Return { dests: defs, .. } => {
                for (dest, _) in defs {
                    *dest = f(dest);
                }
            }
            Self::FinalReturn(_) | Self::Select { .. } | Self::None | Self::Next(..) => (),
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
    pub fn new(stmt: SchedStmt, specs: &Rc<Specs>) -> Self {
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
                    lhs_tags: TripleTag::from_opt(&tag, specs),
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
                    lhs_tag: TripleTag::from_fulltype_opt(&lhs[0].1, specs),
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
                        dest_tag: TripleTag::from_fulltype_opt(&lhs[0].1, specs),
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
                    lhs_tag: TripleTag::from_fulltype_opt(&lhs[0].1, specs),
                    rhs,
                }
            }
            SchedStmt::Decl { .. } => panic!("Invalid declaration"),
            SchedStmt::Return(..)
            | SchedStmt::Block(..)
            | SchedStmt::If { .. }
            | SchedStmt::Seq { .. }
            | SchedStmt::Call(..) => {
                panic!("Unexpected stmt")
            }
            SchedStmt::Hole(info) => Self::Hole(info),
            SchedStmt::InEdgeAnnotation { info, tags } => Self::InAnnotation(info, tags.into_iter().map(|(name, tags)| (name, TripleTag::from_tag_vec(tags, specs))).collect()),
            SchedStmt::OutEdgeAnnotation { info, tags } => Self::OutAnnotation(info, tags.into_iter().map(|(name, tags)| (name, TripleTag::from_tag_vec(tags, specs))).collect()),
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
            Self::Phi {inputs, ..} => {
                res.extend(inputs.iter().map(|(_, name)| name.clone()));
            }
        }
    }

    fn get_defs(&self) -> Option<Vec<String>> {
        match self {
            Self::ConstDecl { lhs,  .. } | Self::VarDecl { lhs, .. } 
            | Self::RefLoad { dest: lhs, ..} | Self::Op { dest: lhs, ..} |
            Self::Phi { dest: lhs, ..}=> {
                Some(vec![lhs.clone()])
            }
            // TODO: re-evaluate the move instruction.
            // Viewing it as a write to a reference, then it had no defs
            Self::Hole(..)
            // RefStore doesn't have a def bc it's a store to a reference
            | Self::RefStore { .. }
            | Self::InAnnotation(..)
            | Self::OutAnnotation(..) => None,
        }
    }

    fn rename_defs(&mut self, f: &mut dyn FnMut(&str) -> String) {
        match self {
            Self::ConstDecl { lhs, .. } | Self::VarDecl { lhs, .. } 
            | Self::RefLoad { dest: lhs, ..} | Self::Op { dest: lhs, ..} |
            Self::Phi { dest: lhs, ..} => {
                *lhs = f(lhs);
            }
            Self::Hole(..)
            | Self::RefStore { .. }
            | Self::InAnnotation(..)
            | Self::OutAnnotation(..) => (),
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
            Self::Phi { .. } => {
                // don't rename uses of phi nodes

            },
            Self::InAnnotation(_, annots) | Self::OutAnnotation(_, annots) => {
                for (name, _) in annots {
                    *name = f(name, UseType::Read);
                }
            }
            Self::Hole(..)
            | Self::VarDecl { rhs: None, .. } => (),
        }
    }
}

/// Convert a list of `SchedStmts` to a list of Hirs
#[allow(clippy::module_name_repetitions)]
pub fn stmts_to_hir(stmts: Vec<SchedStmt>, specs: &Rc<Specs>) -> Vec<HirBody> {
    stmts.into_iter().map(|s| HirBody::new(s, specs)).collect()
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
