use crate::syntax::ast;
use caiman::assembly::ast as asm;
//use caiman::ir;

mod binops;
mod external_cpu;
mod funclet_util;
mod function_classes;
mod label;
mod pipelines;
mod value_funclets;
mod scheduling_funclets_ir;
mod scheduling_funclets;
mod spatial_funclets;
mod timeline_funclets;
mod typing;

macro_rules! to_decl {
    ($v : expr, $kind : ident) => {
        $v.into_iter().map(|x| asm::Declaration::$kind(x))
    };
    ($v : expr) => {
        $v.into_iter().map(|x| asm::Declaration::Funclet(x.0))
    };
}

pub fn frontend_to_asm(mut program: ast::Program) -> asm::Program
{
    let mut typing_ctx = typing::TypingContext::new();

    let (mut asm_function_classes, mut function_class_ctx) = function_classes::make(&program);
    //binops::externalize_binops(&mut program, &mut asm_function_classes, &mut function_class_ctx);

    let asm_external_cpus = external_cpu::lower_cpu_externs(&function_class_ctx, &program);

    let mut asm_value_funclets =
        value_funclets::lower_value_funclets(&function_class_ctx, &program);
    typing_ctx.add_value_funclet_types(&asm_value_funclets);
    typing_ctx.convert_value_funclet_types(&mut asm_value_funclets);

    let asm_scheduling_funclets = scheduling_funclets::lower_scheduling_funclets(
        &function_class_ctx,
        &mut typing_ctx,
        &asm_value_funclets,
        &program,
    );

    let asm_timeline_funclets =
        timeline_funclets::lower_timeline_funclets(&mut typing_ctx, &program);

    let asm_spatial_funclets = spatial_funclets::lower_spatial_funclets(&mut typing_ctx, &program);

    let asm_pipelines = pipelines::lower_pipelines(&program);

    let types = typing_ctx.into_types();
    let declarations = to_decl!(types, TypeDecl)
        .chain(to_decl!(asm_function_classes, FunctionClass))
        .chain(to_decl!(asm_external_cpus, ExternalFunction))
        .chain(to_decl!(asm_value_funclets))
        .chain(to_decl!(asm_scheduling_funclets))
        .chain(to_decl!(asm_timeline_funclets))
        .chain(to_decl!(asm_spatial_funclets))
        .chain(to_decl!(asm_pipelines, Pipeline))
        .collect();

    let version = asm::Version { major: 0, minor: 0, detailed: 2 };
    asm::Program { version, declarations, path: String::new() }
}
