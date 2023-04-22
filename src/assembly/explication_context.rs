use crate::assembly_ast;
use crate::assembly_ast::Hole;
use crate::assembly_ast::{
    ExternalCpuFunctionId, ExternalGpuFunctionId, FuncletId, NodeId, OperationId, StorageTypeId,
    TypeId, ValueFunctionId,
};
use crate::assembly_context;
use crate::assembly_context::Table;
use crate::ir;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Debug)]
pub struct ValueFuncletData {
    // information about allocated value elements
    // explication locations within the scheduling world
    explicated_allocations: HashMap<String, Option<assembly_ast::RemoteNodeId>>,
    // map from call index to output name for each call
    call_outputs: HashMap<NodeId, HashMap<usize, NodeId>>
}

#[derive(Debug)]
struct MetaData {
    variable_index: usize,
}

#[derive(Debug)]
pub struct Context<'a> {
    program: &'a assembly_ast::Program, // reference to the whole program for lookups
    pub inner: assembly_context::Context, // owned for mutability
    value_function_explication_data: HashMap<String, ValueFuncletData>, // table of explicated funclets
    meta_data: MetaData,
    pub schedule_extras: HashMap<String, ir::SchedulingFuncletExtra>,
    // map from schedule to value
    value_map: HashMap<assembly_ast::FuncletId, assembly_ast::FuncletId>,
}

impl ValueFuncletData {
    pub fn new() -> ValueFuncletData {
        ValueFuncletData {
            explicated_allocations: HashMap::new(),
            call_outputs: HashMap::new(),
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
            value_function_explication_data: HashMap::new(),
            meta_data: MetaData { variable_index: 0 },
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
        for key in self.value_function_explication_data.keys() {
            keys.push(key.clone());
        }
        for key in keys {
            self.value_function_explication_data
                .insert(key.clone(), ValueFuncletData::new());
        }
    }

    pub fn add_allocation(&mut self, remote: &assembly_ast::RemoteNodeId) {
        let allocation = assembly_ast::RemoteNodeId {
            funclet_id: self.inner.current_funclet_name(),
            node_id: self.inner.current_node_name(),
        };
        self.value_function_explication_data
            .get_mut(remote.funclet_id.as_str())
            .unwrap()
            .allocate(remote.node_id.clone(), Some(allocation));
    }

    pub fn get_allocation(
        &self,
        remote: &assembly_ast::RemoteNodeId,
    ) -> Option<&assembly_ast::RemoteNodeId> {
        self.value_function_explication_data
            .get(&remote.funclet_id)
            .and_then(|f| f.explicated_allocations.get(&remote.node_id))
            .and_then(|hole| hole.as_ref())
    }

    pub fn get_funclet_data(&self, funclet: String) -> Option<&ValueFuncletData> {
        self.value_function_explication_data.get(funclet.as_str())
    }

    pub fn get_current_funclet(&self) -> Option<&ValueFuncletData> {
        self.get_funclet_data(self.inner.current_funclet_name())
    }

    pub fn explicate_funclet(&mut self, name: String) {
        self.value_function_explication_data
            .insert(name, ValueFuncletData::new()); // dupes are whatever here
    }

    pub fn funclet_explicated(&mut self, name: String) -> bool {
        self.value_function_explication_data
            .contains_key(name.as_str())
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

    pub fn get_cpu_funclet(
        &self,
        name: &ExternalCpuFunctionId,
    ) -> &assembly_ast::ExternalCpuFunction {
        for funclet in &self.program.funclets {
            match funclet {
                assembly_ast::FuncletDef::ExternalCPU(f) => return f,
                _ => {}
            }
        }
        panic!("CPU funclet {} not found", name);
    }

    pub fn get_gpu_funclet(
        &self,
        name: &ExternalGpuFunctionId,
    ) -> &assembly_ast::ExternalGpuFunction {
        for funclet in &self.program.funclets {
            match funclet {
                assembly_ast::FuncletDef::ExternalGPU(f) => return f,
                _ => {}
            }
        }
        panic!("GPU funclet {} not found", name);
    }

    pub fn get_value_function(&self, name: &ValueFunctionId) -> &assembly_ast::ValueFunction {
        for funclet in &self.program.funclets {
            match funclet {
                assembly_ast::FuncletDef::ValueFunction(f) => return f,
                _ => {}
            }
        }
        panic!("Value function {} not found", name);
    }
}
