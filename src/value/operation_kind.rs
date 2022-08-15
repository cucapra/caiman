use crate::ir;
use crate::operations::{BinopKind, UnopKind};
use crate::value::from_op::{Attribs, FromOpError};

macro_rules! _field_type {
	([$elem_type:ident]) => { Box<[_field_type!($elem_type)]> };
	(Type) => { ir::TypeId };
    (ImmediateBool) => { bool };
	(ImmediateI64) => { i64 };
	(ImmediateU64) => { u64 };
	(Index) => { usize };
	(ExternalCpuFunction) => { ir::ExternalCpuFunctionId };
	(ExternalGpuFunction) => { ir::ExternalGpuFunctionId };
	(ValueFunction) => { ir::ValueFunctionId };
	(Operation) => { somethingiswrong };
	(Place) => { ir::Place };
    (Unop) => { UnopKind };
    (Binop) => { BinopKind };
}
macro_rules! _short_name {
    (None) => {
        "none"
    };
    (ExtractResult) => {
        "extract"
    };
    (ConstantInteger) => {
        "csi"
    };
    (ConstantUnsignedInteger) => {
        "cui"
    };
    (ConstantBool) => {
        "cb"
    };
    (CallValueFunction) => {
        "call_val"
    };
    (CallExternalCpu) => {
        "call_cpu"
    };
    (CallExternalGpuCompute) => {
        "call_gpu"
    };
    (Unop) => {
        "unop"
    };
    (Binop) => {
        "binop"
    };
}
macro_rules! _mok_impl {
    ($({
        name: $name:ident,
        deps: { $($dep:ident : $dep_type:tt,)* },
        args: { $($arg:ident : $arg_type:tt,)* }
    },)*) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum OperationKind {
            $( $name {
                $($arg: _field_type!($arg_type)),*
            } ),*
        }

        impl OperationKind {
            pub(super) fn from_ir_node(node: &ir::Node) -> Self {
                match node {
                    $(
                        ir::Node::$name {$($arg,)* ..} => Self::$name { $($arg : $arg.clone()),* },
                    )*
                    _ => panic!("invalid value node for operation conversion")
                }
            }
            #[allow(unused_mut)]
            pub(super) fn num_deps(&self) -> usize{
                match self {
                    $( Self::$name { .. } => { let mut count = 0; $(count += 1; let $arg = (); )* count} )*
                }
            }
            pub(super) fn from_description(kind: &str, attribs: &mut Attribs) -> Result<Self, FromOpError> {
                match kind {
                   $( _short_name!($name) => Ok(Self::$name {
                            $($arg : attribs.get(stringify!($arg))?),*
                    }),)*
                    // convenience overrides
                    "neg" => Ok(Self::Unop{kind: UnopKind::Neg}),
                    "+" => Ok(Self::Binop {kind: BinopKind::Add}),
                    "-" => Ok(Self::Binop {kind: BinopKind::Sub}),
                    "&&" => Ok(Self::Binop {kind: BinopKind::LogicalAnd}),
                    "||" => Ok(Self::Binop {kind: BinopKind::LogicalOr}),
                    // fallback
                    _ => Err(FromOpError::UnknownOp(kind.into()))
                }
            }
        }
    };
}
macro_rules! _mok_arg_step {
    // Base case - jump back to steps
    (
        in_args: {},
        out_deps: { $($dep:ident : $dep_type:tt,)* },
        out_args: { $($arg:ident : $arg_type:tt,)* },
        name: $name:ident,
        remaining: { $($remaining:tt)* },
        processed: { $($processed:tt)* }
    ) => {
        _mok_op_step!(
            remaining: { $($remaining)* },
            processed: {
                $($processed)*
                {
                    name: $name,
                    deps: { $($dep : $dep_type,)* },
                    args: { $($arg : $arg_type,)* }
                },
            }
        );
    };
    // Recursive case - single dep
    (
        in_args: { $arg:ident : Operation, $($in_args:tt)* },
        out_deps: { $($out_deps:tt)* },
        out_args: { $($out_args:tt)* },
        name: $name:ident,
        remaining: { $($remaining:tt)* },
        processed: { $($processed:tt)* }
    ) => {
        _mok_arg_step!(
            in_args: { $($in_args)* },
            out_deps: {
                $($out_deps)*
                $arg : Operation,
            },
            out_args: { $($out_args)* },
            name: $name,
            remaining: { $($remaining)* },
            processed: { $($processed)* }
        );
    };
    // Recursive case - dep array
    (
        in_args: { $arg:ident : [Operation], $($in_args:tt)* },
        out_deps: { $($out_deps:tt)* },
        out_args: { $($out_args:tt)* },
        name: $name:ident,
        remaining: { $($remaining:tt)* },
        processed: { $($processed:tt)* }
    ) => {
        _mok_arg_step!(
            in_args: { $($in_args)* },
            out_deps: {
                $($out_deps)*
                $arg : [Operation],
            },
            out_args: { $($out_args)* },
            name: $name,
            remaining: { $($remaining)* },
            processed: { $($processed)* }
        );
    };
    // Recursive case - non-deps
    (
        in_args: { $arg:ident : $arg_type:tt, $($in_args:tt)* },
        out_deps: { $($out_deps:tt)* },
        out_args: { $($out_args:tt)* },
        name: $name:ident,
        remaining: { $($remaining:tt)* },
        processed: { $($processed:tt)* }
    ) => {
        _mok_arg_step!(
            in_args: { $($in_args)* },
            out_deps: { $($out_deps)* },
            out_args: {
                $($out_args)*
                $arg : $arg_type,
            },
            name: $name,
            remaining: { $($remaining)* },
            processed: { $($processed)* }
        );
    };
}
macro_rules! _mok_op_step {
    // Base case - dispatch to impl
    (
        remaining: {},
        processed: { $($processed:tt)* }
    ) => {
        _mok_impl!($($processed)*);
    };

    // Filter out scheduling nodes
    (
        remaining: { scheduling $_n:ident ( $($_a:tt)* ) -> $_o:ident; $($remaining:tt)* },
        processed: { $($processed:tt)* }
    ) => {
        _mok_op_step!(remaining: { $($remaining)* }, processed: { $($processed)* });
    };

    // Filter out Phi nodes (we use param nodes instead)
    (
        remaining: { $_l:ident Phi ( $($_a:tt)* ) -> $_o:ident; $($remaining:tt)* },
        processed: { $($processed:tt)* }
    ) => {
        _mok_op_step!(remaining: { $($remaining)* }, processed: { $($processed)* });
    };

    // "Main" op step case
    (
        remaining: {
            $_l:ident $name:ident ( $($arg:ident : $arg_type:tt,)* ) -> $_o:ident;
            $($remaining:tt)*
        },
        processed: {$($processed:tt)*}
    ) => {
        _mok_arg_step!(
            in_args: { $($arg : $arg_type,)* },
            out_deps: {},
            out_args: {},
            name: $name,
            remaining: { $($remaining)* },
            processed: { $($processed)* }
        );
    };
}
macro_rules! make_operation_kinds {
    ( $($input:tt)* ) => {
        _mok_op_step!(
            remaining: { $($input)* },
            processed: {}
        );
    }
}

with_operations!(make_operation_kinds);