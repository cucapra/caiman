use crate::parse::ast::{DataType, FloatSize, IntSize};

use super::unification::{Constraint, Env, Kind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CDataType {
    Num,
    Int,
    Float,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ADataType {
    I32,
    I64,
    F64,
    Bool,
    BufferSpace,
    Event,
}

impl Kind for CDataType {}
impl Kind for ADataType {}

/// A high-level data type constraint. Holes in a data type
/// constraint are universally quantified. To get multiple
/// copies of the same constraint, clone the constraint
/// after instantiating it.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
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
        }
    }
}

const DEFAULT_INT_SIZE: IntSize = IntSize::I64;
const DEFAULT_FLOAT_SIZE: FloatSize = FloatSize::F64;

impl TryFrom<DTypeConstraint> for DataType {
    type Error = ();
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
            DTypeConstraint::Any => Err(()),
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
                            Constraint::Term(..) => unreachable!(),
                        }
                    }
                    Constraint::Term(CDataType::Float, mut v) => {
                        let d = v.swap_remove(0);
                        match d {
                            Constraint::Atom(x) => Ok(Self::Float(Some(x.try_into()?))),
                            Constraint::Var(_) => Ok(Self::Float(None)),
                            Constraint::Term(..) => unreachable!(),
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Constraint::Var(_) => Ok(Self::Any),
            _ => todo!(),
        }
    }
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
            _ => todo!(),
        }
    }
}
