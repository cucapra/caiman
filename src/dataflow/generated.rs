use crate::dataflow::{Error, IrDependent, NodeIndex, ValueDependent};
use crate::ir;
use std::collections::HashMap;

macro_rules! _lookup_type {
	([$elem_type:ident]) => { Box<[_lookup_type!($elem_type)]> };
	(Type) => { ir::TypeId };
	(ImmediateI64) => { i64 };
	(ImmediateU64) => { u64 };
	(Index) => { usize };
	(ExternalCpuFunction) => { ir::ExternalCpuFunctionId };
	(ExternalGpuFunction) => { ir::ExternalGpuFunctionId };
	(ValueFunction) => { ir::ValueFunctionId };
	(Operation) => { NodeIndex };
	(Place) => { ir::Place };
    (FuncletId) => { ir::FuncletId };
}
macro_rules! _from_ir_inner {
    ([Operation], $arg:expr, $dependent:expr, $sentinel:expr) => {
        $arg.iter()
            .map(|id| NodeIndex::from_ir_dependency(*id, $dependent, $sentinel))
            .collect::<Result<Box<[NodeIndex]>, Error>>()?
    };
    (Operation, $arg:expr, $dependent:expr, $sentinel:expr) => {
        NodeIndex::from_ir_dependency(*$arg, $dependent, $sentinel)?
    };
    ($_arg_type:tt, $arg:expr, $dependent:expr, $sentinel:expr) => {
        $arg.clone()
    };
}
macro_rules! _from_ir {
    ($arg_type:tt, $arg:expr, $ir_id:expr) => {
        _from_ir_inner!($arg_type, $arg, IrDependent::Node($ir_id), $ir_id)
    };
}
macro_rules! _from_ir_tail {
    ($arg_type:tt, $arg:expr, $sentinel:expr) => {
        _from_ir_inner!($arg_type, $arg, IrDependent::Tail, $sentinel)
    };
}
macro_rules! _to_ir {
    ([Operation], $arg:expr, $map:expr) => {
        $arg.iter().map(|id| $map[id]).collect()
    };
    (Operation, $arg:expr, $map:expr) => {
        $map[$arg]
    };
    ($_arg_type:tt, $arg:expr, $_map:expr) => {
        $arg.clone()
    };
}
macro_rules! _to_ir_helper {
     // Overload for types with no inputs, which are recordless in the IR
     ($context:ident, $name:ident, ()) => { ir::Node::$name };
     ($context:ident, $name:ident, ( $($arg:ident : $arg_type:tt,)* )) => {
         ir::Node::$name {
             $($arg : _to_ir!($arg_type, $arg, $context)),*
         }
     };
}
macro_rules! _for_each_dependency {
    ([Operation], $arg:expr, $func:ident) => {
        $arg.iter().for_each(&mut $func)
    };
    (Operation, $arg:expr, $func:ident) => {
        $func($arg)
    };
    ($_arg_type:tt, $_arg:expr, $_func:ident) => {};
}
macro_rules! _map_dependencies {
    ([Operation], $arg:expr, $func:ident) => {
        $arg.iter_mut().for_each(|index| *index = $func(*index))
    };
    (Operation, $arg:expr, $func:ident) => {
        *$arg = $func(*$arg)
    };
    ($_arg_type:tt, $_arg:expr, $_func:ident) => {};
}

