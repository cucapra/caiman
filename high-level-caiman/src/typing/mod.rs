pub mod context;
use std::collections::{HashMap, HashSet};
mod specs;

use crate::{
    error::{type_error, Info, LocalError},
    parse::ast::{Binop, DataType, SpecType, Uop},
};
use caiman::{assembly::ast as asm, ir};

use self::{
    types::{ADataType, CDataType, DTypeConstraint},
    unification::{Constraint, Env},
};
mod sched;
#[cfg(test)]
mod test;
mod types;
mod unification;

pub use types::{MetaVar, VQType, ValQuot};

/// WGPU flags for all frontent temporaries.
pub const LOCAL_TEMP_FLAGS: ir::BufferFlags = ir::BufferFlags {
    map_read: true,
    map_write: true,
    storage: false,
    uniform: false,
    copy_dst: false,
    copy_src: false,
};

/// A typing environement for deducing quotients.
#[derive(Debug, Clone)]
pub struct NodeEnv {
    env: Env<VQType, ()>,
    /// Map of quotient type to a map between quotients and their equivalence
    /// class names.
    spec_nodes: HashMap<VQType, HashMap<ValQuot, HashSet<String>>>,
    /// List of input node class names, without the leading `$` symbol.
    inputs: Vec<String>,
    /// List of output node class names, without the leading `$` symbol.
    outputs: Vec<String>,
}

impl Default for NodeEnv {
    fn default() -> Self {
        Self {
            env: Env::new(),
            spec_nodes: HashMap::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }
}

impl NodeEnv {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a quotient class and its constraints to the environment.
    /// If the constraint is an input constraint, adds the class name to the
    /// list of input class names.
    /// # Panics
    /// If the class name contains a `$` or if the constraint could not
    /// be added to the environment.
    pub fn add_quotient(&mut self, class_name: &str, constraint: ValQuot) {
        assert!(!class_name.contains('$'));
        if let ValQuot::Input(_) = &constraint {
            self.inputs.push(class_name.to_string());
        }
        let class_name = format!("${class_name}");
        self.env
            .add_class_constraint(&class_name, &From::from(&constraint))
            .unwrap();
        self.spec_nodes
            .entry(constraint.get_type())
            .or_default()
            .entry(constraint)
            .or_default()
            .insert(class_name);
    }

    /// Sets the qotient classes of the outputs.
    /// # Panics
    /// If any of the class names contain a `$`.
    pub fn set_output_classes(&mut self, sig: &NamedSignature) {
        assert!(sig.output.iter().all(|(x, _)| !x.contains('$')));
        self.outputs = sig.output.iter().map(|(x, _)| x.clone()).collect();
    }

    /// Gets the output classes, without the leading `$` symbol.
    #[must_use]
    pub fn get_output_classes(&self) -> &[String] {
        &self.outputs
    }

    /// Returns true if the name is a wildcard name
    fn is_wildcard_name(name: &MetaVar) -> bool {
        !name.starts_with('$')
    }

    /// Adds a constraint to the type variable `name`. If the constraint
    /// uniquely identifies a quotient class, unifies the quotient class with
    /// the type variable.
    /// # Errors
    /// Returns an error if unification fails.
    /// # Panics
    /// If the name contains a `$`.
    pub fn add_constraint(&mut self, name: &str, constraint: &ValQuot) -> Result<(), String> {
        assert!(!name.contains('$'));
        self.env.add_constraint(name, &From::from(constraint))?;
        if let Some(matches) = self.spec_nodes.get(&constraint.get_type()) {
            let mut last_match_classes = None;
            for (possible_match, match_classes) in matches {
                if possible_match.matches(constraint, Self::is_wildcard_name) {
                    if last_match_classes.is_some() {
                        return Ok(()); // Not a unique match
                    }
                    last_match_classes = Some(match_classes);
                }
            }
            if let Some(last_match_classes) = last_match_classes {
                if last_match_classes.len() == 1 {
                    self.env.add_class_constraint(
                        last_match_classes.iter().next().unwrap(),
                        &Constraint::Var(name.to_string()),
                    )?;
                }
            }
        }
        Ok(())
    }

    /// Adds a constraint to the type variable `name` with another variable.
    /// # Errors
    /// Returns an error if unification fails.
    /// # Panics
    /// If the name contains a `$` or if the equiv contains a `$`.
    pub fn add_var_eq(&mut self, name: &str, equiv: &str) -> Result<(), String> {
        assert!(!name.contains('$'));
        assert!(!equiv.contains('$'));
        self.env
            .add_constraint(name, &Constraint::Var(equiv.to_string()))
    }

