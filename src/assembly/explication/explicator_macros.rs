use crate::assembly::ast;
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
            (None, None) => None,
            (Some(v), None) => Some(v),
            (None, Some(v)) => Some(v.clone()),
            (Some(left), Some(right)) => { satisfy_argument!{@ $nested (left right)} }
        }
    };
    (@ false ($left:ident $right:ident)) => {
        assert_eq!(&$left, $right); // safety check
        Some($left)
    };
    (@ true ($left:ident $right:ident)) => {
        assert_eq!($left.len(), $right.len());  // safety check
        let mut result = Vec::new();
        for (index, left_value) in $left.into_iter().enumerate() {
            let right_value = &$right[index];
            result.push(satisfy_argument!(@ left_value right_value false))
        };
        Some(result)
    }
}

macro_rules! generate_satisfiers {
    ($($_lang:ident $name:ident ($($arg:ident : $arg_type:tt,)*) -> $_output:ident;)*) => {
        paste! {
            fn satisfy_explication_request(current: ast::Node, requested: &ast::Node) -> ast::Node {
                match (current, requested) {
                    $((ast::Node::$name { $($arg : [<$arg _one>],)* },
                    ast::Node::$name { $($arg : [<$arg _two>],)* }) => {
                        $(
                            let $arg = satisfy_argument!([<$arg _one>] [<$arg _two>] $arg_type);
                        )*
                        ast::Node::$name { $($arg,)* }
                    })*
                    (current, _) => unreachable!("Trying to request {:?} from {:?}", requested, current)
                }
            }
        }
    };
}

with_operations!(generate_satisfiers);
