use super::*;
use crate::operations::{BinopKind, UnopKind};

use thiserror::Error;

// TODO: something about upcasting? upcastable?
// probably not, we don't need Rust-quality diagnostics here :P
// It would be a good idea to improve these eventually though...
// somehow trace the chain of constant evaluations
// The math example for egg does this, but it's not friendly at all
/// An error produced while constant folding.
#[derive(Debug, Error)]
pub enum CFoldError {
    /// This *should* never happen due to how eclass merging works, we assert that both
    /// constant values are equal. If this error occurs something has gone horribly wrong.
    #[error("constant operands have incompatible types")]
    IncompatibleTypes,
    #[error("constant folding produced an out-of-range value")]
    OutOfRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Constant {
    Bool(bool),

    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),

    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
}
impl Constant {
    /// Converts the constant into a node with the given type ID.
    /// This trusts that the type ID is correct -- i.e. if the constant node is a U32,
    /// the type_id doesn't actually refer to a bool.
    pub fn to_node(&self, type_id: ir::TypeId) -> Node {
        let kind = match self {
            &Self::Bool(value) => OperationKind::ConstantBool { value, type_id },

            // unsigned integers
            &Self::U8(value) => OperationKind::ConstantUnsignedInteger {
                value: value.into(),
                type_id,
            },
            &Self::U16(value) => OperationKind::ConstantUnsignedInteger {
                value: value.into(),
                type_id,
            },
            &Self::U32(value) => OperationKind::ConstantUnsignedInteger {
                value: value.into(),
                type_id,
            },
            &Self::U64(value) => OperationKind::ConstantUnsignedInteger {
                value: value.into(),
                type_id,
            },

            // signed integers
            &Self::I8(value) => OperationKind::ConstantInteger {
                value: value.into(),
                type_id,
            },
            &Self::I16(value) => OperationKind::ConstantInteger {
                value: value.into(),
                type_id,
            },
            &Self::I32(value) => OperationKind::ConstantInteger {
                value: value.into(),
                type_id,
            },
            &Self::I64(value) => OperationKind::ConstantInteger {
                value: value.into(),
                type_id,
            },
        };
        Node {
            kind: NodeKind::Operation { kind },
            deps: Box::new([]),
        }
    }
}

macro_rules! impl_unop {
    ($name:ident, $out0:ident, $tok:tt for {$($variant:ident),*}) => {
        impl Constant {
            pub fn $name(self) -> Result<Constant, CFoldError> {
                match self {
                    $(Self::$variant($out0) => Ok(Self::$variant $tok),)*
                    _ => return Err(CFoldError::IncompatibleTypes)
                }
            }
        }
    };
}
impl_unop!(negate, x, (-x) for {I8, I16, I32, I64});

macro_rules! impl_binop {
    ($name:ident, $out0:ident, $out1:ident, $tok:tt for {$($variant:ident),*}) => {
        impl Constant {
            pub fn $name(self, rhs: Self) -> Result<Constant, CFoldError> {
                match (self, rhs) {
                    $((Self::$variant($out0), Self::$variant($out1)) => Ok(Self::$variant $tok),)*
                    _ => return Err(CFoldError::IncompatibleTypes)
                }
            }
        }
    };
}

impl_binop!(add, a, b, (a.checked_add(b).ok_or(CFoldError::OutOfRange)?)
    for {U8, U16, U32, U64, I8, I16, I32, I64}
);
impl_binop!(sub, a, b, (a.checked_sub(b).ok_or(CFoldError::OutOfRange)?)
    for {U8, U16, U32, U64, I8, I16, I32, I64}
);
impl_binop!(logical_and, a, b, (a && b) for {Bool});
impl_binop!(logical_or, a, b, (a && b) for {Bool});

fn cfold_unop(egraph: &Graph, unop: UnopKind, deps: &[egg::Id]) -> Option<Constant> {
    assert!(deps.len() == 1, "unop has one argument");
    let x = egraph[deps[0]].data.constant?;
    match unop {
        UnopKind::Neg => Some(x.negate().unwrap()),
    }
}

fn cfold_binop(egraph: &Graph, binop: BinopKind, deps: &[egg::Id]) -> Option<Constant> {
    assert!(deps.len() == 2, "binop has two arguments");
    let a = egraph[deps[0]].data.constant?;
    let b = egraph[deps[1]].data.constant?;
    match binop {
        // I'm not sure how to handle errors here... we could silently ignore them by using
        // (a + b).ok() but that seems like a disservice to a user wondering why their
        // value function is buggy, and the errors will almost certainly pop up later
        // in codegen or at runtime. egg doesn't have a way to do fallible analyses AFAIK
        BinopKind::Add => Some(a.add(b).unwrap()),
        BinopKind::Sub => Some(a.sub(b).unwrap()),
        BinopKind::LogicalAnd => Some(a.logical_and(b).unwrap()),
        BinopKind::LogicalOr => Some(a.logical_or(b).unwrap()),
    }
}
impl Node {
    pub fn to_constant(&self, egraph: &Graph) -> Option<Constant> {
        match self.operation()? {
            OperationKind::ConstantBool { value, .. } => Some(Constant::Bool(*value)),
            OperationKind::ConstantInteger { value, .. } => Some(Constant::I64(*value)),
            OperationKind::ConstantUnsignedInteger { value, .. } => Some(Constant::U64(*value)),
            OperationKind::Unop { kind } => cfold_unop(egraph, *kind, &self.deps),
            OperationKind::Binop { kind } => cfold_binop(egraph, *kind, &self.deps),
            _ => None,
        }
    }
}
