use super::*;

impl SpecInfo {
    pub fn new(program: &ast::Program) -> SpecInfo {
        for declaration in program.declarations {
            match declaration {
                ast::Declaration::TypeDecl(_) => {}
                ast::Declaration::ExternalFunction(_) => {}
                ast::Declaration::FunctionClass(_) => {}
                ast::Declaration::Funclet(f) => {}
                ast::Declaration::Pipeline(_) => {}
            }
        }
    }
}