#![allow(clippy::module_name_repetitions)]
use std::collections::{BTreeSet, HashMap};

use crate::{
    enum_cast, lower::{lower_schedule::tag_to_tag, tuple_id}, parse::ast::{Binop, DataType, EncodedCommand, FullType, NestedExpr, SchedExpr, SchedFuncCall, SpecTerm, SpecType, Tag, Tags, TemplateArgs, TimelineOperation, Uop}
};
use caiman::assembly::ast as asm;

use crate::{
    error::Info,
    parse::ast::{Name, SchedStmt, SchedTerm},
};

/// A tag in the HIR with the value, spatial, and timeline information separated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TripleTag {
    pub value: Tag,
    pub spatial: Tag,
    pub timeline: Tag,
}

impl TripleTag {
    pub const fn new_unspecified() -> Self {
        Self {
            value: Tag::new_unspecified(SpecType::Value),
            spatial: Tag::new_unspecified(SpecType::Spatial),
            timeline: Tag::new_unspecified(SpecType::Timeline),
        }
    }
    pub fn from_opt(tags: &Option<Tags>) -> Self {
        tags.as_ref().map_or_else(|| Self::from_owned_opt(None), |tags| Self::from_tags(tags))
    }

    pub fn from_owned_opt(tags: Option<Tags>) -> Self {
        tags.map_or_else(|| Self {
                value: Tag::new_unspecified(SpecType::Value),
                spatial: Tag::new_unspecified(SpecType::Spatial),
                timeline: Tag::new_unspecified(SpecType::Timeline),
            }, Self::from_tag_vec)
    }

    pub fn from_tag_vec(tags: Vec<Tag>) -> Self {
        let mut value = None;
        let mut spatial = None;
        let mut timeline = None;
        for tag in tags {
                match tag.quot_var.spec_type {
                    SpecType::Value => value = Some(tag.clone()),
                    SpecType::Spatial => spatial = Some(tag.clone()),
                    SpecType::Timeline => timeline = Some(tag.clone()),
                }     
        }
        Self {
            value: value.unwrap_or_else(|| Tag::new_unspecified(SpecType::Value)),
            spatial: spatial.unwrap_or_else(|| Tag::new_unspecified(SpecType::Spatial)),
            timeline: timeline.unwrap_or_else(|| Tag::new_unspecified(SpecType::Timeline),)
        }
    }

    pub fn from_tags(tags: &[Tag]) -> Self {
        let mut value = None;
        let mut spatial = None;
        let mut timeline = None;
        for tag in tags {
                match tag.quot_var.spec_type {
                    SpecType::Value => value = Some(tag.clone()),
                    SpecType::Spatial => spatial = Some(tag.clone()),
                    SpecType::Timeline => timeline = Some(tag.clone()),
                }
        }
        Self {
            value: value.unwrap_or_else(|| Tag::new_unspecified(SpecType::Value)),
            spatial: spatial.unwrap_or_else(|| Tag::new_unspecified(SpecType::Spatial)),
            timeline: timeline.unwrap_or_else(|| Tag::new_unspecified(SpecType::Timeline)),
        }
    }

    pub fn from_fulltype(ft: &FullType) -> Self {
        Self::from_tags(&ft.tags)
    }

    pub fn from_fulltype_opt(ft: &Option<FullType>) -> Self {
        ft.as_ref().map_or_else(|| Self::from_owned_opt(None), Self::from_fulltype)
    }

    /// Updates the tag so that all non-null parts of `other` are added to `self`
    pub fn set_specified_info(&mut self, other: Self) {
        self.value.set_specified_info(other.value);
        self.spatial.set_specified_info(other.spatial);
        self.timeline.set_specified_info(other.timeline);
    }

    /// Updates the tag so that all unknown parts of `self` are overridden by `other`
    pub fn override_unknown_info(&mut self, other: Self) {
        self.value.override_unknown_info(other.value);
        self.spatial.override_unknown_info(other.spatial);
        self.timeline.override_unknown_info(other.timeline);
    }

    /// Asserts that the given tag has no holes in its quotient or flow
    fn assert_tag_no_hole(tag: &Tag) {
        assert!(tag.quot.is_some(), "Tag must have a quotient");
        assert!(tag.flow.is_some(), "Tag must have a flow");
    }