macro_rules! make_operations {
    // First, we filter the incoming operations with a TT muncher to exclude scheduling operations.
    // This probably won't be necessary after the language split.
    // We then *actually* construct the operations.
    (@filter {} -> {$( $name:ident ( $($arg:ident : $arg_type:tt,)* ); )*}) => {
        $(
            #[derive(Debug, Clone, Hash, PartialEq, Eq)]
            pub struct $name {
                $( $arg : _lookup_type!($arg_type) ),*
            }
            #[allow(unused_variables, unused_mut)]
            impl ValueDependent for $name {
                fn for_each_dependency(&self, mut closure: impl FnMut(&NodeIndex)) {
                    $( _for_each_dependency!( $arg_type, &self.$arg, closure ); )*
                }
                fn map_dependencies(&mut self, closure: impl Fn(NodeIndex) -> NodeIndex) {
                    $( _map_dependencies!( $arg_type, &mut self.$arg, closure ); )*
                }
            }
        )*
        #[derive(Debug, Clone, Hash, PartialEq, Eq)]
        pub enum Node {
            $( $name($name) ),*
        }
        impl Node {
            pub fn from_ir(node: &ir::Node, ir_id: ir::NodeId) -> Result<Self, Error> {
                match node {
                    $(ir::Node::$name { $($arg),* } => Ok(Self::$name($name {
                        $($arg : _from_ir!($arg_type, $arg, ir_id)),*
                    })),)*
                    _ => panic!("unsupported value node")
                }
            }
            pub fn to_ir(&self, node_map: &HashMap<NodeIndex, ir::NodeId>) -> ir::Node {
                match self {
                    $(Self::$name($name { $($arg),* }) => _to_ir_helper!(
                        node_map, $name, ($($arg : $arg_type,)*)
                    )),*
                }
            }
        }
        impl ValueDependent for Node {
            fn for_each_dependency(&self, closure: impl FnMut(&NodeIndex)) {
                match self {
                    $( Self::$name(inner) => inner.for_each_dependency(closure)),*
                }
            }
            fn map_dependencies(&mut self, closure: impl Fn(NodeIndex) -> NodeIndex) {
                match self {
                    $( Self::$name(inner) => inner.map_dependencies(closure)),*
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

macro_rules! make_tails {
    ($($name:ident ($($arg:ident : $arg_type:tt),*);)*) => {
        $(
            #[derive(Debug, PartialEq)]
            pub struct $name {
                $( $arg : _lookup_type!($arg_type) ),*
            }
            #[allow(unused_variables, unused_mut)]
            impl ValueDependent for $name {
                fn for_each_dependency(&self, mut closure: impl FnMut(&NodeIndex)) {
                    $( _for_each_dependency!( $arg_type, &self.$arg, closure ); )*
                }
                fn map_dependencies(&mut self, closure: impl Fn(NodeIndex) -> NodeIndex) {
                    $( _map_dependencies!( $arg_type, &mut self.$arg, closure ); )*
                }
            }
        )*
        #[derive(Debug, PartialEq)]
        pub enum Tail {
            $( $name($name) ),*
        }
        impl Tail {
            pub fn from_ir(tail: &ir::TailEdge, sentinel: ir::NodeId) -> Result<Self, Error> {
                match tail {
                    $(ir::TailEdge::$name { $($arg),* } => Ok(Self::$name($name {
                        $($arg : _from_ir_tail!($arg_type, $arg, sentinel)),*
                    }))),*
                }
            }
            pub fn to_ir(&self, node_map: &HashMap<NodeIndex, ir::NodeId>) -> ir::TailEdge {
                match self {
                    $(Self::$name($name { $($arg),* }) => ir::TailEdge::$name {
                        $($arg : _to_ir!($arg_type, $arg, node_map)),*
                    }),*
                }
            }
        }
        impl ValueDependent for Tail {
            fn for_each_dependency(&self, closure: impl FnMut(&NodeIndex)) {
                match self {
                    $( Self::$name(inner) => inner.for_each_dependency(closure)),*
                }
            }
            fn map_dependencies(&mut self, closure: impl Fn(NodeIndex) -> NodeIndex) {
                match self {
                    $( Self::$name(inner) => inner.map_dependencies(closure)),*
                }
            }
        }
    };
}

// TODO: In the future, we might want to generate this from caiman-spec/build.rs
macro_rules! with_tails {
    ($macro:ident) => {
        $macro! {
            Return (return_values: [Operation]);
            Yield (
                funclet_ids: [FuncletId],
                captured_arguments: [Operation],
                return_values: [Operation]
            );
        }
    };
}

with_tails!(make_tails);
