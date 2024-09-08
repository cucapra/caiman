use std::{
    collections::{BTreeMap, BTreeSet},
    vec,
};

use crate::parse::ast::{Binop, DataType, FloatSize, IntSize};

use super::unification::{Constraint, Env, Kind, SubtypeConstraint};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CDataType {
    Num,
    Int,
    Float,
    Ref,
    Record,
    Encoder,
    Fence,
    RemoteObj,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ADataType {
    I32,
    I64,
    F64,
    Bool,
    BufferSpace,
    Event,
    SpecEncoder,
    SpecFence,
}

impl Kind for CDataType {}
impl Kind for ADataType {}

/// A constraint on a record type. Either a map from field names to data type constraints
/// or a metavariable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordConstraint {
    Record {
        fields: BTreeMap<String, DTypeConstraint>,
        constraint_kind: SubtypeConstraint,
    },
    Var(String),
    Any,
}

impl TryFrom<DTypeConstraint> for RecordConstraint {
    type Error = String;

    fn try_from(value: DTypeConstraint) -> Result<Self, Self::Error> {
        match value {
            DTypeConstraint::Any => Ok(Self::Any),
            DTypeConstraint::Var(s) => Ok(Self::Var(s)),
            DTypeConstraint::Record(r) => Ok(r),
            x => Err(format!("Unexpected constraint {x:?}")),
        }
    }
}

/// A high-level data type constraint. Holes in a data type
/// constraint are universally quantified. To get multiple
/// copies of the same constraint, clone the constraint
/// after instantiating it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DTypeConstraint {
    Int(Option<IntSize>),
    Float(Option<FloatSize>),
    Bool,
    BufferSpace,
    Event,
    /// Any numeric type.
    Num,
    /// Any type.
    Any,
    /// A reference constraint which contains an already instantiated
    /// data type constraint. This is done for greater flexibility
    /// so that each reference constraint need not have a unique base type.
    Ref(Constraint<CDataType, ADataType>),
    /// A reference constraint which contains a dtype constraint
    /// that will be instantiated to a new inner data type constraint.
    RefN(Box<DTypeConstraint>),
    // Encoder(Constraint<CDataType, ADataType>),
    //Fence(Constraint<CDataType, ADataType>),
    Encoder(Box<DTypeConstraint>),
    Fence(Box<DTypeConstraint>),
    Record(RecordConstraint),
    RemoteObj {
        /// record of all remote variables (have the storage flag)
        all: RecordConstraint,
        /// record of all readable variables that must be copied back to the CPU (`map_read`)
        read: RecordConstraint,
    },
    SpecEncoder,
    SpecFence,
    Var(String),
}

impl DTypeConstraint {
    fn record_into_subtypeable(r: RecordConstraint) -> RecordConstraint {
        match r {
            RecordConstraint::Var(_) | RecordConstraint::Any => r,
            RecordConstraint::Record { fields, .. } => RecordConstraint::Record {
                fields: fields
                    .into_iter()
                    .map(|(k, v)| (k, v.into_subtypeable()))
                    .collect(),
                constraint_kind: SubtypeConstraint::Any,
            },
        }
    }
    /// Converts the constraint into one which allows subtypes.
    /// Used when we generate a constraint from a data type but we want to allow
    /// subtypes of the data type.
    pub fn into_subtypeable(self) -> Self {
        match self {
            Self::Int(_)
            | Self::Float(_)
            | Self::Bool
            | Self::BufferSpace
            | Self::Event
            | Self::Num
            | Self::Any
            | Self::Ref(_)
            | Self::Var(_)
            | Self::SpecEncoder
            | Self::SpecFence => self,
            Self::RefN(x) => Self::RefN(Box::new(x.into_subtypeable())),
            Self::Encoder(x) => Self::Encoder(Box::new(x.into_subtypeable())),
            Self::Fence(x) => Self::Fence(Box::new(x.into_subtypeable())),
            Self::Record(r) => Self::Record(Self::record_into_subtypeable(r)),
            Self::RemoteObj { all, read } => Self::RemoteObj {
                all: Self::record_into_subtypeable(all),
                read: Self::record_into_subtypeable(read),
            },
        }
    }
}

