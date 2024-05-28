pub mod context;
use std::collections::{BTreeSet, HashMap, HashSet};
mod specs;

use crate::{
    error::{type_error, Info, LocalError},
    parse::ast::{Binop, DataType, FlaggedType, FullType, SpecType, Tag, Uop, WGPUFlags},
};
use caiman::{assembly::ast as asm, ir};

use self::{
    types::{ADataType, CDataType, DTypeConstraint, RecordConstraint},
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
    copy_dst: true,
    copy_src: true,
};

/// WGPU flags for destinations for encoded GPU copies. Input variables
/// so to speak.
pub const ENCODE_SRC_FLAGS: ir::BufferFlags = ir::BufferFlags {
    map_read: false,
    map_write: false,
    storage: true,
    uniform: false,
    copy_dst: true,
    copy_src: false,
};
/// WGPU flags for variables copied back from the GPU. Output variables
/// of an encoding so to speak.
pub const ENCODE_DST_FLAGS: ir::BufferFlags = ir::BufferFlags {
    map_read: true,
    map_write: false,
    storage: true,
    uniform: false,
    copy_dst: false,
    copy_src: false,
};

/// WGPU flags for regular encoded variables that are neither input nor output.
pub const ENCODE_STORAGE_FLAGS: ir::BufferFlags = ir::BufferFlags {
    map_read: false,
    map_write: false,
    storage: true,
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
    /// List of spec input node class names, without the leading `$` symbol.
    inputs: Vec<String>,
    /// List of function output node class names, without the leading `$` symbol.
    outputs: Vec<Option<String>>,
    /// List of spec output node class names, without the leading `$` symbol.
    spec_outputs: Vec<String>,
}

impl Default for NodeEnv {
    fn default() -> Self {
        Self {
            env: Env::new(),
            spec_nodes: HashMap::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            spec_outputs: Vec::new(),
        }
    }
}

impl NodeEnv {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the class constraint of the spec node.
    #[must_use]
    pub fn get_spec_node(&self, class: &str) -> Option<Constraint<VQType, ()>> {
        self.env.get_type(&format!("${class}"))
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
        self.spec_outputs = sig.output.iter().map(|(x, _)| x.clone()).collect();
        self.outputs = self.spec_outputs.iter().cloned().map(Some).collect();
    }

    /// Gets the output classes of the spec,
    /// without the leading `$` symbol.
    #[must_use]
    pub fn get_spec_output_classes(&self) -> &[String] {
        &self.spec_outputs
    }

    /// Gets the output classes of the function,
    /// without the leading `$` symbol. A class may be `None` if it is not  
    /// annotated and does not match up with anything in the spec.
    #[must_use]
    pub fn get_function_output_classes(&self) -> &[Option<String>] {
        &self.outputs
    }

    /// Overrides the output classes with the output classes annotated at the
    /// scheduling function.
    pub fn override_output_classes<'a, T: Iterator<Item = (&'a DataType, &'a Tag)>>(
        &mut self,
        outputs: T,
    ) {
        for (id, (_, tag)) in outputs.filter(|(dt, _)| is_value_dtype(dt)).enumerate() {
            if id >= self.outputs.len() {
                self.outputs.push(None);
            }
            if let Some(annotated_quot) = &tag.quot_var.spec_var {
                self.outputs[id] = Some(annotated_quot.clone());
            }
        }
    }

    /// Returns true if the name is a wildcard name
    fn is_wildcard_name(name: &MetaVar) -> bool {
        !name.starts_with('$')
    }

    /// Checks if the constraint uniquely identifies a quotient class and
    /// unifies the class with the type variable if it does.
    fn check_for_unique_match(&mut self, name: &str, constraint: &ValQuot) -> Result<(), String> {
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
        self.check_for_unique_match(name, constraint)
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
    flags: HashMap<String, ir::BufferFlags>,
    /// A side condition that `(sub, sup)` must be an element of the subtype relation.
    side_conditions: HashSet<(String, String)>,
}

impl Default for DTypeEnv {
    fn default() -> Self {
        Self {
            env: Env::new(),
            flags: HashMap::new(),
            side_conditions: HashSet::new(),
        }
    }
}

