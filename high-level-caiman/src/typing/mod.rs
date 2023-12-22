pub mod context;
use std::collections::HashMap;
mod specs;

pub use specs::sched_lit_to_str;

use crate::{
    error::{type_error, Info, LocalError},
    parse::ast::{Binop, DataType},
};
use caiman::assembly::ast as asm;

use self::{
    types::{ADataType, CDataType, DTypeConstraint},
    unification::{Constraint, Env},
};
mod sched;
#[cfg(test)]
mod test;
mod types;
mod unification;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The type of a spec.
pub enum SpecType {
    Value,
    Timeline,
    Spatial,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SpecNode {
    Input(String),
    Output(String),
    Lit(String),
    Op(String, Vec<String>),
    Extract(String, usize),
}

#[derive(Debug, Clone, Default)]
pub struct NodeMap {
    pub nodes: HashMap<String, SpecNode>,
    pub node_names: HashMap<SpecNode, String>,
}

impl NodeMap {
    #[must_use]
    pub fn get_name(&self, node: &SpecNode) -> Option<&String> {
        self.node_names.get(node)
    }

    #[must_use]
    pub fn get_node(&self, name: &str) -> Option<&SpecNode> {
        self.nodes.get(name)
    }

    pub fn insert(&mut self, name: String, node: SpecNode) {
        self.nodes.insert(name.clone(), node.clone());
        self.node_names.insert(node, name);
    }

    /// Inserts a node if its name and value is unique. If the node is already
    /// present, it is removed.
    pub fn insert_or_remove_if_dup(&mut self, name: String, node: SpecNode) {
        if self.contains(&name) || self.contains_node(&node) {
            self.nodes.remove(&name);
            self.node_names.remove(&node);
        } else {
            self.insert(name, node);
        }
    }

    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.nodes.contains_key(name)
    }

    #[must_use]
    pub fn contains_node(&self, node: &SpecNode) -> bool {
        self.node_names.contains_key(node)
    }
}

/// A data type typing environment.
#[derive(Debug, Clone)]
pub struct DTypeEnv {
    env: Env<CDataType, ADataType>,
}

impl Default for DTypeEnv {
    fn default() -> Self {
        Self { env: Env::new() }
    }
}

impl DTypeEnv {
    #[must_use]
    pub fn new() -> Self {
        Self { env: Env::new() }
    }
    /// Adds a constraint that a variable must adhere to a certain type.
    /// # Errors
    /// Returns an error if unification fails.
    pub fn add_constraint(
        &mut self,
        name: &str,
        constraint: DTypeConstraint,
        info: Info,
    ) -> Result<(), LocalError> {
        let c = constraint.instantiate(&mut self.env);
        self.env.add_constraint(name, &c).map_err(|e| {
            type_error(
                info,
                &format!("Failed to unify type constraints of variable {name}"),
            )
        })
    }

    /// Adds a constraint that a variable must have a certain type.
    /// # Errors
    /// Returns an error if unification fails.
    pub fn add_dtype_constraint(
        &mut self,
        name: &str,
        constraint: DataType,
        info: Info,
    ) -> Result<(), LocalError> {
        let constraint = DTypeConstraint::from(constraint);
        self.add_constraint(name, constraint, info)
    }

    /// Adds a constraint that two variables are equivalent.
    /// # Errors
    /// Returns an error if unification fails.
    pub fn add_var_equiv(&mut self, name: &str, equiv: &str, info: Info) -> Result<(), LocalError> {
        self.env
            .add_constraint(name, &Constraint::Var(equiv.to_string()))
            .map_err(|e| {
                type_error(
                    info,
                    &format!("Failed to unify type constraints of variable {name}"),
                )
            })
    }

    /// Adds a constraint that a variable must adhere to.
    /// # Errors
    /// Returns an error if unification fails.
    pub fn add_raw_constraint(
        &mut self,
        name: &str,
        constraint: &Constraint<CDataType, ADataType>,
        info: Info,
    ) -> Result<(), LocalError> {
        self.env.add_constraint(name, constraint).map_err(|e| {
            type_error(
                info,
                &format!("Failed to unify type constraints of variable {name}"),
            )
        })
    }
}

