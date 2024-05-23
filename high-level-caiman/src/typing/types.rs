use std::{
    collections::{BTreeMap, HashSet},
    vec,
};

use crate::parse::ast::{Binop, DataType, FloatSize, IntSize};

use super::unification::{Constraint, Env, Kind};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordConstraint {
    pub fields: BTreeMap<String, DTypeConstraint>,
    /// whether or not the constraint is contravariant. If true, then
    /// something that adheres to this constraint cannot be a subtype,
    /// but can be a supertype.
    pub is_contravariant: bool,
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
        /// record of all readable variables (map_read)
        read: RecordConstraint,
        /// record of all writable variables (copy_dst)
        write: RecordConstraint,
    },
    SpecEncoder,
    SpecFence,
    Var(String),
}

impl DTypeConstraint {
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
            Self::Record(r) => Self::Record(RecordConstraint {
                fields: r
                    .fields
                    .into_iter()
                    .map(|(k, v)| (k, v.into_subtypeable()))
                    .collect(),
                is_contravariant: r.is_contravariant,
            }),
            Self::RemoteObj { all, read, write } => Self::RemoteObj {
                all: RecordConstraint {
                    fields: all
                        .fields
                        .into_iter()
                        .map(|(k, v)| (k, v.into_subtypeable()))
                        .collect(),
                    is_contravariant: false,
                },
                read: RecordConstraint {
                    fields: read
                        .fields
                        .into_iter()
                        .map(|(k, v)| (k, v.into_subtypeable()))
                        .collect(),
                    is_contravariant: false,
                },
                write: RecordConstraint {
                    fields: write
                        .fields
                        .into_iter()
                        .map(|(k, v)| (k, v.into_subtypeable()))
                        .collect(),
                    is_contravariant: false,
                },
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
        let mut mp = BTreeMap::new();
        for (k, v) in r.fields {
            mp.insert(k, v.instantiate(env));
        }
        Constraint::DynamicTerm(CDataType::Record, mp, r.is_contravariant)
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
            Self::RemoteObj { all, read, write } => Constraint::Term(
                CDataType::RemoteObj,
                vec![
                    Self::instantiate_record(all, env),
                    Self::instantiate_record(read, env),
                    Self::instantiate_record(write, env),
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
            DTypeConstraint::Any | DTypeConstraint::Var(_) => Err(()),
            DTypeConstraint::Ref(x) => Ok(Self::Ref(Box::new(Self::try_from(
                DTypeConstraint::try_from(x).map_err(|_| ())?,
            )?))),
            DTypeConstraint::RefN(x) => Ok(Self::Ref(Box::new(Self::try_from(*x)?))),
            // allow abstract encoders and fences
            // DTypeConstraint::Encoder(typ) => Ok(Self::Encoder(Some(Box::new(Self::try_from(
            //     DTypeConstraint::try_from(typ).map_err(|_| ())?,
            // )?)))),
            // DTypeConstraint::Fence(vars) => Ok(Self::Fence(Some(Box::new(Self::try_from(
            //     DTypeConstraint::try_from(vars).map_err(|_| ())?,
            // )?)))),
            DTypeConstraint::Record(r) => {
                let mut mp = BTreeMap::new();
                for (k, v) in r.fields {
                    mp.insert(k, Self::try_from(v)?);
                }
                Ok(Self::Record(mp))
            }
            DTypeConstraint::RemoteObj { all, read, write } => {
                let mut mp = BTreeMap::new();
                for (k, v) in all.fields {
                    mp.insert(k, Self::try_from(v)?);
                }
                let read = read.fields.into_keys().collect();
                let write = write.fields.into_keys().collect();
                Ok(Self::RemoteObj {
                    all: mp,
                    read,
                    write,
                })
            }
            DTypeConstraint::Encoder(typ) => {
                Ok(Self::Encoder(Some(Box::new(Self::try_from(*typ)?))))
            }
            DTypeConstraint::Fence(vars) => Ok(Self::Fence(Some(Box::new(Self::try_from(*vars)?)))),
            DTypeConstraint::SpecEncoder => Ok(Self::Encoder(None)),
            DTypeConstraint::SpecFence => Ok(Self::Fence(None)),
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
                            Constraint::Term(..) | Constraint::DynamicTerm(..) => unreachable!(),
                        }
                    }
                    Constraint::Term(CDataType::Float, mut v) => {
                        let d = v.swap_remove(0);
                        match d {
                            Constraint::Atom(x) => Ok(Self::Float(Some(x.try_into()?))),
                            Constraint::Var(_) => Ok(Self::Float(None)),
                            Constraint::Term(..) | Constraint::DynamicTerm(..) => unreachable!(),
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
            Constraint::DynamicTerm(CDataType::Record, fields, is_contravariant) => {
                let mut mp = BTreeMap::new();
                for (k, v) in fields {
                    mp.insert(k, Self::try_from(v)?);
                }
                Ok(Self::Record(RecordConstraint {
                    fields: mp,
                    is_contravariant,
                }))
            }
            Constraint::Term(CDataType::RemoteObj, mut v) => {
                assert_eq!(
                    v.len(),
                    3,
                    "Remote obj constraint should have exactly two children"
                );
                let write = Self::try_from(v.pop().unwrap())?;
                let read = Self::try_from(v.pop().unwrap())?;
                let all = Self::try_from(v.pop().unwrap())?;
                if let (Self::Record(all), Self::Record(read), Self::Record(write)) =
                    (all, read, write)
                {
                    Ok(Self::RemoteObj { all, read, write })
                } else {
                    Err("RemoteObj constraint should have record children".to_string())
                }
            }
            Constraint::Atom(ADataType::SpecEncoder) => Ok(Self::SpecEncoder),
            Constraint::Atom(ADataType::SpecFence) => Ok(Self::SpecFence),
            _ => todo!(),
        }
    }
}

/// Converts a map of string to data types to a map of string to data type constraints
fn record_dtypes_to_constraints(
    fields: BTreeMap<String, DataType>,
) -> BTreeMap<String, DTypeConstraint> {
    let mut mp = BTreeMap::new();
    for (k, v) in fields {
        mp.insert(k, DTypeConstraint::from(v));
    }
    mp
}

/// Converts a set of strings to a map of string to the Any data type constraint
fn set_dtypes_to_constraints(set: HashSet<String>) -> BTreeMap<String, DTypeConstraint> {
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
            DataType::Record(fields) => Self::Record(RecordConstraint {
                fields: record_dtypes_to_constraints(fields),
                // when annotated, we cannot deduce a subtype of the annotation
                is_contravariant: true,
            }),
            DataType::RemoteObj { all, read, write } => Self::RemoteObj {
                // annotations cannot be deduced to be lower types in the lattice then the annotation
                all: RecordConstraint {
                    fields: record_dtypes_to_constraints(all),
                    is_contravariant: true,
                },
                read: RecordConstraint {
                    fields: set_dtypes_to_constraints(read),
                    is_contravariant: true,
                },
                write: RecordConstraint {
                    fields: set_dtypes_to_constraints(write),
                    is_contravariant: true,
                },
            },
            _ => todo!(),
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
    pub fn new_class_name(s: &String) -> Self {
        Self(format!("${s}"))
    }

    /// Creates a type variable name
    #[must_use]
    pub fn new_var_name(s: &String) -> Self {
        Self(s.to_string())
    }

    /// Returns the string representation of the metavariable
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn into_string(self) -> String {
        self.0
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
    Call(String, Vec<MetaVar>),
    Extract(MetaVar, usize),
    Bop(Binop, MetaVar, MetaVar),
    Select {
        guard: MetaVar,
        true_id: MetaVar,
        false_id: MetaVar,
    },
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
            (Self::Call(f1, args1), Self::Call(f2, args2)) => {
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
            ValQuot::Call(f, _) => Self::Call(f.clone()),
            ValQuot::Extract(_, j) => Self::Extract(*j),
            ValQuot::Bop(op, _, _) => Self::Bop(*op),
            ValQuot::Select { .. } => Self::Select,
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