    /// Asserts that the triple tag has no holes
    fn assert_no_holes(&self) {
        Self::assert_tag_no_hole(&self.value);
        Self::assert_tag_no_hole(&self.spatial);
        Self::assert_tag_no_hole(&self.timeline);
    }

    /// Converts a triple tag into an assembly tag vector, using
    /// the given data type to determine the default flow for the spatial tag.
    pub fn tags_vec(&self) -> Vec<asm::Tag> {
        self.assert_no_holes();
        vec![
            tag_to_tag(&self.value),
            tag_to_tag(&self.spatial),
            tag_to_tag(&self.timeline),
        ]
    }

}


impl From<TripleTag> for Tags {
    fn from(val: TripleTag) -> Self {
        vec![val.value, val.spatial, val.timeline]
    }
}

impl From<TripleTag> for Option<Tags> {
    fn from(val: TripleTag) -> Self {
        Some(Tags::from(val))
    }
}

#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub enum DataMovement {
    HostToDevice,
}

#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub enum FenceOp {
    Submit,
    Sync
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
    /// Data movement from pointers across different memory spaces
    /// Does not create a new variable, but rather moves the data
    /// from one variable to another
    DeviceCopy {
        info: Info,
        dest: Name,
        src: Name,
        dir: DataMovement,
        encoder: Name,
    },
    /// Begin encoding for a device, definining all of `device_vars`
    BeginEncoding {
        info: Info,
        device: String,
        device_vars: Vec<(Name, TripleTag)>,
        tags: TripleTag,
        encoder: String,
        /// The active fences that haven't been consumed yet at the time of the encoding
        /// Filled via analysis
        active_fences: Vec<String>,
    },
    /// Invoke a function on the device, storing the results into the dest
    /// pointers
    EncodeDo {
        info: Info,
        dests: Vec<Name>,
        func: HirFuncCall,
        encoder: Name,
    },
    FenceOp {
        info: Info,
        dest: Option<Name>,
        op: FenceOp,
        src: SchedTerm,
        tags: TripleTag,
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
    /// External operation (performs a const decl for the destinations)
    Op {
        info: Info,
        dests: Vec<(Name, TripleTag)>,
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

/// The type of an FFI external operation. 
#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub enum OpType {
    /// An external operation for a built-in binary operation
    Binary,
    /// An external operation for a built-in unary operation
    Unary,
    /// An external operation for a user-supplied external function
    External
}

/// A high level IR external operation.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[allow(clippy::upper_case_acronyms)]
pub enum HirOp {
    /// an unlowered binary operation
    Binary(Binop),
    /// an unlowered unary operation
    Unary(Uop),
    /// a lowered operation into an external call
    FFI(Name, OpType),
}

impl HirOp {
    /// Lowers a HIR operation into the name of the external function to call.
    /// Panics if the operation is not lowered.
    pub fn lower(&self) -> Name {
        match self {
            Self::Binary(_) | Self::Unary(_) => panic!("Cannot lower unlowered operation"),
            Self::FFI(name, _) => name.clone(),
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
    pub fn new(value: SchedFuncCall) ->Self {
        if let NestedExpr::Term(SchedTerm::Var { name, .. }) = *value.target {
            let mut starting_args = vec![];
            match value.templates {
                Some(TemplateArgs::Vals(vs)) => {
                    for v in vs {
                        if let NestedExpr::Term(SpecTerm::Var { name, .. }) = v {
                            starting_args.push(name);
                        } else {
                            panic!("Invalid template argument {v:?}")
                        }
                    }
                },
                Some(TemplateArgs::Type(_)) => todo!(),
                None => (),

            }
            let args = starting_args.into_iter().chain(value.args
                .into_iter()
                .map(|a| {
                    enum_cast!(
                        SchedTerm::Var { name, .. },
                        name,
                        enum_cast!(SchedExpr::Term, a)
                    )
                }))
                .collect();
            return Self {
                target: name,
                args,
                tag: Self::to_val_tuple_tag(TripleTag::from_opt(&value.tag)),
            };
        }
        panic!("Invalid internal function call")
    }

    fn to_val_tuple_tag(mut tag: TripleTag) -> TripleTag {
        if let Some(val) = tag.value.quot_var.spec_var.as_mut() {
            *val = tuple_id(&[val.clone()]);
        }
        tag
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
        /// The destination names and tags for the return values in the **parent** scope
        dests: Vec<(String, TripleTag)>,
        /// The returned variables in the child scope
        rets: Vec<String>,
        /// The variables that aren't directly returned by the user but are
        /// captured by the select
        passthrough: Vec<String>,
    },
    /// The final return statement in the final basic block. This is **NOT**
    /// a return statement in the frontend, but rather a special return statement
    /// for final block in the canonical CFG. Takes an argument which is
    /// the names of the return values. Essentially returns `_out0` to `_out{n-1}`
    FinalReturn(Vec<String>),
    /// No terminator, continue to the next block. A `None` terminator is just
    /// a temporary value until live vars and tag analysis can be done to know
    /// what the output variables are for the `Next` terminator
    None,
    /// No terminator, continue to next block with the specified returns
    Next(Vec<String>),
    /// A yield which will capture its arguments to pass them to the
    /// continuation
    Yield(Vec<String>),
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
            Self::FinalReturn(_) | Self::Select { .. } | Self::None | Self::Next(..) | Self::Yield(_) => None,
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
            Self::Return { rets, passthrough, ..}  => {
                for node in rets.iter().chain(passthrough.iter()) {
                    uses.insert(node.clone());
                }
            }
            Self::FinalReturn(names) | Self::Next(names) | Self::Yield(names)=> {
                uses.extend(names.iter().cloned());
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
            Self::Next(rets) | Self::FinalReturn(rets) => {
                for node in rets {
                    *node = f(node, UseType::Read);
                }
            }
            Self::Yield(names) => {
                for name in names.iter_mut() {
                    *name = f(name, UseType::Read);
                }
            }
            Self::Return { rets, passthrough, .. } => {
                for node in rets.iter_mut().chain(passthrough.iter_mut()) {
                    *node = f(node, UseType::Read);
                }
            }
            Self::None => (),
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
            Self::FinalReturn(_) | Self::Select { .. } | Self::None | Self::Next(..) | Self::Yield(_) => (),
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
        match stmt {
            SchedStmt::Assign {
                info,
                lhs,
                rhs,
                ..
            } => {   
                if let SchedExpr::Term(SchedTerm::Var { name, tag, ..}) = lhs {
                        let rhs = enum_cast!(SchedExpr::Term, rhs);
                    Self::RefStore {
                        info,
                        lhs_tags: TripleTag::from_opt(&tag),
                        lhs: name,
                        rhs,
                    }
                } else {
                    panic!("Invalid assignment")
                }       
            },
            SchedStmt::Decl {
                info,
                lhs,
                expr: Some(expr),
                is_const: true,
            } => Self::from_const_decl(expr, info, lhs),
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
                    lhs_tag: TripleTag::from_fulltype_opt(&lhs[0].1),
                    rhs,
                }
            }
            SchedStmt::Encode { info, stmt, encoder, cmd, .. } => {
                // TODO: encoder scoping
                match cmd {
                    EncodedCommand::Copy => {
                        assert_eq!(stmt.lhs.len(), 1);  
                        Self::DeviceCopy { info, dest: stmt.lhs[0].clone(), 
                            src: enum_cast!(SchedTerm::Var {name, ..}, name, enum_cast!(SchedExpr::Term, stmt.rhs)), 
                            dir: DataMovement::HostToDevice, encoder }
                    },
                    EncodedCommand::Invoke => {
                        let dests = stmt.lhs.into_iter().collect();
                        if let SchedTerm::Call(info, call) = enum_cast!(SchedExpr::Term, stmt.rhs) {
                            let func = HirFuncCall::new(call);
                            Self::EncodeDo { info, dests, func, encoder}
                        } else {
                            panic!("Invalid encode")
                        }
                    },
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
            SchedStmt::InEdgeAnnotation { info, tags } => Self::InAnnotation(info, tags.into_iter().map(|(name, tags)| (name, TripleTag::from_tag_vec(tags))).collect()),
            SchedStmt::OutEdgeAnnotation { info, tags } => Self::OutAnnotation(info, tags.into_iter().map(|(name, tags)| (name, TripleTag::from_tag_vec(tags))).collect()),
        }
    }

    /// Converts a tag into a timeline tag, setting the timeline to the tuple id of the
    /// existing timeline variable if it exists.
    fn to_tmln_tuple_tag(mut tag: TripleTag) -> TripleTag {
        if let Some(val) = tag.timeline.quot_var.spec_var.as_mut() {
            *val = tuple_id(&[val.clone()]);
        }
        tag
    }

    /// Constructs a new `HirBody` from a constant declaration.
    fn from_const_decl(expr: SchedExpr, info: Info, lhs: Vec<(String, Option<FullType>)>) -> Self {
        match expr {
            SchedExpr::Term(rhs) => {
                match rhs {
                    SchedTerm::Call(info, call) => {
                        let target = enum_cast!(SchedTerm::Var { name, ..}, name, enum_cast!(SchedExpr::Term, &*call.target));
                        Self::Op {
                            info,
                            dests: lhs.into_iter().map(|(name, tags)| (name, TripleTag::from_fulltype_opt(&tags))).collect(),
                            op: HirOp::FFI(target.clone(), OpType::External),
                            args: call.args.iter().map(|x| enum_cast!(SchedExpr::Term, x)).cloned().collect(),
                        }
                    },
                    SchedTerm::TimelineOperation { info, op, arg, tag } => {
                        Self::FenceOp { info, dest: Some(lhs[0].0.clone()), 
                            op: if op == TimelineOperation::Submit { FenceOp::Submit } else { FenceOp::Sync }, 
                            src: enum_cast!(SchedExpr::Term, *arg), 
                            tags: TripleTag::from_opt(&tag) }
                        },
                    SchedTerm::EncodeBegin { info, device, tag, defs } => {
                        Self::BeginEncoding {
                            info,
                            device,
                            device_vars: defs.into_iter().map(|(name, tags)| (name, TripleTag::from_fulltype_opt(&tags))).collect(),
                            tags: Self::to_tmln_tuple_tag(TripleTag::from_opt(&tag)),
                            encoder: lhs[0].0.clone(),
                            active_fences: vec![],
                        }
                    },
                    _ => Self::ConstDecl {
                        info,
                        lhs: lhs[0].0.clone(),
                        lhs_tag: TripleTag::from_fulltype_opt(&lhs[0].1),
                        rhs,
                    }
                }
            },
            SchedExpr::Binop { info, op: Binop::Dot, lhs: _, rhs: op_rhs} => {
                assert_eq!(lhs.len(), 1);
                // TODO: encoder scope naming
                Self::ConstDecl {
                    info,
                    lhs: lhs[0].0.clone(),
                    lhs_tag: TripleTag::from_fulltype_opt(&lhs[0].1),
                    rhs: enum_cast!(SchedExpr::Term, op_rhs.as_ref().clone()),
                }
            }
            SchedExpr::Binop {
                info,
                op,
                lhs: op_lhs,
                rhs: op_rhs,
            } => {
                let lhs_term = enum_cast!(SchedExpr::Term, op_lhs.as_ref());
                let rhs_term = enum_cast!(SchedExpr::Term, op_rhs.as_ref());
                assert_eq!(lhs.len(), 1);
                Self::Op {
                    info,
                    dests: lhs.into_iter().map(|(name, tags)| (name, TripleTag::from_fulltype_opt(&tags))).collect(),
                    op: HirOp::Binary(op),
                    args: vec![lhs_term.clone(), rhs_term.clone()],
                }
            },
            SchedExpr::Uop { 
                info, op, expr
            } => {
                let term = enum_cast!(SchedExpr::Term, *expr);
                assert_eq!(lhs.len(), 1);
                Self::Op {
                    info,
                    dests: lhs.into_iter().map(|(name, tags)| (name, TripleTag::from_fulltype_opt(&tags))).collect(),
                    op: HirOp::Unary(op),
                    args: vec![term],
                }
            },
            SchedExpr::Conditional { .. } => panic!("Inline conditonal expresssions not allowed in schedule"),
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
            Self::InAnnotation(..) | Self::OutAnnotation(..) | Self::Hole(..) | Self::BeginEncoding { .. } => (),
            Self::Phi {inputs, ..} => {
                res.extend(inputs.iter().map(|(_, name)| name.clone()));
            }
            Self::EncodeDo { dests, func, encoder, ..} => {
                for arg in &func.args {
                    res.insert(arg.clone());
                }
                for name in dests {
                    res.insert(name.clone());
                }
                res.insert(encoder.clone());
            }
            Self::DeviceCopy { dest, src, encoder, .. } => {
                res.insert(dest.clone());
                res.insert(src.clone());
                res.insert(encoder.clone());
            }
            Self::FenceOp { src, .. } => 
                term_get_uses(src, res),
            
        }
    }

    fn get_defs(&self) -> Option<Vec<String>> {
        match self {
            Self::ConstDecl { lhs,  .. } | Self::VarDecl { lhs, .. } 
            | Self::RefLoad { dest: lhs, ..} | 
            Self::Phi { dest: lhs, ..} => {
                Some(vec![lhs.clone()])
            }
            Self::Op { dests, ..} => {
                Some(dests.iter().map(|(name, _)| name.clone()).collect())
            }
            Self::BeginEncoding { device_vars, encoder, .. } => {
                let mut res = device_vars.iter().map(|(name, _)| name.clone()).collect::<Vec<_>>();
                res.push(encoder.clone());
                Some(res)
                
            }
            Self::FenceOp { dest, ..} => dest.as_ref().map(|d| vec![d.clone()]),
            // TODO: re-evaluate the move instruction.
            // Viewing it as a write to a reference, then it had no defs
            Self::Hole(..)
            // RefStore doesn't have a def bc it's a store to a reference
            | Self::RefStore { .. }
            | Self::InAnnotation(..)
            | Self::OutAnnotation(..) 
            | Self::DeviceCopy { .. }
            | Self::EncodeDo {..} => None,
        }
    }

    fn rename_defs(&mut self, f: &mut dyn FnMut(&str) -> String) {
        match self {
            Self::ConstDecl { lhs, .. } | Self::VarDecl { lhs, .. } 
            | Self::RefLoad { dest: lhs, ..} |
            Self::Phi { dest: lhs, ..} => {
                *lhs = f(lhs);
            }
            Self::Op { dests, ..} => {
                for (name, _) in dests {
                    *name = f(name);
                }
            }
            Self::BeginEncoding { device_vars, encoder, ..  } => {
                for (name, _) in device_vars {
                    *name = f(name);
                }
                *encoder = f(encoder);
            }
            Self::FenceOp { dest, ..} => {
                if let Some(dest) = dest {
                    *dest = f(dest);
                }
            }
            Self::Hole(..)
            | Self::RefStore { .. }
            | Self::InAnnotation(..)
            | Self::OutAnnotation(..) 
            | Self::EncodeDo {..} 
            | Self::DeviceCopy {..}=> (),
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
            Self::DeviceCopy { src, dest, encoder, ..} => {
                *src = f(src, UseType::Read);
                *dest = f(dest, UseType::Write);
                *encoder = f(encoder, UseType::Read);
            }
            Self::EncodeDo { func, encoder, dests, ..} => {
                for arg in &mut func.args {
                    *arg = f(arg, UseType::Read);
                }
                for dest in dests {
                    *dest = f(dest, UseType::Write);
                }
                *encoder = f(encoder, UseType::Read);
            }
            Self::FenceOp { src, ..} => {
                term_rename_uses(src, &mut |name| f(name, UseType::Read));
            }
            Self::Hole(..)
            | Self::VarDecl { rhs: None, .. } 
            | Self::BeginEncoding { .. } => (),
        }
    }
}

/// Convert a list of `SchedStmts` to a list of Hirs
#[allow(clippy::module_name_repetitions)]
pub fn stmts_to_hir(stmts: Vec<SchedStmt>, ) -> Vec<HirBody> {
    stmts.into_iter().map(HirBody::new).collect()
}

/// Get the uses in a `SchedTerm`
fn term_get_uses(t: &SchedTerm, res: &mut BTreeSet<String>) {
    match t {
        SchedTerm::Var { name, .. } => {
            res.insert(name.clone());
        }
        SchedTerm::Hole(..) | SchedTerm::Lit { .. } => (),
        SchedTerm::Call(..) | SchedTerm::TimelineOperation { .. } 
        | SchedTerm::EncodeBegin { .. } => todo!(),
    }
}

/// Renames all uses in a `SchedTerm` using the given function
fn term_rename_uses(t: &mut SchedTerm, f: &mut dyn FnMut(&str) -> String) {
    match t {
        SchedTerm::Var { name, .. } => *name = f(name),
        SchedTerm::Hole(..) | SchedTerm::Lit { .. } => (),
        SchedTerm::Call(..) | SchedTerm::TimelineOperation { .. } |
        SchedTerm::EncodeBegin{ ..} => todo!(),
    }
}