impl From<IntSize> for ADataType {
    fn from(i: IntSize) -> Self {
        match i {
            IntSize::I32 => Self::I32,
            IntSize::I64 => Self::I64,
        }
    }
}

impl TryFrom<ADataType> for IntSize {
    type Error = String;
    fn try_from(i: ADataType) -> Result<Self, Self::Error> {
        match i {
            ADataType::I32 => Ok(Self::I32),
            ADataType::I64 => Ok(Self::I64),
            _ => Err(format!("Cannot convert {i:?} to IntSize")),
        }
    }
}

impl From<FloatSize> for ADataType {
    fn from(f: FloatSize) -> Self {
        match f {
            FloatSize::F64 => Self::F64,
        }
    }
}

impl TryFrom<ADataType> for FloatSize {
    type Error = String;
    fn try_from(f: ADataType) -> Result<Self, Self::Error> {
        match f {
            ADataType::F64 => Ok(Self::F64),
            _ => Err(format!("Cannot convert {f:?} to FloatSize")),
        }
    }
}

impl DTypeConstraint {
    fn instantiate_record(
        r: RecordConstraint,
        env: &mut Env<CDataType, ADataType>,
    ) -> Constraint<CDataType, ADataType> {
        match r {
            RecordConstraint::Record {
                fields,
                constraint_kind,
            } => {
                let mut mp = BTreeMap::new();
                for (k, v) in fields {
                    mp.insert(k, v.instantiate(env));
                }
                Constraint::DynamicTerm(CDataType::Record, mp, constraint_kind)
            }
            RecordConstraint::Var(s) => Constraint::Var(s),
            RecordConstraint::Any => {
                let t = env.new_temp_type();
                Constraint::Var(t)
            }
        }
    }
    /// Instantiates a high-level type constraint into a unification-level
    /// constraint.
    pub fn instantiate(
        self,
        env: &mut Env<CDataType, ADataType>,
    ) -> Constraint<CDataType, ADataType> {
        match self {
            Self::Num => {
                let t = env.new_temp_type();
                Constraint::Term(CDataType::Num, vec![Constraint::Var(t)])
            }
            Self::Int(Some(x)) => Constraint::Term(
                CDataType::Num,
                vec![Constraint::Term(
                    CDataType::Int,
                    vec![Constraint::Atom(x.into())],
                )],
            ),
            Self::Int(None) => {
                let t = env.new_temp_type();
                Constraint::Term(
                    CDataType::Num,
                    vec![Constraint::Term(CDataType::Int, vec![Constraint::Var(t)])],
                )
            }
            Self::Float(Some(x)) => Constraint::Term(
                CDataType::Num,
                vec![Constraint::Term(
                    CDataType::Float,
                    vec![Constraint::Atom(x.into())],
                )],
            ),
            Self::Float(None) => {
                let t = env.new_temp_type();
                Constraint::Term(
                    CDataType::Num,
                    vec![Constraint::Term(CDataType::Float, vec![Constraint::Var(t)])],
                )
            }
            Self::Any => {
                let t = env.new_temp_type();
                Constraint::Var(t)
            }
            Self::Bool => Constraint::Atom(ADataType::Bool),
            Self::BufferSpace => Constraint::Atom(ADataType::BufferSpace),
            Self::Event => Constraint::Atom(ADataType::Event),
            Self::Ref(x) => Constraint::Term(CDataType::Ref, vec![x]),
            Self::RefN(x) => Constraint::Term(CDataType::Ref, vec![x.instantiate(env)]),
            Self::Encoder(typ) => Constraint::Term(CDataType::Encoder, vec![typ.instantiate(env)]),
            Self::Fence(public) => {
                Constraint::Term(CDataType::Fence, vec![public.instantiate(env)])
            }
            Self::Record(r) => Self::instantiate_record(r, env),
            Self::RemoteObj { all, read } => Constraint::Term(
                CDataType::RemoteObj,
                vec![
                    Self::instantiate_record(all, env),
                    Self::instantiate_record(read, env),
                ],
            ),
            // Self::EncoderN(typ) => Constraint::Term(CDataType::Encoder, vec![typ.instantiate(env)]),
            // Self::FenceN(public) => {
            //     Constraint::Term(CDataType::Fence, vec![public.instantiate(env)])
            // }
            Self::SpecEncoder => Constraint::Atom(ADataType::SpecEncoder),
            Self::SpecFence => Constraint::Atom(ADataType::SpecFence),
            Self::Var(s) => Constraint::Var(s),
        }
    }
}

