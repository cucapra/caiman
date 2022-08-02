use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UnopKind {
    // Arithmetic
    Negate,
    // TODO
}
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BinopKind {
    // Arithmetic
    Add,
    Sub,
    // TODO
}
include!(concat!(env!("OUT_DIR"), "/generated/with_operations.rs"));

/// Convienence macro.
#[macro_export]
macro_rules! filter_scheduling {
    // Base Case
    (callback: $callback:ident, input: {}, output: {$($processed:tt)*}) => {
        $callback! {$($processed)*}
    };
    // Recursive Case (scheduling)
    (
        callback: $callback:ident,
        input: {
            scheduling $name:ident ($($arg:ident : $arg_type:tt,)*) -> $output:ident;
            $($remaining:tt)*
        },
        output: {$($processed:tt)*}
    ) => {
        filter_scheduling! {
            callback: $callback,
            input: {$($remaining)*},
            output: {$($processed)*}
        }
    };
    // Recursive Case (non-scheduling)
    (
        callback: $callback:ident,
        input: {
            $lang:ident $name:ident ($($arg:ident : $arg_type:tt,)*) -> $output:ident;
            $($remaining:tt)*
        },
        output: {$($processed:tt)*}
    ) => {
        filter_scheduling! {
            callback: $callback,
            input: {$($remaining)*},
            output: {
                $($processed)*
                $name ( $($arg : $arg_type,)* ) -> $output;
            }
        }
    };
}
#[cfg(test)]
mod tests {
    macro_rules! example {
        ($($lang:ident $name:ident ($($arg:ident : $arg_type:tt,)*) -> $output:ident;)*) => {
            println!("operations:");
            $(
                println!("{}: fn {}(", stringify!($lang), stringify!($name));
                $(
                    println!("{}: {},", stringify!($arg), stringify!($arg_type));
                )*
                println!(") -> {}", stringify!($output));
            )*
        };
    }

    #[test]
    fn test_macro() {
        with_operations!(example);
    }
}
