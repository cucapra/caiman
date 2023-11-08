use super::function_classes::FunctionClassContext;
use super::typing;
use crate::parse::ast;
use caiman::assembly::ast as asm;

pub fn lower_cpu_externs(
    function_class_ctx: &FunctionClassContext,
    program: &ast::Program,
) -> Vec<asm::ExternalFunction> {
    program
        .iter()
        .filter_map(|(_info, decl)| match decl {
            ast::DeclKind::ExternCPU {
                name,
                input,
                output,
            } => Some(lower_cpu_extern(function_class_ctx, name, input, output)),
            _ => None,
        })
        .collect()
}

fn lower_cpu_extern(
    function_class_ctx: &FunctionClassContext,
    name: &str,
    input: &Vec<ast::value::Type>,
    output: &ast::value::Type,
) -> asm::ExternalFunction {
    let convert_type = |vt: &ast::value::Type| match typing::convert_value_type(vt.clone()) {
        asm::TypeId::FFI(ffi_type) => asm::ExternalArgument {
            // TODO named args
            name: None,
            ffi_type,
        },
        _ => panic!("Value type is non-FFI somehow"),
    };

    let function_class = function_class_ctx
        .get(name)
        .unwrap_or(asm::FunctionClassId(name.to_string()));

    asm::ExternalFunction {
        kind: asm::ExternalFunctionKind::CPUPure,
        value_function_binding: asm::FunctionClassBinding {
            default: false,
            function_class,
        },
        name: name.to_string(),
        input_args: input.iter().map(convert_type).collect(),
        output_types: vec![convert_type(output)],
    }
}