impl DTypeEnv {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
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
        self.env.add_constraint(name, &c).map_err(|c| {
            type_error(
                info,
                &format!("Failed to unify type constraints of variable {name}\n\n{c}"),
            )
        })?;
        self.check_side_conds(info)
    }

    /// Adds a side condition that `(subtype, supertype)` must be an element of the subtype relation.
    pub fn add_var_side_cond(&mut self, subtype: &str, supertype: &str) {
        self.side_conditions
            .insert((subtype.to_string(), supertype.to_string()));
    }

    /// Returns ok if all side conditions are satisfied.
    /// # Errors
    /// Returns an error if a side condition is not satisfied.
    fn check_side_conds(&self, info: Info) -> Result<(), LocalError> {
        for (subtype, supertype) in &self.side_conditions {
            if !match (
                self.env.get_type(subtype).map(DTypeConstraint::try_from),
                self.env.get_type(supertype).map(DTypeConstraint::try_from),
            ) {
                (
                    Some(Ok(DTypeConstraint::Record(RecordConstraint::Record {
                        fields: sub_fields,
                        ..
                    }))),
                    Some(Ok(DTypeConstraint::Record(RecordConstraint::Record {
                        fields: super_fields,
                        ..
                    }))),
                ) => !super_fields
                    .iter()
                    .any(|(n, _)| !sub_fields.contains_key(n)),
                (
                    Some(Ok(
                        DTypeConstraint::Record { .. }
                        | DTypeConstraint::Any
                        | DTypeConstraint::Var(_),
                    )),
                    Some(Ok(
                        DTypeConstraint::Record { .. }
                        | DTypeConstraint::Any
                        | DTypeConstraint::Var(_),
                    )),
                )
                | (Some(_), None)
                | (None, Some(_))
                | (Some(Err(_)), Some(Err(_))) => true,
                _ => false,
            } {
                return Err(type_error(
                    info,
                    &format!("Constraint caused violation of condition that {subtype} is a subtype of {supertype}"),
                ));
            }
        }
        Ok(())
    }

    /// Adds a constraint that a device variable is used in a particular way.
    pub fn add_usage(&mut self, name: &str, flag: WGPUFlags) {
        let f = self
            .flags
            .entry(name.to_string())
            .or_insert_with(|| ir::BufferFlags {
                map_read: false,
                map_write: false,
                storage: true,
                uniform: false,
                copy_dst: false,
                copy_src: false,
            });
        flag.apply_flag(f);
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
        self.add_constraint(name, constraint, info)?;
        self.check_side_conds(info)
    }

    /// Adds a constraint that two variables are equivalent.
    /// # Errors
    /// Returns an error if unification fails.
    pub fn add_var_equiv(&mut self, name: &str, equiv: &str, info: Info) -> Result<(), LocalError> {
        self.env
            .add_constraint(name, &Constraint::Var(equiv.to_string()))
            .map_err(|c| {
                type_error(
                    info,
                    &format!("Failed to unify type constraints of variable {name}\n\n{c}"),
                )
            })?;
        self.check_side_conds(info)
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
        self.env.add_constraint(name, constraint).map_err(|c| {
            type_error(
                info,
                &format!("Failed to unify type constraints of variable {name}\n\n{c}"),
            )
        })?;
        self.check_side_conds(info)
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
    /// Map from device variable name to flags.
    pub flags: HashMap<String, ir::BufferFlags>,
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
            Self::Extern(..) => panic!("Expected schedule, got extern"),
        }
    }

    /// Unwraps the schedule info.
    /// # Panics
    /// If this is an extern.
    #[must_use]
    pub fn unwrap_sched_mut(&mut self) -> &mut SchedInfo {
        match self {
            Self::Sched(s) => s,
            Self::Extern(..) => panic!("Expected schedule, got extern"),
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
            flags: HashMap::new(),
        })
    }
}

/// A function type signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    pub input: Vec<FlaggedType>,
    pub output: Vec<FlaggedType>,
}

impl Signature {
    #[must_use]
    pub fn new(input: Vec<DataType>, output: Vec<DataType>) -> Self {
        Self {
            input: input
                .into_iter()
                .map(|x| FlaggedType {
                    info: Info::default(),
                    base: x,
                    flags: BTreeSet::new(),
                    settings: BTreeSet::new(),
                })
                .collect(),
            output: output
                .into_iter()
                .map(|x| FlaggedType {
                    info: Info::default(),
                    base: x,
                    flags: BTreeSet::new(),
                    settings: BTreeSet::new(),
                })
                .collect(),
        }
    }
}

/// A function type signature with named inputs.
#[derive(Debug, Clone)]
pub struct NamedSignature {
    pub input: Vec<(String, FlaggedType)>,
    pub output: Vec<(String, FlaggedType)>,
}

impl NamedSignature {
    /// Creates a new named signature from a list of input and output types.
    #[must_use]
    pub fn new<'a, I: Iterator<Item = &'a (Option<String>, DataType)>>(
        input: &[(String, DataType)],
        output: I,
    ) -> Self {
        Self {
            input: input
                .iter()
                .map(|(name, typ)| (name.clone(), FlaggedType::from(typ.clone())))
                .collect(),
            output: output
                .enumerate()
                .map(|(idx, (name, typ))| {
                    (
                        name.clone().unwrap_or_else(|| format!("_out{idx}")),
                        FlaggedType::from(typ.clone()),
                    )
                })
                .collect::<Vec<_>>(),
        }
    }
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
#[derive(Debug, Clone)]
pub struct Context {
    /// Required type declarations for the program.
    pub type_decls: Vec<asm::Declaration>,
    /// Signatures of function classes. Map from class name to (input types, output types).
    pub signatures: HashMap<String, Signature>,
    /// Map from spec name to spec info.
    pub specs: HashMap<String, SpecInfo>,
    /// Map from function name to specs it implements.
    pub scheds: HashMap<String, SchedOrExtern>,
    /// Set of external function names.
    pub externs: HashSet<String>,
    /// User defined types. Map from type name to type.
    pub user_types: HashMap<String, FlaggedType>,
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

/// Returns `true` if the type can be represented in the timeline spec.
/// If this function returns false, then the argument or return value can be
/// ignored for the purpose of inferring timeline quotients.
#[must_use]
#[allow(unused)]
pub const fn is_timeline_dtype(t: &DataType) -> bool {
    matches!(
        t,
        DataType::Event | DataType::Fence(_) | DataType::Encoder(_)
    )
}

/// Returns `true` if the type can be represented in the timeline spec.
/// If this function returns false, then the argument or return value can be
/// ignored for the purpose of inferring timeline quotients.
#[must_use]
#[allow(unused)]
pub const fn is_timeline_fulltype(t: &FullType) -> bool {
    if let Some(t) = &t.base {
        is_timeline_dtype(&t.base)
    } else {
        false
    }
}

/// Returns `true` if the type can be represented in the value spec.
/// If this function returns false, then the argument or return value can be
/// ignored for the purpose of inferring value quotients.
#[must_use]
pub const fn is_value_dtype(t: &DataType) -> bool {
    // TODO: Add more types
    matches!(
        t,
        DataType::Int(_)
            | DataType::Float(_)
            | DataType::Bool
            | DataType::Ref(_)
            | DataType::Array(_, _)
    )
}
