use crate::assembly_ast;
use crate::assembly_context;
use crate::assembly_context::Table;
use crate::ir;
use std::collections::{HashSet, HashMap};
use std::fmt::Debug;
use std::hash::Hash;

// just putting this wrapper in up-front to make getting information easy later
pub struct Explication {
    pending : bool // if true, this explication has been checked, but is pending a loop
}

pub struct Context<'a> {
    program: &'a assembly_ast::Program, // reference to the whole program for lookups
    assembly_context: assembly_context::Context, // owned for mutability
    explicated_allocations: Table<String, Explication>, // table of allocated value elements
    explicated_funclets: HashSet<String>, // table of explicated funclets
}

impl<'a> Context<'a> {
    pub fn new(
        assembly_context: assembly_context::Context,
        program: &'a assembly_ast::Program,
    ) -> Context<'a> {
        Context {
            program,
            assembly_context,
            explicated_allocations: Table::new(),
            explicated_funclets: HashSet::new(),
        }
    }
    pub fn inner(&mut self) -> &mut assembly_context::Context {
        &mut self.assembly_context
    }
    pub fn program(&mut self) -> &assembly_ast::Program {
        self.program
    }

    pub fn reset(&mut self) { // for use at end of pass
        self.explicated_allocations = Table::new();
        self.explicated_funclets = HashSet::new();
    }

    pub fn explicate_allocation(&mut self, name : String, pending : bool) {
        self.explicated_allocations.push(name, Explication { pending });
    }

    pub fn get_allocation(&mut self, name : String) -> Option<(&Explication, usize)> {
        self.explicated_allocations.get(&name)
    }
}
