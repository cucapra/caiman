#![allow(clippy::module_name_repetitions)]
use std::collections::{BTreeSet, HashMap};

pub use crate::lower::op_to_str;
use crate::{
    enum_cast,
    lower::{lower_schedule::tag_to_tag, tuple_id},
    parse::ast::{
        self, hole_or_var, Binop, DataType, EncodedCommand, FullType, NestedExpr, SchedExpr,
        SchedFuncCall, SpecTerm, SpecType, Tag, Tags, TemplateArgs, TimelineOperation, Uop,
    },
};
use caiman::assembly::ast as asm;
use caiman::explication::Hole;

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
    #[must_use]
    pub const fn new_unspecified() -> Self {
        Self {
            value: Tag::new_unspecified(SpecType::Value),
            spatial: Tag::new_unspecified(SpecType::Spatial),
            timeline: Tag::new_unspecified(SpecType::Timeline),
        }
    }
    #[must_use]
    pub const fn new_none_usable() -> Self {
        Self {
            value: Tag::new_none_usable(SpecType::Value),
            spatial: Tag::new_none_usable(SpecType::Spatial),
            timeline: Tag::new_none_usable(SpecType::Timeline),
        }
    }
    #[must_use]
    pub fn from_opt(tags: &Option<Tags>) -> Self {
        tags.as_ref()
            .map_or_else(|| Self::from_owned_opt(None), |tags| Self::from_tags(tags))
    }

    pub fn from_owned_opt(tags: Option<Tags>) -> Self {
        tags.map_or_else(
            || Self {
                value: Tag::new_unspecified(SpecType::Value),
                spatial: Tag::new_unspecified(SpecType::Spatial),
                timeline: Tag::new_unspecified(SpecType::Timeline),
            },
            Self::from_tag_vec,
        )
    }

    #[must_use]
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
            timeline: timeline.unwrap_or_else(|| Tag::new_unspecified(SpecType::Timeline)),
        }
    }

    #[must_use]
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

    #[must_use]
    pub fn from_fulltype(ft: &FullType) -> Self {
        Self::from_tags(&ft.tags)
    }

    pub fn from_fulltype_opt(ft: &Option<FullType>) -> Self {
        ft.as_ref()
            .map_or_else(|| Self::from_owned_opt(None), Self::from_fulltype)
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

    /// Converts a triple tag into an assembly tag vector.
    #[must_use]
    pub fn tags_vec(&self) -> Vec<asm::Tag> {
        self.assert_no_holes();
        vec![
            tag_to_tag(&self.value),
            tag_to_tag(&self.spatial),
            tag_to_tag(&self.timeline),
        ]
    }

    /// Resets all specs to be unspecified except for the specs in `e`
    #[must_use]
    pub fn retain(mut self, e: &[SpecType]) -> Self {
        const SPECS: [SpecType; 3] = [SpecType::Value, SpecType::Timeline, SpecType::Spatial];
        for s in SPECS {
            if !e.contains(&s) {
                match s {
                    SpecType::Spatial => self.spatial = Tag::new_unspecified(SpecType::Spatial),
                    SpecType::Timeline => self.timeline = Tag::new_unspecified(SpecType::Timeline),
                    SpecType::Value => self.value = Tag::new_unspecified(SpecType::Value),
                }
            }
        }
        self
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

/// A type that represents information that changes due to an analysis pass.
/// # Parameters
/// - `T`: The initial type
/// - `U`: The type after the analysis pass
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FillIn<T: std::fmt::Debug, U: std::fmt::Debug> {
    Initial(T),
    Processed(U),
}

impl<T: std::fmt::Debug, U: std::fmt::Debug> FillIn<T, U> {
    pub fn processed(&self) -> &U {
        match self {
            Self::Processed(u) => u,
            Self::Initial(_) => panic!("Unprocessed value"),
        }
    }

    pub fn processed_mut(&mut self) -> &mut U {
        match self {
            Self::Processed(u) => u,
            Self::Initial(_) => panic!("Unprocessed value"),
        }
    }

    pub fn process(&mut self, f: impl FnOnce(&mut T) -> U) {
        match self {
            Self::Initial(t) => *self = Self::Processed(f(t)),
            Self::Processed(_) => (),
        }
    }

    pub const fn is_initial(&self) -> bool {
        matches!(self, Self::Initial(..))
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
    /// A data movement into a mutable variable
    RefStore {
        info: Info,
        lhs_tags: TripleTag,
        lhs: Name,
        rhs: HirTerm,
    },
    /// Load from a reference into a new variable
    RefLoad {
        info: Info,
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
        dest_tag: TripleTag,
        src: HirTerm,
        dir: DataMovement,
        encoder: Name,
    },
    /// Begin encoding for a device, definining all of `device_vars`
    BeginEncoding {
        info: Info,
        device: String,
        device_vars: Vec<(Name, TripleTag)>,
        tags: TripleTag,
        encoder: (String, TripleTag),
        /// The active fences that haven't been consumed yet at the time of the encoding
        /// Filled via analysis
        active_fences: Vec<String>,
    },
    /// Invoke a function on the device, storing the results into the dest
    /// pointers
    EncodeDo {
        info: Info,
        dests: Vec<(Name, TripleTag)>,
        func: HirFuncCall,
        encoder: Name,
    },
    Submit {
        info: Info,
        dest: Name,
        src: Name,
        // only one tag since a submit does not require an extraction
        tags: TripleTag,
    },
    /// A sync-fence folowed by copying all the encoded variables in `src` to
    /// local ones in `dests`.
    Sync {
        info: Info,
        /// the local versions of the encoded variables or the name of the record
        /// destination.
        dests: FillIn<(Name, TripleTag), Vec<(Name, TripleTag)>>,
        /// fence name or fence followed by all the variables being copied to the local device
        srcs: FillIn<Name, Vec<Name>>,
        // sync does not require an extraction
        tags: TripleTag,
    },
    /// Declaration of an immutable variable
    ConstDecl {
        info: Info,
        lhs: Name,
        lhs_tag: TripleTag,
        rhs: HirTerm,
    },
    /// Declaration of a mutable variable (reference)
    VarDecl {
        info: Info,
        lhs: Name,
        lhs_tag: TripleTag,
        rhs: Option<HirTerm>,
    },
    /// A hole, which may have one or more definitions if a hole is on the RHS
    /// of a declaration.
    Hole {
        dests: Vec<(String, TripleTag)>,
        info: Info,
        /// all the variables that the hole might use. Filled in by reaching defs
        uses: FillIn<(), Vec<String>>,
        /// variables that must be initialized (made usable) at this hole, and the
        /// value node they will be initialized to.
        initialized: HashMap<String, Option<String>>,
        /// the fences that are active at the current point
        active_fences: Vec<String>,
    },
    /// External pure operation (performs a const decl for the destinations)
    Op {
        info: Info,
        dests: Vec<(Name, TripleTag)>,
        op: HirOp,
        args: Vec<HirTerm>,
    },
    InAnnotation(Info, Vec<(String, TripleTag)>),
    OutAnnotation(Info, Vec<(String, TripleTag)>),
    Phi {
        info: Info,
        dest: Name,
        /// Map from incoming block id to the incoming variable name
        /// from that block
        inputs: HashMap<usize, Name>,
        /// original name of the variable
        original: Name,
    },
}

/// An atomic term in the HIR that can be used as an argument for
/// other HIR instructions
#[derive(Debug)]
pub enum HirTerm {
    Lit {
        info: Info,
        lit: ast::SchedLiteral,
        tag: TripleTag,
    },
    Var {
        info: Info,
        name: Name,
        tag: TripleTag,
    },
    Hole {
        #[allow(unused)]
        info: Info,
        uses: FillIn<(), Vec<String>>,
    },
}

impl HirTerm {
    /// Gets hole containing the variable name if this term is a hole or variable,
    /// otherwise returns `None` if this term is a literal
    pub const fn hole_or_var(&self) -> Option<Hole<&String>> {
        match self {
            Self::Hole { .. } => Some(Hole::Empty),
            Self::Var { name, .. } => Some(Hole::Filled(name)),
            Self::Lit { .. } => None,
        }
    }

    /// Gets the tags of this term, or `None` if it's a hole
    pub const fn get_tags(&self) -> Option<&TripleTag> {
        match self {
            Self::Lit { tag, .. } | Self::Var { tag, .. } => Some(tag),
            Self::Hole { .. } => None,
        }
    }

    /// If this term is a hole, fills the uses of the hole by the result of calling
    /// the given function
    pub fn fill_uses(&mut self, f: impl Fn() -> Vec<String>) {
        if let Self::Hole { uses, .. } = self {
            assert!(uses.is_initial());
            uses.process(|()| f());
        }
    }

    /// Gets the uses of the term
    pub fn get_uses(&self) -> Vec<String> {
        match self {
            Self::Hole { uses, .. } => uses.processed().clone(),
            Self::Var { name, .. } => vec![name.clone()],
            Self::Lit { .. } => vec![],
        }
    }

    /// Renames the uses of this term, with the provided use type
    pub fn rename_uses(&mut self, ut: UseType, f: &mut dyn FnMut(&str, UseType) -> String) {
        match self {
            Self::Hole { uses, .. } => {
                for u in uses.processed_mut() {
                    *u = f(u, ut);
                }
            }
            Self::Var { name, .. } => *name = f(name, ut),
            Self::Lit { .. } => (),
        }
    }
}

impl TryFrom<SchedTerm> for HirTerm {
    type Error = ();

    fn try_from(value: SchedTerm) -> Result<Self, Self::Error> {
        match value {
            SchedTerm::Lit { info, lit, tag } => Ok(Self::Lit {
                info,
                lit,
                tag: TripleTag::from_opt(&tag),
            }),
            SchedTerm::Var { info, name, tag } => Ok(Self::Var {
                info,
                name,
                tag: TripleTag::from_opt(&tag),
            }),
            SchedTerm::Hole { info, .. } => Ok(Self::Hole {
                info,
                uses: FillIn::Initial(()),
            }),
            _ => Err(()),
        }
    }
}

/// A high level IR external operation.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[allow(clippy::upper_case_acronyms)]
pub enum HirOp {
    /// an unlowered binary operation or a pair of a binop name and vector of
    /// argument type names
    Binary(FillIn<Binop, (Name, Vec<Option<String>>)>),
    /// an unlowered unary operation or a pair of a uop name and argument type name
    Unary(FillIn<Uop, (Name, Option<String>)>),
    /// an external call
    External(Name),
}

impl HirOp {
    /// Lowers a HIR operation into the name of the external function to call.
    /// Panics if the operation is not lowered.
    pub fn lower(&self) -> Hole<Name> {
        match self {
            Self::Binary(name) => {
                let (basename, arg_types) = name.processed();
                if arg_types.iter().any(std::option::Option::is_none) {
                    // some arg type is unknown
                    Hole::Empty
                } else {
                    Hole::Filled(op_to_str(
                        basename,
                        arg_types.iter().map(|x| x.as_ref().unwrap()),
                    ))
                }
            }
            Self::Unary(name) => {
                let (basename, arg_type) = name.processed();
                arg_type.as_ref().map_or(Hole::Empty, |arg_type| {
                    Hole::Filled(op_to_str(basename, std::iter::once(arg_type)))
                })
            }
            Self::External(name) => Hole::Filled(name.clone()),
        }
    }
}

/// An internal function call in the high-level IR.
#[derive(Clone, Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct HirFuncCall {
    pub info: Info,
    pub target: String,
    pub args: Vec<Hole<String>>,
    pub tag: TripleTag,
    /// The number of value template arguments that occur in `args` before the
    /// normal arguments
    pub num_dims: usize,
    /// If any arg is a hole, these are the extra uses of the function call due to
    /// the hole
    pub extra_uses: Option<FillIn<(), Vec<String>>>,
}

impl HirFuncCall {
    pub fn new(value: SchedFuncCall) -> Self {
        if let NestedExpr::Term(SchedTerm::Var { name, .. }) = *value.target {
            let mut starting_args = vec![];
            let num_dims = value.templates.as_ref().map_or(0, |t| {
                if let TemplateArgs::Vals(v) = t {
                    v.len()
                } else {
                    0
                }
            });
            match value.templates {
                Some(TemplateArgs::Vals(vs)) => {
                    for v in vs {
                        if let NestedExpr::Term(SpecTerm::Var { name, .. }) = v {
                            starting_args.push(Hole::Filled(name));
                        } else {
                            panic!("Invalid template argument {v:?}")
                        }
                    }
                }
                Some(TemplateArgs::Type(_)) => unimplemented!("Type template arguments"),
                None => (),
            }
            let args: Vec<_> = starting_args
                .into_iter()
                .chain(
                    value
                        .args
                        .into_iter()
                        .map(|a| hole_or_var(&a).unwrap().cloned()),
                )
                .collect();
            return Self {
                info: value.info,
                extra_uses: if args.iter().any(Hole::is_empty) {
                    Some(FillIn::Initial(()))
                } else {
                    None
                },
                target: name,
                args,
                tag: Self::to_tuple_tag(TripleTag::from_opt(&value.tag)),
                num_dims,
            };
        }
        panic!("Invalid internal function call")
    }

    fn to_tuple_tag(mut tag: TripleTag) -> TripleTag {
        if let Some(val) = tag.value.quot_var.spec_var.as_mut() {
            *val = tuple_id(&[val.clone()]);
        }
        if let Some(sptl) = tag.spatial.quot_var.spec_var.as_mut() {
            *sptl = tuple_id(&[sptl.clone()]);
        }
        if let Some(tmln) = tag.timeline.quot_var.spec_var.as_mut() {
            *tmln = tuple_id(&[tmln.clone()]);
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
        info: Info,
        dests: Vec<(String, TripleTag)>,
        guard: Hole<String>,
        tag: TripleTag,
        /// Extra uses if the guard is a hole.
        extra_uses: FillIn<(), Vec<String>>,
    },
    /// A return statement which returns values to the parent scope, **NOT** out
    /// of the function. This is modeled as an assignment to the destination variables.
    /// For returning from the function, the destination variables are
    /// `_out0`, `_out1`, etc.
    Return {
        info: Info,
        /// The destination names and tags for the return values in the **parent** scope.
        /// Note: this is the only case where Hir may not be in SSA form
        dests: Vec<(String, TripleTag)>,
        /// The returned variables in the child scope
        rets: Vec<Hole<String>>,
        /// The variables that aren't directly returned by the user but are
        /// captured by the select
        passthrough: Vec<String>,
        /// Extra uses if any returned variable is a hole
        extra_uses: FillIn<(), Vec<String>>,
    },
    /// The final return statement in the final basic block. This is **NOT**
    /// a return statement in the frontend, but rather a special return statement
    /// for final block in the canonical CFG. Takes an argument which is
    /// the names of the return values. Essentially returns `_out0` to `_out{n-1}`
    FinalReturn(Info, Vec<String>),
    /// No terminator, continue to the next block. A `None` terminator is just
    /// a temporary value until live vars and tag analysis can be done to know
    /// what the output variables are for the `Next` terminator
    None(Info),
    /// No terminator, continue to next block with the specified returns
    Next(Info, Vec<String>),
    /// A yield which will capture its arguments to pass them to the
    /// continuation
    Yield(Info, Vec<String>),
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

    /// Get the variables that are written to by this statement.
    /// These are uses that are written to.
    fn get_write_uses(&self) -> Option<Vec<String>>;

    /// Renames all uses in this statement using the given function which is
    /// passed the name of the variable and the type of use.
    fn rename_uses(&mut self, f: &mut dyn FnMut(&str, UseType) -> String);

    /// Renames all defs in this statement using the given function which is
    /// passed the name of the variable.
    fn rename_defs(&mut self, f: &mut dyn FnMut(&str) -> String);

    fn get_info(&self) -> Info;
}

impl Hir for Terminator {
    fn get_info(&self) -> Info {
        match self {
            Self::Call(_, call) | Self::CaptureCall { call, .. } => call.info,
            Self::Select { info, .. }
            | Self::Return { info, .. }
            | Self::FinalReturn(info, ..)
            | Self::None(info)
            | Self::Next(info, ..)
            | Self::Yield(info, ..) => *info,
        }
    }
    fn get_defs(&self) -> Option<Vec<String>> {
        match self {
            Self::Call(defs, ..)
            | Self::CaptureCall { dests: defs, .. }
            | Self::Return { dests: defs, .. } => {
                Some(defs.iter().map(|(d, _)| d.clone()).collect())
            }
            // we don't consider the defs of a select to be defs of this terminator,
            // but rather they are the defs of the left and right funclets
            Self::FinalReturn(..)
            | Self::Select { .. }
            | Self::None(..)
            | Self::Next(..)
            | Self::Yield(..) => None,
        }
    }
    fn get_write_uses(&self) -> Option<Vec<String>> {
        None
    }

    fn get_uses(&self, uses: &mut BTreeSet<String>) {
        match self {
            Self::Call(_, call) | Self::CaptureCall { call, .. } => {
                for arg in &call.args {
                    if let Hole::Filled(arg) = arg {
                        uses.insert(arg.clone());
                    }
                }
                if let Some(extras) = &call.extra_uses {
                    if !extras.is_initial() {
                        uses.extend(extras.processed().iter().cloned());
                    }
                }
            }
            Self::Select {
                guard, extra_uses, ..
            } => {
                if let Hole::Filled(guard) = guard {
                    uses.insert(guard.clone());
                }
                if !extra_uses.is_initial() {
                    uses.extend(extra_uses.processed().iter().cloned());
                }
            }
            Self::Return {
                rets,
                passthrough,
                extra_uses,
                ..
            } => {
                for node in rets
                    .iter()
                    .filter_map(|x| x.as_ref().opt())
                    .chain(passthrough.iter())
                {
                    uses.insert(node.clone());
                }
                if !extra_uses.is_initial() {
                    uses.extend(extra_uses.processed().iter().cloned());
                }
            }
            Self::FinalReturn(_, names) | Self::Next(_, names) | Self::Yield(_, names) => {
                uses.extend(names.iter().cloned());
            }
            Self::None(..) => (),
        }
    }

    fn rename_uses(&mut self, f: &mut dyn FnMut(&str, UseType) -> String) {
        match self {
            Self::Call(_, call) | Self::CaptureCall { call, .. } => {
                for arg in &mut call.args {
                    if let Hole::Filled(arg) = arg {
                        *arg = f(arg, UseType::Read);
                    }
                }
                if let Some(extras) = &mut call.extra_uses {
                    if !extras.is_initial() {
                        for u in extras.processed_mut() {
                            *u = f(u, UseType::Read);
                        }
                    }
                }
            }
            Self::Select {
                guard, extra_uses, ..
            } => {
                if let Hole::Filled(guard) = guard {
                    *guard = f(guard, UseType::Read);
                }
                if !extra_uses.is_initial() {
                    for u in extra_uses.processed_mut() {
                        *u = f(u, UseType::Read);
                    }
                }
            }
            Self::Next(_, rets) | Self::FinalReturn(_, rets) => {
                for node in rets {
                    *node = f(node, UseType::Read);
                }
            }
            Self::Yield(_, names) => {
                for name in names.iter_mut() {
                    *name = f(name, UseType::Read);
                }
            }
            Self::Return {
                rets,
                passthrough,
                extra_uses,
                ..
            } => {
                for node in rets
                    .iter_mut()
                    .filter_map(|x| x.as_mut().opt())
                    .chain(passthrough.iter_mut())
                {
                    *node = f(node, UseType::Read);
                }
                if !extra_uses.is_initial() {
                    for u in extra_uses.processed_mut() {
                        *u = f(u, UseType::Read);
                    }
                }
            }
            Self::None(..) => (),
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
            Self::FinalReturn(..)
            | Self::Select { .. }
            | Self::None(..)
            | Self::Next(..)
            | Self::Yield(..) => (),
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
#[derive(Debug)]
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
    #[allow(clippy::too_many_lines)]
    pub fn new(stmt: SchedStmt) -> Self {
        match stmt {
            SchedStmt::Assign { info, lhs, rhs, .. } => {
                if let SchedExpr::Term(SchedTerm::Var { name, tag, .. }) = lhs {
                    let rhs = enum_cast!(SchedExpr::Term, rhs);
                    Self::RefStore {
                        info,
                        lhs_tags: TripleTag::from_opt(&tag),
                        lhs: name,
                        rhs: TryFrom::try_from(rhs).unwrap(),
                    }
                } else {
                    panic!("Invalid assignment")
                }
            }
            SchedStmt::Decl {
                info,
                lhs,
                is_const,
                expr:
                    Some(SchedExpr::Term(SchedTerm::Hole {
                        can_generate_code: true,
                        ..
                    })),
            } => {
                assert!(
                    is_const,
                    "Variable decl with holes is currently unsupported"
                );
                Self::Hole {
                    dests: lhs
                        .into_iter()
                        .map(|(nm, typ)| (nm, TripleTag::from_fulltype_opt(&typ)))
                        .collect(),
                    info,
                    uses: FillIn::Initial(()),
                    initialized: HashMap::new(),
                    active_fences: vec![],
                }
            }
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
                    rhs: rhs.map(|x| HirTerm::try_from(x).unwrap()),
                }
            }
            SchedStmt::Encode {
                info,
                stmt,
                encoder,
                cmd,
                ..
            } => match cmd {
                EncodedCommand::Copy => {
                    assert_eq!(stmt.lhs.len(), 1);
                    Self::DeviceCopy {
                        info,
                        dest: stmt.lhs[0].0.clone(),
                        dest_tag: TripleTag::from_opt(&stmt.lhs[0].1),
                        src: HirTerm::try_from(enum_cast!(SchedExpr::Term, stmt.rhs)).unwrap(),
                        dir: DataMovement::HostToDevice,
                        encoder,
                    }
                }
                EncodedCommand::Invoke => {
                    let dests = stmt
                        .lhs
                        .into_iter()
                        .map(|(nm, tag)| (nm, TripleTag::from_opt(&tag)))
                        .collect();
                    if let SchedTerm::Call(info, call) = enum_cast!(SchedExpr::Term, stmt.rhs) {
                        let func = HirFuncCall::new(call);
                        Self::EncodeDo {
                            info,
                            dests,
                            func,
                            encoder,
                        }
                    } else {
                        panic!("Invalid encode")
                    }
                }
            },
            SchedStmt::Decl { .. } => panic!("Invalid declaration"),
            SchedStmt::Return(..)
            | SchedStmt::Block(..)
            | SchedStmt::If { .. }
            | SchedStmt::Seq { .. }
            | SchedStmt::Call(..) => {
                panic!("Unexpected stmt")
            }
            SchedStmt::Hole(info) => Self::Hole {
                info,
                dests: vec![],
                uses: FillIn::Initial(()),
                initialized: HashMap::new(),
                active_fences: vec![],
            },
            SchedStmt::InEdgeAnnotation { info, tags } => Self::InAnnotation(
                info,
                tags.into_iter()
                    .map(|(name, tags)| (name, TripleTag::from_tag_vec(tags)))
                    .collect(),
            ),
            SchedStmt::OutEdgeAnnotation { info, tags } => Self::OutAnnotation(
                info,
                tags.into_iter()
                    .map(|(name, tags)| (name, TripleTag::from_tag_vec(tags)))
                    .collect(),
            ),
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
    #[allow(clippy::too_many_lines)]
    fn from_const_decl(expr: SchedExpr, info: Info, lhs: Vec<(String, Option<FullType>)>) -> Self {
        match expr {
            SchedExpr::Term(rhs) => match rhs {
                SchedTerm::Call(info, call) => {
                    let target = enum_cast!(
                        SchedTerm::Var { name, .. },
                        name,
                        enum_cast!(SchedExpr::Term, &*call.target)
                    );
                    Self::Op {
                        info,
                        dests: lhs
                            .into_iter()
                            .map(|(name, tags)| (name, TripleTag::from_fulltype_opt(&tags)))
                            .collect(),
                        op: HirOp::External(target.clone()),
                        args: call
                            .args
                            .iter()
                            .map(|x| {
                                HirTerm::try_from(enum_cast!(SchedExpr::Term, x).clone()).unwrap()
                            })
                            .collect(),
                    }
                }
                SchedTerm::TimelineOperation {
                    info,
                    op: TimelineOperation::Submit,
                    arg,
                    tag,
                } => Self::Submit {
                    info,
                    dest: lhs[0].0.clone(),
                    src: enum_cast!(
                        SchedTerm::Var { name, .. },
                        name,
                        enum_cast!(SchedExpr::Term, *arg)
                    ),
                    tags: TripleTag::from_opt(&tag),
                },
                SchedTerm::TimelineOperation {
                    info,
                    op: TimelineOperation::Await,
                    arg,
                    tag,
                } => {
                    let arg_name = enum_cast!(
                        SchedTerm::Var { name, .. },
                        name,
                        enum_cast!(SchedExpr::Term, *arg)
                    );
                    Self::Sync {
                        info,
                        dests: FillIn::Initial((
                            lhs[0].0.clone(),
                            TripleTag::from_fulltype_opt(&lhs[0].1),
                        )),
                        srcs: FillIn::Initial(arg_name),
                        tags: TripleTag::from_opt(&tag),
                    }
                }
                SchedTerm::EncodeBegin {
                    info,
                    device,
                    tag,
                    defs,
                } => Self::BeginEncoding {
                    info,
                    device,
                    device_vars: defs
                        .into_iter()
                        .map(|(name, tags)| (name, TripleTag::from_fulltype_opt(&tags)))
                        .collect(),
                    tags: Self::to_tmln_tuple_tag(TripleTag::from_opt(&tag)),
                    encoder: (lhs[0].0.clone(), TripleTag::from_fulltype_opt(&lhs[0].1)),
                    active_fences: vec![],
                },
                _ => Self::ConstDecl {
                    info,
                    lhs: lhs[0].0.clone(),
                    lhs_tag: TripleTag::from_fulltype_opt(&lhs[0].1),
                    rhs: HirTerm::try_from(rhs).unwrap(),
                },
            },
            SchedExpr::Binop {
                info,
                op: Binop::Dot,
                lhs: op_lhs,
                rhs: op_rhs,
            } if matches!(*op_rhs, SchedExpr::Term(SchedTerm::Hole { .. }))
                || matches!(*op_lhs, SchedExpr::Term(SchedTerm::Hole { .. })) =>
            {
                // TODO: give holes names for typing
                assert_eq!(lhs.len(), 1);
                Self::Hole {
                    dests: vec![(lhs[0].0.clone(), TripleTag::from_fulltype_opt(&lhs[0].1))],
                    info,
                    uses: FillIn::Initial(()),
                    initialized: HashMap::new(),
                    active_fences: vec![],
                }
            }
            SchedExpr::Binop {
                info,
                op: Binop::Dot,
                lhs: op_lhs,
                rhs: op_rhs,
            } => {
                assert_eq!(lhs.len(), 1);
                let op_lhs_name = enum_cast!(
                    SchedTerm::Var { name, .. },
                    name,
                    enum_cast!(SchedExpr::Term, *op_lhs)
                );
                let rhs_name = enum_cast!(
                    SchedTerm::Var { name, .. },
                    name,
                    enum_cast!(SchedExpr::Term, *op_rhs)
                );
                Self::ConstDecl {
                    info,
                    lhs: lhs[0].0.clone(),
                    lhs_tag: TripleTag::from_fulltype_opt(&lhs[0].1),
                    rhs: HirTerm::Var {
                        name: format!("{op_lhs_name}::{rhs_name}"),
                        tag: TripleTag::new_unspecified(),
                        info,
                    },
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
                    dests: lhs
                        .into_iter()
                        .map(|(name, tags)| (name, TripleTag::from_fulltype_opt(&tags)))
                        .collect(),
                    op: HirOp::Binary(FillIn::Initial(op)),
                    args: vec![
                        HirTerm::try_from(lhs_term.clone()).unwrap(),
                        HirTerm::try_from(rhs_term.clone()).unwrap(),
                    ],
                }
            }
            SchedExpr::Uop { info, op, expr } => {
                let term = enum_cast!(SchedExpr::Term, *expr);
                assert_eq!(lhs.len(), 1);
                Self::Op {
                    info,
                    dests: lhs
                        .into_iter()
                        .map(|(name, tags)| (name, TripleTag::from_fulltype_opt(&tags)))
                        .collect(),
                    op: HirOp::Unary(FillIn::Initial(op)),
                    args: vec![HirTerm::try_from(term).unwrap()],
                }
            }
            SchedExpr::Conditional { .. } => {
                panic!("Inline conditonal expresssions not allowed in schedule")
            }
        }
    }
}
impl Hir for HirBody {
    fn get_info(&self) -> Info {
        match self {
            Self::RefStore { info, .. }
            | Self::RefLoad { info, .. }
            | Self::DeviceCopy { info, .. }
            | Self::BeginEncoding { info, .. }
            | Self::EncodeDo { info, .. }
            | Self::Submit { info, .. }
            | Self::ConstDecl { info, .. }
            | Self::VarDecl { info, .. }
            | Self::Hole { info, .. }
            | Self::Op { info, .. }
            | Self::InAnnotation(info, ..)
            | Self::OutAnnotation(info, ..)
            | Self::Phi { info, .. }
            | Self::Sync { info, .. } => *info,
        }
    }
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
            Self::RefLoad { src, .. } | Self::Submit { src, .. } => {
                res.insert(src.clone());
            }
            Self::Op { args, .. } => {
                for arg in args {
                    term_get_uses(arg, res);
                }
            }
            Self::Hole {
                uses, initialized, ..
            } => {
                if !uses.is_initial() {
                    for u in uses.processed() {
                        res.insert(u.clone());
                    }
                }
                for init in initialized.keys() {
                    assert!(!uses.is_initial() && uses.processed().contains(init));
                    res.insert(init.clone());
                }
            }
            Self::InAnnotation(..) | Self::OutAnnotation(..) | Self::BeginEncoding { .. } => (),
            Self::Phi { inputs, .. } => {
                res.extend(inputs.iter().map(|(_, name)| name.clone()));
            }
            Self::EncodeDo {
                dests,
                func,
                encoder,
                ..
            } => {
                for arg in &func.args {
                    if let Hole::Filled(arg) = arg {
                        res.insert(arg.clone());
                    }
                }
                // semantics of encode-do is to store into references
                for (name, _) in dests {
                    res.insert(name.clone());
                }
                if let Some(extras) = &func.extra_uses {
                    res.extend(extras.processed().iter().cloned());
                }
                res.insert(encoder.clone());
            }
            Self::DeviceCopy {
                dest, src, encoder, ..
            } => {
                res.insert(dest.clone());
                res.extend(src.get_uses());
                res.insert(encoder.clone());
            }
            Self::Sync { srcs, .. } => match srcs {
                FillIn::Initial(name) => {
                    res.insert(name.clone());
                }
                FillIn::Processed(srcs) => {
                    res.extend(srcs.iter().cloned());
                }
            },
        }
    }

    fn get_write_uses(&self) -> Option<Vec<String>> {
        match self {
            Self::RefStore { lhs, .. } => Some(vec![lhs.clone()]),
            Self::EncodeDo { dests, .. } => {
                Some(dests.iter().map(|(name, _)| name.clone()).collect())
            }
            Self::DeviceCopy { dest, .. } => Some(vec![dest.clone()]),
            Self::Hole { initialized, .. } => Some(initialized.keys().cloned().collect()),
            Self::ConstDecl { .. }
            | Self::VarDecl { .. }
            | Self::RefLoad { .. }
            | Self::Op { .. }
            | Self::InAnnotation(..)
            | Self::OutAnnotation(..)
            | Self::BeginEncoding { .. }
            | Self::Phi { .. }
            | Self::Submit { .. }
            | Self::Sync { .. } => None,
        }
    }

    fn get_defs(&self) -> Option<Vec<String>> {
        match self {
            Self::ConstDecl { lhs, .. }
            | Self::VarDecl { lhs, .. }
            | Self::RefLoad { dest: lhs, .. }
            | Self::Phi { dest: lhs, .. } => Some(vec![lhs.clone()]),
            Self::Op { dests, .. } | Self::Hole { dests, .. } => {
                Some(dests.iter().map(|(name, _)| name.clone()).collect())
            }
            Self::BeginEncoding {
                device_vars,
                encoder,
                ..
            } => {
                let mut res = device_vars
                    .iter()
                    .map(|(name, _)| name.clone())
                    .collect::<Vec<_>>();
                res.push(encoder.0.clone());
                Some(res)
            }
            Self::Submit { dest, .. } => Some(vec![dest.clone()]),
            Self::Sync { dests, .. } => match dests {
                FillIn::Initial((name, _)) => Some(vec![name.clone()]),
                FillIn::Processed(dests) => {
                    Some(dests.iter().map(|(name, _)| name.clone()).collect())
                }
            },
            // RefStore doesn't have a def bc it's a store to a reference
            Self::RefStore { .. }
            | Self::InAnnotation(..)
            | Self::OutAnnotation(..)
            | Self::DeviceCopy { .. }
            | Self::EncodeDo { .. } => None,
        }
    }

    fn rename_defs(&mut self, f: &mut dyn FnMut(&str) -> String) {
        match self {
            Self::ConstDecl { lhs, .. }
            | Self::VarDecl { lhs, .. }
            | Self::RefLoad { dest: lhs, .. }
            | Self::Phi { dest: lhs, .. } => {
                *lhs = f(lhs);
            }
            Self::Op { dests, .. } | Self::Hole { dests, .. } => {
                for (name, _) in dests {
                    *name = f(name);
                }
            }
            Self::BeginEncoding {
                device_vars,
                encoder,
                ..
            } => {
                for (name, _) in device_vars {
                    *name = f(name);
                }
                *encoder = (f(&encoder.0), encoder.1.clone());
            }
            Self::Submit { dest, .. } => {
                *dest = f(dest);
            }
            Self::Sync { dests, .. } => match dests {
                FillIn::Initial((name, _)) => {
                    *name = f(name);
                }
                FillIn::Processed(dests) => {
                    for (name, _) in dests {
                        *name = f(name);
                    }
                }
            },
            Self::RefStore { .. }
            | Self::InAnnotation(..)
            | Self::OutAnnotation(..)
            | Self::EncodeDo { .. }
            | Self::DeviceCopy { .. } => (),
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
            Self::RefLoad { src, .. } | Self::Submit { src, .. } => {
                *src = f(src, UseType::Read);
            }
            Self::Phi { .. } => {
                // don't rename uses of phi nodes
            }
            Self::InAnnotation(_, annots) | Self::OutAnnotation(_, annots) => {
                for (name, _) in annots {
                    *name = f(name, UseType::Read);
                }
            }
            Self::DeviceCopy {
                src, dest, encoder, ..
            } => {
                src.rename_uses(UseType::Read, f);
                *dest = f(dest, UseType::Write);
                *encoder = f(encoder, UseType::Read);
            }
            Self::EncodeDo {
                func,
                encoder,
                dests,
                ..
            } => {
                for arg in &mut func.args {
                    if let Hole::Filled(arg) = arg {
                        *arg = f(arg, UseType::Read);
                    }
                }
                if let Some(extras) = &mut func.extra_uses {
                    if !extras.is_initial() {
                        for u in extras.processed_mut() {
                            *u = f(u, UseType::Read);
                        }
                    }
                }
                *encoder = f(encoder, UseType::Read);
                for (dest, _) in dests {
                    *dest = f(dest, UseType::Write);
                }
            }
            Self::Sync { srcs, .. } => match srcs {
                FillIn::Initial(src) => {
                    *src = f(src, UseType::Read);
                }
                FillIn::Processed(srcs) => {
                    for src in srcs {
                        *src = f(src, UseType::Read);
                    }
                }
            },
            Self::Hole {
                uses, initialized, ..
            } => {
                if !uses.is_initial() {
                    for u in uses.processed_mut() {
                        *u = f(u, UseType::Read);
                    }
                }
                let mut new_map = HashMap::new();
                for (init, v) in std::mem::take(initialized) {
                    new_map.insert(f(&init, UseType::Write), v);
                }
                *initialized = new_map;
            }
            Self::VarDecl { rhs: None, .. } | Self::BeginEncoding { .. } => (),
        }
    }
}

/// Convert a list of `SchedStmts` to a list of Hirs
#[allow(clippy::module_name_repetitions)]
pub fn stmts_to_hir(stmts: Vec<SchedStmt>) -> Vec<HirBody> {
    stmts.into_iter().map(HirBody::new).collect()
}

/// Get the uses in a `HirTerm`
fn term_get_uses(t: &HirTerm, res: &mut BTreeSet<String>) {
    match t {
        HirTerm::Var { name, .. } => {
            res.insert(name.clone());
        }
        HirTerm::Hole { uses, .. } => {
            if !uses.is_initial() {
                res.extend(uses.processed().iter().cloned());
            }
        }
        HirTerm::Lit { .. } => {}
    }
}

/// Renames all uses in a `HirTerm` using the given function
fn term_rename_uses(t: &mut HirTerm, f: &mut dyn FnMut(&str) -> String) {
    match t {
        HirTerm::Var { name, .. } => *name = f(name),
        HirTerm::Lit { .. } => (),
        HirTerm::Hole { uses, .. } => {
            if !uses.is_initial() {
                for u in uses.processed_mut() {
                    *u = f(u);
                }
            }
        }
    }
}