const DEFAULT_INT_SIZE: IntSize = IntSize::I64;
const DEFAULT_FLOAT_SIZE: FloatSize = FloatSize::F64;

impl TryFrom<DTypeConstraint> for DataType {
    type Error = ();
    /// Tries to convert a data type constraint into a concrete data type.
    /// If the high-level data type constraint is not constrained enough to be converted
    /// into a concrete data type, an error is returned.
    fn try_from(dt: DTypeConstraint) -> Result<Self, ()> {
        match dt {
            DTypeConstraint::Int(Some(x)) => Ok(Self::Int(x)),
            // if there's no size constraint, use the default size
            DTypeConstraint::Int(None) | DTypeConstraint::Num => Ok(Self::Int(DEFAULT_INT_SIZE)),
            DTypeConstraint::Float(Some(x)) => Ok(Self::Float(x)),
            // if there's no size constraint, use the default size
            DTypeConstraint::Float(None) => Ok(Self::Float(DEFAULT_FLOAT_SIZE)),
            DTypeConstraint::Bool => Ok(Self::Bool),
            DTypeConstraint::BufferSpace => Ok(Self::BufferSpace),
            DTypeConstraint::Event => Ok(Self::Event),
            DTypeConstraint::Ref(x) => Ok(Self::Ref(Box::new(Self::try_from(
                DTypeConstraint::try_from(x).map_err(|_| ())?,
            )?))),
            DTypeConstraint::RefN(x) => Ok(Self::Ref(Box::new(Self::try_from(*x)?))),
            DTypeConstraint::Record(RecordConstraint::Record { fields, .. }) => {
                let mut mp = Vec::new();
                for (k, v) in fields {
                    mp.push((k, Self::try_from(v)?));
                }
                Ok(Self::Record(mp))
            }
            DTypeConstraint::RemoteObj {
                all: RecordConstraint::Record { fields, .. },
                read: RecordConstraint::Record { fields: read, .. },
            } => {
                let mut mp = Vec::new();
                for (k, v) in fields {
                    mp.push((k, Self::try_from(v)?));
                }
                let read = read.into_keys().collect();
                Ok(Self::RemoteObj { all: mp, read })
            }
            DTypeConstraint::RemoteObj {
                all: RecordConstraint::Any,
                read: RecordConstraint::Record { fields, .. },
            }
            | DTypeConstraint::RemoteObj {
                all: RecordConstraint::Record { fields, .. },
                read: RecordConstraint::Any,
            } => {
                let all = fields
                    .iter()
                    .map(|(nm, constraint)| {
                        (nm.clone(), Self::try_from(constraint.clone()).unwrap())
                    })
                    .collect();
                let rw: BTreeSet<_> = fields.into_keys().collect();
                Ok(Self::RemoteObj { all, read: rw })
            }
            DTypeConstraint::Encoder(typ) => {
                Ok(Self::Encoder(Some(Box::new(Self::try_from(*typ)?))))
            }
            DTypeConstraint::Fence(vars) => Ok(Self::Fence(Some(Box::new(Self::try_from(*vars)?)))),
            DTypeConstraint::SpecEncoder => Ok(Self::Encoder(None)),
            DTypeConstraint::SpecFence => Ok(Self::Fence(None)),
            DTypeConstraint::Any
            | DTypeConstraint::Var(_)
            | DTypeConstraint::Record(RecordConstraint::Var(_) | RecordConstraint::Any)
            | DTypeConstraint::RemoteObj { .. } => Err(()),
        }
    }
}