    /// Adds an equivalence between variable `name` and spec node name `class_name`.
    /// # Panics
    /// If the name contains a `$` or if the class name contains a `$`.
    /// # Errors
    /// Returns an error if unification fails.
    pub fn add_node_eq(&mut self, name: &str, class_name: &str) -> Result<(), String> {
        assert!(!name.contains('$'));
        let class_name = if class_name.starts_with('$') {
            class_name.to_string()
        } else {
            format!("${class_name}")
        };
        assert_eq!(class_name.chars().filter(|x| *x == '$').count(), 1);
        self.env
            .add_class_constraint(&class_name, &Constraint::Var(name.to_string()))
    }

    /// Returns the variable's matching node name in the spec if it has one.
    #[must_use]
    pub fn get_node_name(&self, name: &str) -> Option<String> {
        self.env
            .get_class_id(name)
            .map(|x| x.trim_matches('$').to_string())
    }

    /// Returns the classes of the input variables
    #[must_use]
    pub fn get_input_classes(&self) -> &[String] {
        &self.inputs
    }

    /// Gets names of equivalence classes that are literals in the spec.
    #[must_use]
    pub fn literal_classes(&self) -> HashSet<String> {
        let mut res = HashSet::new();
        for classes in self.spec_nodes.values() {
            for (c, class) in classes {
                if matches!(c, ValQuot::Int(_) | ValQuot::Bool(_) | ValQuot::Float(_)) {
                    res.extend(class.iter().map(|x| x.trim_matches('$').to_string()));
                }
            }
        }
        res
    }
}

/// A data type typing environment.
#[derive(Debug)]
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
        self.env.add_constraint(name, &c).map_err(|_| {
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
            .map_err(|_| {
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
        self.env.add_constraint(name, constraint).map_err(|_| {
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
    /// Typing environment storing all nodes in the spec.
    pub nodes: NodeEnv,
    /// Map from variable name to type.
    pub types: HashMap<String, DataType>,
    /// Source-level starting and ending line and column number.
    pub info: Info,
    /// The function class if the spec is a value spec.
    pub feq: Option<String>,
}

impl SpecInfo {
    #[must_use]
    pub fn new(typ: SpecType, sig: NamedSignature, info: Info, class_name: Option<&str>) -> Self {
        Self {
            typ,
            sig,
            types: HashMap::new(),
            info,
            nodes: NodeEnv::new(),
            feq: class_name.map(ToString::to_string),
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
    pub fn new(
        specs: [String; 3],
        ctx: &Context,
        sig: Signature,
        info: &Info,
    ) -> Result<Self, LocalError> {
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
            dtype_sig: sig,
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
    pub output: Vec<(String, DataType)>,
}

impl From<&NamedSignature> for Signature {
    fn from(sig: &NamedSignature) -> Self {
        Self {
            input: sig.input.iter().cloned().map(|(_, t)| t).collect(),
            output: sig.output.iter().cloned().map(|(_, t)| t).collect(),
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
        && sig1.input.iter().eq(sig2.input.iter().map(|(_, t)| t))
        && sig1.output.iter().eq(sig2.output.iter().map(|(_, t)| t))
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

/// Returns constraints on the type of the operand and the result of a unary operation.
/// # Returns
/// A tuple of (operand constraint, result constraint).
fn uop_to_contraints(
    op: Uop,
    env: &mut Env<CDataType, ADataType>,
) -> (
    Constraint<CDataType, ADataType>,
    Constraint<CDataType, ADataType>,
) {
    match op {
        Uop::Neg => {
            let a = DTypeConstraint::Num;
            let a = a.instantiate(env);
            (a.clone(), a)
        }
        Uop::LNot => {
            let a = DTypeConstraint::Bool;
            let a = a.instantiate(env);
            (a.clone(), a)
        }
        Uop::Ref => {
            let a = DTypeConstraint::Any;
            let a = a.instantiate(env);
            let r = DTypeConstraint::Ref(a.clone());
            (a, r.instantiate(env))
        }
        Uop::Deref => {
            let any = DTypeConstraint::Any.instantiate(env);
            let a = DTypeConstraint::Ref(any.clone());
            let a = a.instantiate(env);
            (a, any)
        }
        Uop::Not => {
            let a = DTypeConstraint::Int(None);
            let a = a.instantiate(env);
            (a.clone(), a)
        }
    }
}
