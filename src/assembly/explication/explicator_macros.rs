use crate::assembly::ast;
use paste::paste;

// fn satisfy_explication_request(current: ast::Node, requested: &ast::Node) -> ast::Node {
//     match (current, requested) {
//         (
//             ast::Node::FulfillCaptures {
//                 continuation: continuation_one,
//                 haves: haves_one,
//                 needs: needs_one,
//             },
//             ast::Node::FulfillCaptures {
//                 continuation: continuation_two,
//                 haves: haves_two,
//                 needs: needs_two,
//             },
//         ) => {
//             let continuation = match (continuation_one, continuation_two) {
//                 (None, None) => None,
//                 (Some(v), None) => Some(v),
//                 (None, Some(v)) => Some(v.clone()),
//                 (Some(left), Some(right)) => {
//                     Some(left)
//                 }
//             };
//             let haves = match (haves_one, haves_two) {
//                 (None, None) => None,
//                 (Some(v), None) => Some(v),
//                 (None, Some(v)) => Some(v.clone()),
//                 (Some(left), Some(right)) => {
//                     let mut result = Vec::new();
//                     for (index, left_value) in left.into_iter().enumerate() {
//                         let right_value = &right[index];
//                         result.push(match (left_value, right_value) {
//                             (None, None) => None,
//                             (Some(v), None) => Some(v),
//                             (None, Some(v)) => Some(v.clone()),
//                             (Some(left), Some(right)) => {
//                                 match (&(&left), &right) {
//                                     (left_val, right_val) => {
//                                     }
//                                 };
//                                 Some(left)
//                             }
//                         })
//                     }
//                     result
//                 }
//             };
//             let needs = match (needs_one, needs_two) {
//                 (None, None) => None,
//                 (Some(v), None) => Some(v),
//                 (None, Some(v)) => Some(v.clone()),
//                 (Some(left), Some(right)) => {
//                     let result = Vec::new();
//                     for index in 0..left.len() {
//                         let left_value = &left[index];
//                         let right_value = &right[index];
//                         result.push(match (left_value, right_value) {
//                             (None, None) => None,
//                             (Some(v), None) => Some(v),
//                             (None, Some(v)) => Some(v.clone()),
//                             (Some(left), Some(right)) => {
//                                 Some(left)
//                             }
//                         })
//                     }
//                     result
//                 }
//             };
//             ast::Node::FulfillCaptures {
//                 continuation,
//                 haves,
//                 needs,
//             }
//         }
//         _ => todo!()
//     }
// }

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
