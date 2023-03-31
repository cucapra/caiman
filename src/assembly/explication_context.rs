use crate::assembly_ast;
use crate::assembly_context;
use crate::assembly_context::Table;
use crate::ir;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

// just putting this wrapper in up-front to make getting information easy later
pub struct Allocation {
    name: Option<String>, // is None exactly when the variable has been checked, but result pending
}

pub struct FuncletData {
    explicated_allocations: HashMap<String, Allocation>, // information about allocated value elements
}

struct Indices {
    allocation_index: usize,
}

pub struct Context<'a> {
    program: &'a assembly_ast::Program, // reference to the whole program for lookups
    assembly_context: assembly_context::Context, // owned for mutability
    explicated_funclets: HashMap<String, FuncletData>, // table of explicated funclets
    indices: Indices,
}

impl FuncletData {
    pub fn new() -> FuncletData {
        FuncletData {
            explicated_allocations: HashMap::new(),
        }
    }
    pub fn allocate(&mut self, name: String, allocation: Allocation) {
        self.explicated_allocations.insert(name, allocation);
    }
    pub fn get_allocation(&self, name: String) -> Option<&Allocation> {
        self.explicated_allocations.get(name.as_str())
    }
}

impl<'a> Context<'a> {
    pub fn new(
        assembly_context: assembly_context::Context,
        program: &'a assembly_ast::Program,
    ) -> Context<'a> {
        Context {
            program,
            assembly_context,
            explicated_funclets: HashMap::new(),
            indices: Indices {
                allocation_index: 0,
            },
        }
    }
    pub fn inner(&mut self) -> &mut assembly_context::Context {
        &mut self.assembly_context
    }
    pub fn program(&mut self) -> &assembly_ast::Program {
        self.program
    }

    fn allocation_name(&mut self) -> String {
        self.indices.allocation_index += 1;
        format!("${}", self.indices.allocation_index)
    }

    pub fn clear_allocations(&mut self) {
        self.indices.allocation_index = 0;
        let mut keys = Vec::new();
        // todo: fix
        for key in self.explicated_funclets.keys() {
            keys.push(key.clone());
        }
        for key in keys {
            self.explicated_funclets
                .insert(key.clone(), FuncletData::new());
        }
    }

    pub fn explicate_allocation(
        &mut self,
        remote: &assembly_ast::RemoteNodeId,
        valid: bool,
    ) -> Option<String> {
        let name = if valid {
            Some(self.allocation_name())
        } else {
            None
        };
        self.explicated_funclets
            .get_mut(remote.funclet_id.as_str())
            .unwrap()
            .allocate(remote.node_id.clone(), Allocation { name: name.clone() });
        name
    }

    pub fn get_allocation(&mut self, target: String, name: String) -> Option<&Allocation> {
        self.explicated_funclets
            .get(target.as_str())
            .and_then(|x| x.get_allocation(name))
    }

    pub fn explicate_funclet(&mut self, name: String) {
        self.explicated_funclets.insert(name, FuncletData::new()); // dupes are whatever here
    }

    pub fn funclet_explicated(&mut self, name: String) -> bool {
        self.explicated_funclets.contains_key(name.as_str())
    }
}