impl TryFrom<Constraint<CDataType, ADataType>> for DTypeConstraint {
    type Error = String;
    fn try_from(c: Constraint<CDataType, ADataType>) -> Result<Self, String> {
        match c {
            Constraint::Atom(ADataType::Bool) => Ok(Self::Bool),
            Constraint::Atom(ADataType::BufferSpace) => Ok(Self::BufferSpace),
            Constraint::Atom(ADataType::Event) => Ok(Self::Event),
            Constraint::Term(CDataType::Num, mut v) => {
                let d = v.swap_remove(0);
                match d {
                    Constraint::Var(_) => Ok(Self::Num),
                    Constraint::Term(CDataType::Int, mut v) => {
                        let d = v.swap_remove(0);
                        match d {
                            Constraint::Atom(x) => Ok(Self::Int(Some(x.try_into()?))),
                            Constraint::Var(_) => Ok(Self::Int(None)),
                            Constraint::Term(..)
                            | Constraint::DynamicTerm(..)
                            | Constraint::DropTerm(..) => unreachable!(),
                        }
                    }
                    Constraint::Term(CDataType::Float, mut v) => {
                        let d = v.swap_remove(0);
                        match d {
                            Constraint::Atom(x) => Ok(Self::Float(Some(x.try_into()?))),
                            Constraint::Var(_) => Ok(Self::Float(None)),
                            Constraint::Term(..)
                            | Constraint::DynamicTerm(..)
                            | Constraint::DropTerm(..) => unreachable!(),
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Constraint::Term(CDataType::Ref, mut v) => {
                assert_eq!(v.len(), 1, "Ref constraint should have exactly one child");
                let d = v.swap_remove(0);
                Ok(Self::Ref(d))
            }
            Constraint::Var(_) => Ok(Self::Any),
            Constraint::Term(CDataType::Encoder, mut v) => {
                assert_eq!(
                    v.len(),
                    1,
                    "Encoder constraint should have exactly one child"
                );
                let d = v.swap_remove(0);
                Ok(Self::Encoder(Box::new(d.try_into()?)))
            }
            Constraint::Term(CDataType::Fence, mut v) => {
                assert_eq!(v.len(), 1, "Fence constraint should have exactly one child");
                let d = v.swap_remove(0);
                Ok(Self::Fence(Box::new(d.try_into()?)))
            }
            Constraint::DynamicTerm(CDataType::Record, fields, constraint_kind) => {
                let mut mp = BTreeMap::new();
                for (k, v) in fields {
                    mp.insert(k, Self::try_from(v)?);
                }
                Ok(Self::Record(RecordConstraint::Record {
                    fields: mp,
                    constraint_kind,
                }))
            }
            Constraint::Term(CDataType::RemoteObj, mut v) => {
                assert_eq!(
                    v.len(),
                    2,
                    "Remote obj constraint should have exactly two children"
                );
                let read = Self::try_from(v.pop().unwrap())?;
                let all = Self::try_from(v.pop().unwrap())?;
                Ok(Self::RemoteObj {
                    all: RecordConstraint::try_from(all)?,
                    read: RecordConstraint::try_from(read)?,
                })
            }
            Constraint::Atom(ADataType::SpecEncoder) => Ok(Self::SpecEncoder),
            Constraint::Atom(ADataType::SpecFence) => Ok(Self::SpecFence),
            _ => todo!("Cannot convert {c:?} to DTypeConstraint"),
        }
    }
}

/// Converts a map of string to data types to a map of string to data type constraints
fn record_dtypes_to_constraints(
    fields: Vec<(String, DataType)>,
) -> BTreeMap<String, DTypeConstraint> {
    let mut mp = BTreeMap::new();
    for (k, v) in fields {
        mp.insert(k, DTypeConstraint::from(v));
    }
    mp
}

/// Converts a set of strings to a map of string to the Any data type constraint
fn set_dtypes_to_constraints(set: BTreeSet<String>) -> BTreeMap<String, DTypeConstraint> {
    let mut mp = BTreeMap::new();
    for k in set {
        mp.insert(k, DTypeConstraint::Any);
    }
    mp
}

impl From<DataType> for DTypeConstraint {
    fn from(dt: DataType) -> Self {
        match dt {
            DataType::Int(IntSize::I32) => Self::Int(Some(IntSize::I32)),
            DataType::Int(IntSize::I64) => Self::Int(Some(IntSize::I64)),
            DataType::Float(FloatSize::F64) => Self::Float(Some(FloatSize::F64)),
            DataType::Bool => Self::Bool,
            DataType::BufferSpace => Self::BufferSpace,
            DataType::Event => Self::Event,
            DataType::Ref(x) => Self::RefN(Box::new(Self::from(*x))),
            DataType::Encoder(None) => Self::SpecEncoder,
            DataType::Fence(None) => Self::SpecFence,
            DataType::Encoder(Some(x)) => Self::Encoder(Box::new(Self::from(*x))),
            DataType::Fence(Some(x)) => Self::Fence(Box::new(Self::from(*x))),
            DataType::Record(fields) => Self::Record(RecordConstraint::Record {
                fields: record_dtypes_to_constraints(fields),
                // when annotated, we cannot deduce a subtype of the annotation
                constraint_kind: SubtypeConstraint::Contravariant,
            }),
            DataType::RemoteObj { all, read } => Self::RemoteObj {
                // annotations cannot be deduced to be lower types in the lattice then the annotation
                all: RecordConstraint::Record {
                    fields: record_dtypes_to_constraints(all),
                    constraint_kind: SubtypeConstraint::Contravariant,
                },
                read: RecordConstraint::Record {
                    fields: set_dtypes_to_constraints(read),
                    constraint_kind: SubtypeConstraint::Contravariant,
                },
            },
            _ => todo!("Cannot convert {dt:?} to DTypeConstraint"),
        }
    }
}

/// A type metavariable. Either a class name or a variable name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MetaVar(String);

impl MetaVar {
    /// Returns true if the metavariable starts with the given character
    #[must_use]
    pub fn starts_with(&self, c: char) -> bool {
        self.0.starts_with(c)
    }

    /// Creates a type equivalence class name
    #[must_use]
    pub fn new_class_name(s: &str) -> Self {
        Self(format!("${s}"))
    }

    /// Creates a type variable name
    #[must_use]
    pub fn new_var_name(s: &str) -> Self {
        Self(s.to_string())
    }

    /// Returns the string representation of the metavariable
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn into_string(self) -> String {
        self.0
    }

    #[must_use]
    pub fn get(&self) -> &str {
        &self.0
    }

    /// If this is a class name, returns that name without the leading `$`
    #[must_use]
    pub fn get_class_name(&self) -> Option<&str> {
        if self.0.starts_with('$') {
            Some(&self.0[1..])
        } else {
            None
        }
    }
}

/// A constraint on a value quotient
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ValQuot {
    Int(String),
    Float(String),
    Bool(bool),
    Input(String),
    Output(MetaVar),
    /// A call in the spec, or a call in the schedule where unconstrained
    /// arguments cannot be dropped.
    Call(String, Vec<MetaVar>),
    /// A call with a single return that does not need an extraction.
    /// Unconstrained arguments are not dropped.
    CallOne(String, Vec<MetaVar>),
    Extract(MetaVar, usize),
    Bop(Binop, MetaVar, MetaVar),
    Select {
        guard: MetaVar,
        true_id: MetaVar,
        false_id: MetaVar,
    },
    /// A call in the schedule, where unmatching unconstrained arguments
    /// can be ignored.
    SchedCall(String, Vec<MetaVar>),
}

impl ValQuot {
    /// Returns true if the value quotient matches the other value quotient
    /// up to variable wildcards
    /// # Arguments
    /// * `other` - The other value quotient to match against
    /// * `is_wildcard_name` - A function that returns true if the given metavariable
    ///                       is a wildcard metavariable
    pub fn matches<F: Fn(&MetaVar) -> bool>(&self, other: &Self, is_wildcard_name: F) -> bool {
        match (self, other) {
            (Self::Float(x), Self::Float(y))
            | (Self::Int(x), Self::Int(y))
            | (Self::Input(x), Self::Input(y)) => x == y,
            (Self::Output(x), Self::Output(y)) => x == y,
            (Self::Bool(x), Self::Bool(y)) => x == y,
            (Self::Call(f1, args1) | Self::SchedCall(f1, args1), Self::Call(f2, args2))
            | (Self::CallOne(f1, args1), Self::CallOne(f2, args2))
            | (Self::Call(f1, args1), Self::SchedCall(f2, args2)) => {
                f1 == f2
                    && args1.len() == args2.len()
                    && args1
                        .iter()
                        .zip(args2.iter())
                        .all(|(x, y)| x == y || is_wildcard_name(x) || is_wildcard_name(y))
            }
            (Self::Extract(x, i), Self::Extract(y, j)) => {
                i == j && (x == y || is_wildcard_name(x) || is_wildcard_name(y))
            }
            (Self::Bop(op1, x1, y1), Self::Bop(op2, x2, y2)) => {
                op1 == op2
                    && (x1 == x2 || is_wildcard_name(x1) || is_wildcard_name(x2))
                    && (y1 == y2 || is_wildcard_name(y1) || is_wildcard_name(y2))
            }
            (
                Self::Select {
                    guard: g1,
                    true_id: t1,
                    false_id: f1,
                },
                Self::Select {
                    guard: g2,
                    true_id: t2,
                    false_id: f2,
                },
            ) => {
                (g1 == g2 || is_wildcard_name(g1) || is_wildcard_name(g2))
                    && (t1 == t2 || is_wildcard_name(t1) || is_wildcard_name(t2))
                    && (f1 == f2 || is_wildcard_name(f1) || is_wildcard_name(f2))
            }
            _ => false,
        }
    }

    /// Returns the type of the value quotient
    #[must_use]
    pub fn get_type(&self) -> VQType {
        self.into()
    }
}

/// Classifications of value quotients. These correspond to the nodes
/// in the value specification resource graph.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum VQType {
    Int(String),
    Float(String),
    Bool(bool),
    Input(String),
    Output,
    Call(String),
    Extract(usize),
    Bop(Binop),
    Select,
    CallBuiltin(String),
}

impl std::fmt::Debug for VQType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(i) => write!(f, "Int({i})"),
            Self::Float(i) => write!(f, "Float({i})"),
            Self::Bool(b) => write!(f, "Bool({b})"),
            Self::Input(i) => write!(f, "Input({i})"),
            Self::Output => write!(f, "Output"),
            Self::Call(i) => write!(f, "Call({i})"),
            Self::Extract(i) => write!(f, "Extract({i})"),
            Self::Bop(op) => write!(f, "Bop({op:?})"),
            Self::Select => write!(f, "Select"),
            Self::CallBuiltin(i) => write!(f, "{i}"),
        }
    }
}

