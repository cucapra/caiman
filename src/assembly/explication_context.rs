use crate::assembly_ast;
use crate::assembly_ast::Hole;
use crate::assembly_context;
use crate::assembly_context::Table;
use crate::ir;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Debug)]
pub struct FuncletData {
    // information about allocated value elements
    explicated_allocations: HashMap<String, Option<assembly_ast::RemoteNodeId>>,
}

#[derive(Debug)]
struct MetaData {
    variable_index: usize,
}

#[derive(Debug)]
pub struct Context<'a> {
    program: &'a assembly_ast::Program, // reference to the whole program for lookups
    pub inner: assembly_context::Context, // owned for mutability
    explicated_funclets: HashMap<String, FuncletData>, // table of explicated funclets
    meta_data: MetaData,
    pub schedule_extras: HashMap<String, ir::SchedulingFuncletExtra>,
    // map from schedule to value
    value_map: HashMap<assembly_ast::FuncletId, assembly_ast::FuncletId>,
}

impl FuncletData {
    pub fn new() -> FuncletData {
        FuncletData {
            explicated_allocations: HashMap::new(),
        }
    }
    pub fn allocate(&mut self, name: String, allocation: Option<assembly_ast::RemoteNodeId>) {
        self.explicated_allocations.insert(name, allocation);
    }
    pub fn get_allocation(&self, name: String) -> Option<&assembly_ast::RemoteNodeId> {
        self.explicated_allocations
            .get(name.as_str())
            .and_then(|x| x.as_ref())
    }
}

impl<'a> Context<'a> {
    pub fn new(
        assembly_context: assembly_context::Context,
        program: &'a assembly_ast::Program,
    ) -> Context<'a> {
        Context {
            program,
            inner: assembly_context,
            explicated_funclets: HashMap::new(),
            meta_data: MetaData {
                variable_index: 0,
            },
            schedule_extras: HashMap::new(),
            value_map: HashMap::new(),
        }
    }

    pub fn program(&mut self) -> &assembly_ast::Program {
        self.program
    }

    fn allocation_name(&mut self) -> String {
        self.meta_data.variable_index += 1;
        format!("${}", self.meta_data.variable_index)
    }

    pub fn clear_allocations(&mut self) {
        self.meta_data.variable_index = 0;
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

    // pub fn explicate_allocation(&mut self, remote: &assembly_ast::RemoteNodeId, valid: bool) {
    //     let allocation = if valid {
    //         Some(assembly_ast::RemoteNodeId {
    //             funclet_id: self.inner.current_funclet_name(),
    //             node_id: self.allocation_name(),
    //         })
    //     } else {
    //         None
    //     };
    //     self.explicated_funclets
    //         .get_mut(remote.funclet_id.as_str())
    //         .unwrap()
    //         .allocate(remote.node_id.clone(), allocation);
    // }

    pub fn add_allocation(&mut self, remote: &assembly_ast::RemoteNodeId) {
        let allocation = assembly_ast::RemoteNodeId {
            funclet_id: self.inner.current_funclet_name(),
            node_id: self.inner.current_node_name(),
        };
        self.explicated_funclets
            .get_mut(remote.funclet_id.as_str())
            .unwrap()
            .allocate(remote.node_id.clone(), Some(allocation));
    }

    pub fn get_allocation(
        &self,
        remote: &assembly_ast::RemoteNodeId,
    ) -> Option<&assembly_ast::RemoteNodeId> {
        self.explicated_funclets
            .get(&remote.funclet_id)
            .and_then(|f| f.explicated_allocations.get(&remote.node_id))
            .and_then(|hole| hole.as_ref())
    }

    pub fn get_funclet_data(&self, funclet: String) -> Option<&FuncletData> {
        self.explicated_funclets.get(funclet.as_str())
    }

    pub fn get_current_funclet(&self) -> Option<&FuncletData> {
        self.get_funclet_data(self.inner.current_funclet_name())
    }

    pub fn explicate_funclet(&mut self, name: String) {
        self.explicated_funclets.insert(name, FuncletData::new()); // dupes are whatever here
    }

    pub fn funclet_explicated(&mut self, name: String) -> bool {
        self.explicated_funclets.contains_key(name.as_str())
    }

    pub fn get_current_extra(&self) -> &ir::SchedulingFuncletExtra {
        self.schedule_extras
            .get(&self.inner.current_funclet_name().clone())
            .unwrap()
    }

    pub fn get_value(&self, funclet: &assembly_ast::FuncletId) -> &assembly_ast::FuncletId {
        self.value_map.get(funclet).unwrap()
    }

    pub fn set_value(&mut self, schedule: assembly_ast::FuncletId, value: assembly_ast::FuncletId) {
        assert!(!self.value_map.contains_key(&schedule));
        self.value_map.insert(schedule, value);
    }

    pub fn node_lookup(&self, location: &assembly_ast::RemoteNodeId) -> &assembly_ast::Node {
        let funclet_id = self.inner.funclet_id(&location.funclet_id);
        let node_id = self
            .inner
            .remote_node_id(location.funclet_id.clone(), location.node_id.clone());
        match &self.program.funclets[funclet_id] {
            assembly_ast::FuncletDef::Local(f) => f.commands[node_id].as_ref().unwrap(),
            _ => panic!(
                "attempted to access non-local node in {}",
                location.funclet_id.clone()
            ),
        }
    }
}
