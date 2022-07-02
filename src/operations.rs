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
