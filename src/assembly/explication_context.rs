use crate::assembly_ast;
use crate::assembly_context;
use crate::assembly_context::Table;
use crate::ir;
use std::collections::HashSet;

#[derive(Debug, Clone, Hash)]
pub struct Explication {
    name: String,
    pending: bool,
}

impl PartialEq<Self> for Explication {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Explication {}

pub struct Context<'a> {
    program: &'a assembly_ast::Program, // reference to the whole program for lookups
    assembly_context: assembly_context::Context, // owned for mutability
    explicated_allocations: Table<Explication>, // table of allocated value elements
    explicated_funclets: HashSet<Explication>, // table of explicated funclets
}

impl<'a> Context<'a> {
    pub fn new(assembly_context : assembly_context::Context, program : &'a assembly_ast::Program) -> Context<'a> {

        Context {
            assembly_context,
            explicated_allocations: Table::new(),
            explicated_funclets: HashSet::new(),
            program
        }
    }
    pub fn inner(&mut self) -> &mut assembly_context::Context {
        &mut self.assembly_context
    }
    pub fn program(&mut self) -> &assembly_ast::Program {
        self.program
    }
}