/// Information about a specification.
#[derive(Debug, Clone)]
pub struct SpecInfo {
    /// Type of the spec
    pub typ: SpecType,
    /// Type signature
    pub sig: NamedSignature,
    /// Map from variable name to node and node to variable name.
    // pub nodes: NodeMap,
    /// Map from variable name to type.
    pub types: HashMap<String, DataType>,
    pub info: Info,
}

impl SpecInfo {
    #[must_use]
    pub fn new(typ: SpecType, sig: NamedSignature, info: Info) -> Self {
        Self {
            typ,
            sig,
            // nodes: NodeMap::default(),
            types: HashMap::new(),
            info,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mutability {
    Const,
    Mut,
}

#[derive(Debug, Clone)]
/// Information about a schedule.
pub struct SchedInfo {
    /// Name of the value spec.
    pub value: String,
    /// Name of the timeline spec.
    pub timeline: String,
    /// Name of the spatial spec.
    pub spatial: String,
    /// The type signature of the schedule.
    pub dtype_sig: Signature,
    /// Map from variable name to type.
    pub types: HashMap<String, DataType>,
    /// Set of defined names and mapping from defined name to
    /// whether it is a constant. Non-constants are references.
    pub defined_names: HashMap<String, Mutability>,
}

#[derive(Debug, Clone)]
pub enum SchedOrExtern {
    Sched(SchedInfo),
    Extern(Signature),
}

impl SchedOrExtern {
    #[must_use]
    pub const fn sig(&self) -> &Signature {
        match self {
            Self::Sched(s) => &s.dtype_sig,
            Self::Extern(s) => s,
        }
    }

    /// Unwraps the schedule info.
    /// # Panics
    /// If this is an extern.
    #[must_use]
    pub fn unwrap_sched(&self) -> &SchedInfo {
        match self {
            Self::Sched(s) => s,
            Self::Extern(_) => panic!("Expected schedule, got extern"),
        }
    }

    /// Unwraps the schedule info.
    /// # Panics
    /// If this is an extern.
    #[must_use]
    pub fn unwrap_sched_mut(&mut self) -> &mut SchedInfo {
        match self {
            Self::Sched(s) => s,
            Self::Extern(_) => panic!("Expected schedule, got extern"),
        }
    }
}

impl SchedInfo {
    /// Creates a new spec map from a list of spec names.
    /// # Errors
    /// If any of the spec names are not present in the context or
    /// we don't have a value, timeline, and spatial spec.
    /// # Panics
    /// Internal error
    pub fn new(specs: [String; 3], ctx: &Context, info: &Info) -> Result<Self, LocalError> {
        let mut val = None;
        let mut timeline = None;
        let mut spatial = None;
        for name in specs {
            if !ctx.specs.contains_key(&name) {
                return Err(type_error(*info, &format!("{info}: Spec {name} not found")));
            }
            let typ = ctx.specs.get(&name).unwrap().typ;
            match typ {
                SpecType::Value => {
                    if val.is_some() {
                        return Err(type_error(
                            *info,
                            &format!("{info}: Duplicate value specs {name} and {}", val.unwrap()),
                        ));
                    }
                    val = Some(name);
                }
                SpecType::Timeline => {
                    if timeline.is_some() {
                        return Err(type_error(
                            *info,
                            &format!(
                                "{info}: Duplicate timeline specs {name} and {}",
                                timeline.unwrap()
                            ),
                        ));
                    }
                    timeline = Some(name);
                }
                SpecType::Spatial => {
                    if spatial.is_some() {
                        return Err(type_error(
                            *info,
                            &format!(
                                "{info}: Duplicate spatial specs {name} and {}",
                                spatial.unwrap()
                            ),
                        ));
                    }
                    spatial = Some(name);
                }
            }
        }
        if val.is_none() {
            return Err(type_error(*info, &format!("{info}: Missing value spec")));
        }
        if timeline.is_none() {
            return Err(type_error(*info, &format!("{info}: Missing timeline spec")));
        }
        if spatial.is_none() {
            return Err(type_error(*info, &format!("{info}: Missing spatial spec")));
        }
        Ok(Self {
            dtype_sig: std::iter::once(&ctx.specs.get(val.as_ref().unwrap()).as_ref().unwrap().sig)
                .map(Signature::from)
                .next()
                .unwrap(),
            value: val.unwrap(),
            timeline: timeline.unwrap(),
            spatial: spatial.unwrap(),
            types: HashMap::new(),
            defined_names: HashMap::new(),
        })
    }
}

/// A function type signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    pub input: Vec<DataType>,
    pub output: Vec<DataType>,
}

/// A function type signature with named inputs.
#[derive(Debug, Clone)]
pub struct NamedSignature {
    pub input: Vec<(String, DataType)>,
    pub output: Vec<DataType>,
}

impl From<&NamedSignature> for Signature {
    fn from(sig: &NamedSignature) -> Self {
        Self {
            input: sig.input.iter().cloned().map(|(_, t)| t).collect(),
            output: sig.output.clone(),
        }
    }
}

impl PartialEq for NamedSignature {
    fn eq(&self, other: &Self) -> bool {
        self.output == other.output
            && self
                .input
                .iter()
                .map(|(_, x)| x)
                .eq(other.input.iter().map(|(_, x)| x))
    }
}

impl Eq for NamedSignature {}

/// Returns true if the two signatures match, ignoring the names of the inputs.
fn sig_match(sig1: &Signature, sig2: &NamedSignature) -> bool {
    sig1.input.len() == sig2.input.len()
        && sig1
            .input
            .iter()
            .zip(sig2.input.iter())
            .all(|(t1, (_, t2))| t1 == t2)
        && sig1.output == sig2.output
}

/// A global context for a caiman program. This contains information about constants,
/// type aliases, and function signatures.
pub struct Context {
    /// Required type declarations for the program.
    pub type_decls: Vec<asm::Declaration>,
    /// Signatures of function classes. Map from class name to (input types, output types).
    pub signatures: HashMap<String, Signature>,
    /// Map from spec name to spec info.
    pub specs: HashMap<String, SpecInfo>,
    /// Map from function name to specs it implements.
    pub scheds: HashMap<String, SchedOrExtern>,
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

/// An unresolved typed binary operation containing type variables
/// instead of concrete types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct UnresolvedTypedBinop {
    op: Binop,
    op_l: String,
    op_r: String,
    ret: String,
}

/// Returns constraints on the types of the operands and the result of a binary operation.
/// # Returns
/// A tuple of (left operand constraint, right operand constraint, result constraint).
fn binop_to_contraints(
    op: Binop,
    env: &mut Env<CDataType, ADataType>,
) -> (
    Constraint<CDataType, ADataType>,
    Constraint<CDataType, ADataType>,
    Constraint<CDataType, ADataType>,
) {
    match op {
        Binop::Add | Binop::Sub | Binop::Mul | Binop::Div => {
            let s = DTypeConstraint::Num;
            let s = s.instantiate(env);
            (s.clone(), s.clone(), s)
        }
        Binop::Lt | Binop::Gt | Binop::Geq | Binop::Leq => {
            let a = DTypeConstraint::Num;
            let a = a.instantiate(env);
            let r = DTypeConstraint::Bool;
            let r = r.instantiate(env);
            (a.clone(), a, r)
        }
        Binop::Land | Binop::Lor => {
            let a = DTypeConstraint::Bool;
            let a = a.instantiate(env);
            (a.clone(), a.clone(), a)
        }
        Binop::Eq | Binop::Neq => {
            let a = DTypeConstraint::Any;
            let a = a.instantiate(env);
            let r = DTypeConstraint::Bool;
            let r = r.instantiate(env);
            (a.clone(), a, r)
        }
        Binop::And
        | Binop::Or
        | Binop::Xor
        | Binop::AShr
        | Binop::Shr
        | Binop::Shl
        | Binop::Mod => {
            let a = DTypeConstraint::Int(None);
            let a = a.instantiate(env);
            (a.clone(), a.clone(), a)
        }
        Binop::Dot | Binop::Range | Binop::Index | Binop::Cons => todo!(),
    }
}
