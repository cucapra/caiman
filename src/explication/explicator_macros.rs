use crate::ir;
use crate::explication::expir;
use crate::explication::expir::Node;
use crate::explication::Hole;
use paste::paste;

macro_rules! satisfy_argument {
    ($arg1:ident $arg2:ident [$arg_type:ident]) => {
        satisfy_argument!(@ $arg1 $arg2 true)
    };
    ($arg1:ident $arg2:ident $arg_type:ident) => {
        satisfy_argument!(@ $arg1 $arg2 false)
    };
    (@ $arg1:ident $arg2:ident $nested:tt) => {
        match ($arg1, $arg2) {
            // matching each arrangement
            // we want to add stuff if it's currently none
            // if both are vectors, then we also wanna handle that
            (Hole::Empty, Hole::Empty) => Hole::Empty,
            (Hole::Filled(v), Hole::Empty) => Hole::Filled(v.clone()),
            (Hole::Empty, Hole::Filled(v)) => Hole::Filled(v.clone()),
            (Hole::Filled(left), Hole::Filled(right)) => { satisfy_argument!{@ $nested (left right)} }
        }
    };
    (@ false ($left:ident $right:ident)) => {
        assert_eq!($left, $right); // safety check
        Hole::Filled($left.clone())
    };
    (@ true ($left:ident $right:ident)) => {
        assert_eq!($left.len(), $right.len());  // safety check
        let mut result = Vec::new();
        for (index, left_value) in $left.into_iter().enumerate() {
            let right_value = &$right[index];
            result.push(satisfy_argument!(@ left_value right_value false))
        };
        Hole::Filled(result.into_boxed_slice())
    }
}

macro_rules! generate_satisfiers {
    ($($_lang:ident $name:ident ($($arg:ident : $arg_type:tt,)*) -> $_output:ident;)*) => {
        paste! {
            fn satisfy_explication_request(current: &expir::Node, requested: &expir::Node) -> expir::Node {
                match (current, requested) {
                    $((expir::Node::$name { $($arg : [<$arg _one>],)* },
                    expir::Node::$name { $($arg : [<$arg _two>],)* }) => {
                        $(
                            let $arg = satisfy_argument!([<$arg _one>] [<$arg _two>] $arg_type);
                        )*
                        expir::Node::$name { $($arg,)* }
                    })*
                    (current, _) => unreachable!("Trying to request {:?} from {:?}", requested, current)
                }
            }
        }
    };
}

with_operations!(generate_satisfiers);

macro_rules! lower_element {
    ($arg:ident [$_arg_type:ident] $error:ident) => {
        $arg.as_ref().opt().expect(&$error).iter().map(|e| e.clone().opt().expect(&$error)).collect()
    };
    ($arg:ident $_arg_type:ident $error:ident) => {
        $arg.clone().opt().expect(&$error)
    }
}

macro_rules! force_lower_node {
    ($($_lang:ident $name:ident ($($arg:ident : $arg_type:tt,)*) -> $_output:ident;)*) => {
        paste! {
            /*
             * Lowers by rejecting every hole in the node
             */
            pub fn force_lower_node(node : &expir::Node) -> ir::Node {
                let error = format!("Hole not allowed in {:?}", node);
                match node {
                    $(expir::Node::$name { $($arg,)* } => {
                        ir::Node::$name {
                            $($arg : lower_element!($arg $arg_type error),)*
                        }
                    }),*
                }
            }
        }
    };
}

with_operations!(force_lower_node);