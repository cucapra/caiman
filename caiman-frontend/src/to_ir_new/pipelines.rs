use crate::parse::ast;
use caiman::assembly::ast as asm;

pub fn lower_pipelines(program: &ast::Program) -> Vec<asm::Pipeline> {
    program
        .iter()
        .filter_map(|(_info, decl)| match decl {
            ast::DeclKind::Pipeline {
                name,
                funclet_to_run,
            } => Some(asm::Pipeline {
                name: name.clone(),
                funclet: asm::FuncletId(funclet_to_run.clone()),
            }),
            _ => None,
        })
        .collect()
}
