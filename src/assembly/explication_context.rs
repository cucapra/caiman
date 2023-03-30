use crate::assembly_context;
use crate::assembly_ast;
use crate::ir;

pub struct Context {
    program_context : assembly_context::Context
}

impl Context {
    pub fn new(program_context : assembly_context::Context) -> Context {
        Context { program_context }
    }
    pub fn inner(&mut self) -> &mut assembly_context::Context {
        &mut self.program_context
    }
}