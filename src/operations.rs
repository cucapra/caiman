macro_rules! with_operations {
    (@invoke, $macro:ident, {
        $(
            // The language(s) in which these operations can be used.
            // Can be either 'functional, 'scheduling, or 'shared (for both).
            // This is marked as a lifetime for syntax highlighting purposes.
            $language:lifetime: $(
                // The name of the operation. The "fn" is here for syntax highlighting purposes.
                fn $name:ident
                // The arguments to the operation. $type doesn't refer to a fixed Rust type;
                // instead, it refers to an "abstract type" that $macro is expected to replace
                // with a fixed Rust type.
                ($($input:ident : $input_type:tt),* $(,)?)
                // The output "shape" of this operation. Can be either None, Single, or Multiple.
                -> $output:ident;
            )*
        )*
    }) => {
        $macro! { $($($language ($name $output ($($input : $input_type,)*));)*)* }
    };
    ($macro:ident) => {with_operations! { @invoke, $macro, {
        // TODO: has_local_side_effect & intrinsic?
        'shared:
            fn None() -> None;
            fn Phi(index: Index) -> Single;
            fn ExtractResult(node_id: Operation, index: Index) -> Single;

        'functional:
            fn ConstantInteger(value: ImmediateI64, type_id: Type) -> Single;
            fn ConstantUnsignedInteger(value: ImmediateU64, type_id: Type) -> Single;

            fn CallValueFunction(
                function_id: ValueFunction,
                arguments: [Operation]
            ) -> Multiple;

            fn CallExternalCpu(
                external_function_id: ExternalCpuFunction,
                arguments: [Operation]
            ) -> Multiple;

            fn CallExternalGpuCompute(
                external_function_id: ExternalGpuFunction,
                dimensions: [Operation],
                arguments: [Operation],
            ) -> Multiple;

        'scheduling:
            fn SyncLocal(values: [Operation]) -> None;
            fn EncodeGpu(values: [Operation]) -> None;
            fn Hole() -> Single;

            // TODO: is_inferable?
            fn EncodeDo(
                value: Operation,
                inputs: [Operation],
                outputs: [Operation]
            ) -> Single;

            fn EncodeCopy(value: Operation) -> Single;
            fn Submit(place: Place) -> None;
            fn EncodeFence(place: Place) -> None;
            fn SyncFence(place: Place, fence: Operation) -> None;
    } } }
}

#[cfg(test)]
mod tests {
    macro_rules! example {
        ($($lang:lifetime ($name:ident $output:ident ($($arg:ident : $arg_type:tt,)*));)*) => {
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