impl From<&ValQuot> for VQType {
    fn from(value: &ValQuot) -> Self {
        match value {
            ValQuot::Int(i) => Self::Int(i.clone()),
            ValQuot::Float(f) => Self::Float(f.clone()),
            ValQuot::Bool(b) => Self::Bool(*b),
            ValQuot::Input(i) => Self::Input(i.clone()),
            ValQuot::Output(_) => Self::Output,
            ValQuot::Call(f, _) | ValQuot::SchedCall(f, _) => Self::Call(f.clone()),
            ValQuot::Extract(_, j) => Self::Extract(*j),
            ValQuot::Bop(op, _, _) => Self::Bop(*op),
            ValQuot::Select { .. } => Self::Select,
            ValQuot::CallOne(s, _) => Self::CallBuiltin(s.clone()),
        }
    }
}

impl Kind for VQType {}
impl Kind for () {}

impl From<&ValQuot> for Constraint<VQType, ()> {
    fn from(value: &ValQuot) -> Self {
        match value {
            ValQuot::Int(i) => Self::Term(VQType::Int(i.clone()), vec![]),
            ValQuot::Float(f) => Self::Term(VQType::Float(f.clone()), vec![]),
            ValQuot::Bool(b) => Self::Term(VQType::Bool(*b), vec![]),
            ValQuot::Input(i) => Self::Term(VQType::Input(i.clone()), vec![]),
            ValQuot::Output(o) => Self::Term(VQType::Output, vec![Self::Var(o.0.clone())]),
            ValQuot::Call(f, args) => Self::Term(
                VQType::Call(f.clone()),
                args.iter().map(|x| Self::Var(x.0.clone())).collect(),
            ),
            ValQuot::CallOne(f, args) => Self::Term(
                VQType::CallBuiltin(f.clone()),
                args.iter().map(|x| Self::Var(x.0.clone())).collect(),
            ),
            ValQuot::SchedCall(f, args) => Self::DropTerm(
                VQType::Call(f.clone()),
                args.iter().map(|x| Self::Var(x.0.clone())).collect(),
            ),
            ValQuot::Extract(tuple, j) => {
                Self::Term(VQType::Extract(*j), vec![Self::Var(tuple.0.clone())])
            }
            ValQuot::Bop(op, x, y) => Self::Term(
                VQType::Bop(*op),
                vec![Self::Var(x.0.clone()), Self::Var(y.0.clone())],
            ),
            ValQuot::Select {
                guard,
                true_id,
                false_id,
            } => Self::Term(
                VQType::Select,
                vec![
                    Self::Var(guard.0.clone()),
                    Self::Var(true_id.0.clone()),
                    Self::Var(false_id.0.clone()),
                ],
            ),
        }
    }
}

