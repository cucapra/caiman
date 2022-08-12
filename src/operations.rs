use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("unknown unary operation \"{0}\"")]
pub struct UnknownUnop(String);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UnopKind {
    // Arithmetic
    Neg,
    // TODO
}
impl FromStr for UnopKind {
    type Err = UnknownUnop;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "neg" => Ok(Self::Neg),
            _ => Err(UnknownUnop(s.into())),
        }
    }
}

#[derive(Debug, Error)]
#[error("unknown binary operation \"{0}\"")]
pub struct UnknownBinop(String);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BinopKind {
    // Arithmetic
    Add,
    Sub,
    // Logical
    LogicalAnd,
    LogicalOr,
    // TODO
}
impl FromStr for BinopKind {
    type Err = UnknownBinop;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "add" => Ok(Self::Add),
            "sub" => Ok(Self::Sub),
            _ => Err(UnknownBinop(s.into())),
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/generated/with_operations.rs"));

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
