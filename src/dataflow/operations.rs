use crate::dataflow::{Error, IrDependent, NodeIndex};
use crate::ir;
use std::collections::HashMap;

macro_rules! lookup_abstract_type {
	([$elem_type:ident]) => { Box<[lookup_abstract_type!($elem_type)]> };
	(Type) => { ir::TypeId };
	(ImmediateI64) => { i64 };
	(ImmediateU64) => { u64 };
	(Index) => { usize };
	(ExternalCpuFunction) => { ir::ExternalCpuFunctionId };
	(ExternalGpuFunction) => { ir::ExternalGpuFunctionId };
	(ValueFunction) => { ir::ValueFunctionId };
	(Operation) => { NodeIndex };
	(Place) => { ir::Place };
}
macro_rules! convert_from_ir {
    ([Operation], $arg:ident, $ir_id:ident) => {
        $arg.iter()
            .map(|id| NodeIndex::from_ir_dependency(*id, IrDependent::Node($ir_id), $ir_id))
            .collect::<Result<Box<[NodeIndex]>, Error>>()?
    };
    (Operation, $arg:ident, $ir_id:ident) => {
        NodeIndex::from_ir_dependency(*$arg, IrDependent::Node($ir_id), $ir_id)?
    };
    ($_arg_type:tt, $arg:ident, $_needed_by:ident) => {
        $arg.clone()
    };
}
macro_rules! convert_to_ir {
    ([Operation], $arg:ident, $map:ident) => {
        $arg.iter().map(|id| $map[id]).collect()
    };
    (Operation, $arg:ident, $map:ident) => {
        $map[$arg]
    };
    ($_arg_type:tt, $arg:ident, $_map:ident) => {
        $arg.clone()
    };
}
macro_rules! for_each_dep {
    ([Operation], $arg:ident, $func:ident) => {
        $arg.iter().for_each(&mut $func)
    };
    (Operation, $arg:ident, $func:ident) => {
        $func($arg)
    };
    ($_arg_type:tt, $_arg:ident, $_func:ident) => {};
}
macro_rules! convert_to_ir_helper {
     // Overload for types with no inputs, which are recordless in the IR
     ($context:ident, $name:ident, ()) => { ir::Node::$name };
     ($context:ident, $name:ident, ( $($arg:ident : $arg_type:tt,)* )) => {
         ir::Node::$name {
             $($arg : convert_to_ir!($arg_type, $arg, $context)),*
         }
     };
}

macro_rules! make_operations {
    // First, we filter the incoming operations with a TT muncher to exclude scheduling operations.
    // This probably won't be necessary after the language split.
    // We then *actually* construct the operation.
    (@filter {} -> {$( $name:ident ( $($arg:ident : $arg_type:tt,)* ); )*}) => {
        pub enum Operation {
            $( $name { $( $arg : lookup_abstract_type!($arg_type) ),* } ),*
        }
        impl Operation {
            pub fn from_ir(node: &ir::Node, ir_id: ir::NodeId) -> Result<Self, Error> {
                match node {
                    $(ir::Node::$name { $($arg),* } => Ok(Self::$name {
                        $($arg : convert_from_ir!($arg_type, $arg, ir_id)),*
                    }),)*
                    _ => panic!("unsupported value node")
                }
            }
            pub fn to_ir(&self, node_map: &HashMap<NodeIndex, ir::NodeId>) -> ir::Node {
                match self {
                    $(Self::$name { $($arg),* } => convert_to_ir_helper!(
                        node_map, $name, ($($arg : $arg_type,)*)
                    )),*
                }
            }
            pub fn for_each_dependency(&self, mut f: impl FnMut(&NodeIndex)) {
                #[allow(unused_variables)]
                match self {
                    $(Self::$name { $($arg),* } => {
                        $( for_each_dep!($arg_type, $arg, f); )*
                    }),*
                }
            }
        }
    };
    (@filter
        {
            scheduling $_name:ident ($($_arg:ident : $_arg_type:tt,)*) -> $_output:ident;
            $($rest:tt)*
        }
        -> {$($filtered:tt)*}
    ) => {
        make_operations! { @filter {$($rest)*} -> {$($filtered)*} }
    };
    (@filter
        {
            $_lang:ident $name:ident ($($arg:ident : $arg_type:tt,)*) -> $_output:ident;
            $($rest:tt)*
        }
        -> {$($filtered:tt)*}
    ) => {
        make_operations! { @filter {$($rest)*} -> {$($filtered)* $name ($($arg: $arg_type,)*);} }
    };
    ($($lang:ident $name:ident ($($arg:ident : $arg_type:tt,)*) -> $output:ident;)*) => {
        make_operations! { @filter {$($lang $name ($($arg : $arg_type,)*) -> $output;)*} -> {} }
    };
}

with_operations!(make_operations);