/// Converts a constraint to a value quotient with wildcard metavariables.
/// This value quotient will have all metavariables with `?` as the name.
///
/// This intention is for the returned value quotient to be used in a situation
/// that operates modulo metavariable names.
pub fn constraint_to_wildcard_vq(value: &Constraint<VQType, ()>) -> ValQuot {
    match value {
        Constraint::Term(VQType::Int(i), _) => ValQuot::Int(i.clone()),
        Constraint::Term(VQType::Float(f), _) => ValQuot::Float(f.clone()),
        Constraint::Term(VQType::Bool(b), _) => ValQuot::Bool(*b),
        Constraint::Term(VQType::Input(i), _) => ValQuot::Input(i.clone()),
        Constraint::Term(VQType::Output, _) => ValQuot::Output(MetaVar(String::from("?"))),
        Constraint::Term(VQType::Call(f), args) => ValQuot::Call(
            f.clone(),
            args.iter().map(|_| MetaVar(String::from("?"))).collect(),
        ),
        Constraint::Term(VQType::CallBuiltin(f), args) => ValQuot::CallOne(
            f.clone(),
            args.iter().map(|_| MetaVar(String::from("?"))).collect(),
        ),
        Constraint::DropTerm(VQType::Call(f), args) => ValQuot::SchedCall(
            f.clone(),
            args.iter().map(|_| MetaVar(String::from("?"))).collect(),
        ),
        Constraint::Term(VQType::Extract(j), _) => ValQuot::Extract(MetaVar(String::from("?")), *j),
        Constraint::Term(VQType::Bop(op), _) => {
            ValQuot::Bop(*op, MetaVar(String::from("?")), MetaVar(String::from("?")))
        }
        Constraint::Term(VQType::Select, _) => ValQuot::Select {
            guard: MetaVar(String::from("?")),
            true_id: MetaVar(String::from("?")),
            false_id: MetaVar(String::from("?")),
        },
        _ => unreachable!(),
    }
}
