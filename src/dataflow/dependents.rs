use crate::dataflow::{Error, Graph, IrDependent, NodeIndex, ValueDependent};
use crate::ir;
use crate::operations::{BinopKind, UnopKind};
use std::collections::HashMap;
use std::ops::AddAssign;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ConstSumAcc {
    Int(i64, ir::TypeId),
    Uint(u64, ir::TypeId),
}
impl From<ConstSumAcc> for Node {
    fn from(acc: ConstSumAcc) -> Self {
        match acc {
            ConstSumAcc::Int(value, type_id) => {
                Self::ConstantInteger(ConstantInteger { value, type_id })
            }
            ConstSumAcc::Uint(value, type_id) => {
                Self::ConstantUnsignedInteger(ConstantUnsignedInteger { value, type_id })
            }
        }
    }
}

/// TODO: This has pretty bad perf characterstics because it's always reallocating everything
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Sum {
    /// Invariant: either arguments is non-empty, or const_acc is Some
    arguments: Vec<NodeIndex>,
    const_acc: Option<ConstSumAcc>,
}
impl Sum {
    pub fn new() -> Self {
        Self {
            arguments: Vec::with_capacity(8),
            const_acc: None,
        }
    }
    pub fn add_arg(&mut self, graph: &Graph, arg: NodeIndex) {
        match (graph.node(arg), &mut self.const_acc) {
            (Node::Sum(other), None) => {
                self.arguments.extend(other.arguments.iter());
                self.arguments.sort_unstable();
                self.const_acc = other.const_acc;
            }
            (
                Node::Sum(Sum {
                    arguments: other_arguments,
                    const_acc: None,
                }),
                _,
            ) => {
                self.arguments.extend(other_arguments.iter());
                self.arguments.sort_unstable();
            }
            (
                Node::Sum(Sum {
                    arguments: other_arguments,
                    const_acc: Some(ConstSumAcc::Int(other_value, other_type_id)),
                }),
                Some(ConstSumAcc::Int(value, type_id)),
            ) if other_type_id == type_id => {
                self.arguments.extend(other_arguments.iter());
                self.arguments.sort_unstable();
                value.add_assign(other_value);
            }
            (
                Node::Sum(Sum {
                    arguments: other_arguments,
                    const_acc: Some(ConstSumAcc::Uint(other_value, other_type_id)),
                }),
                Some(ConstSumAcc::Uint(value, type_id)),
            ) if other_type_id == type_id => {
                self.arguments.extend(other_arguments.iter());
                self.arguments.sort_unstable();
                value.add_assign(other_value);
            }
            (Node::ConstantInteger(ci), None) => {
                self.const_acc = Some(ConstSumAcc::Int(ci.value, ci.type_id))
            }
            (Node::ConstantInteger(ci), Some(ConstSumAcc::Int(value, type_id)))
                if *type_id == ci.type_id =>
            {
                value.add_assign(ci.value)
            }
            (Node::ConstantUnsignedInteger(cui), None) => {
                self.const_acc = Some(ConstSumAcc::Uint(cui.value, cui.type_id))
            }
            (Node::ConstantUnsignedInteger(cui), Some(ConstSumAcc::Uint(value, type_id)))
                if *type_id == cui.type_id =>
            {
                value.add_assign(cui.value)
            }
            _ => {
                self.arguments.push(arg);
                self.arguments.sort_unstable();
            }
        }
    }
    pub fn reduce(&self, graph: &mut Graph) -> Node {
        if let Some((first, rest)) = self.arguments.split_first() {
            let mut cur = *first;
            for arg in rest {
                let node = Node::Binop(Binop {
                    kind: BinopKind::Add,
                    arg0: cur,
                    arg1: *arg,
                });
                cur = graph.add_node(node);
            }
            match self.const_acc {
                core::option::Option::None
                | Some(ConstSumAcc::Int(0, _))
                | Some(ConstSumAcc::Uint(0, _)) => (),

                Some(other) => {
                    let const_acc_index = graph.add_node(other.into());
                    let node = Node::Binop(Binop {
                        kind: BinopKind::Add,
                        arg0: cur,
                        arg1: const_acc_index,
                    });
                    cur = graph.add_node(node);
                }
            }
            // a bit weird, but whatever
            graph.node(cur).clone()
        } else {
            self.const_acc
                .map(Node::from)
                .unwrap_or(Node::None(self::None {}))
        }
    }
}
impl ValueDependent for Sum {
    fn for_each_dependency(&self, closure: impl FnMut(&NodeIndex))
    where
        Self: Sized,
    {
        self.arguments.iter().for_each(closure)
    }
    fn map_dependencies(&mut self, closure: impl Fn(NodeIndex) -> NodeIndex)
    where
        Self: Sized,
    {
        self.arguments
            .iter_mut()
            .for_each(|dep| *dep = closure(*dep));
        self.arguments.sort_unstable();
    }
}
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
    (Unop) => { UnopKind };
    (Binop) => { BinopKind };
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
                $( pub $arg : _lookup_type!($arg_type) ),*
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
            $( $name($name), )*
            Sum(Sum)
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
                    ),)*
                    _ => panic!("support node still live")
                }
            }
        }
        impl ValueDependent for Node {
            fn for_each_dependency(&self, closure: impl FnMut(&NodeIndex)) {
                match self {
                    $( Self::$name(inner) => inner.for_each_dependency(closure),)*
                    Self::Sum(inner) => inner.for_each_dependency(closure)
                }
            }
            fn map_dependencies(&mut self, closure: impl Fn(NodeIndex) -> NodeIndex) {
                match self {
                    $( Self::$name(inner) => inner.map_dependencies(closure),)*
                    Self::Sum(inner) => inner.map_dependencies(closure)
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
                $( pub $arg : _lookup_type!($arg_type) ),*
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
        }
    };
}

with_tails!(make_tails);
