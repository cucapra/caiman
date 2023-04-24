use crate::assembly_ast;
use crate::assembly_ast::Hole;
use crate::assembly_ast::{
    ExternalCpuFunctionId, ExternalGpuFunctionId, FuncletId, NodeId, OperationId, StorageTypeId,
    TypeId, ValueFunctionId,
};
use crate::ir;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Debug)]
pub struct Table<T>
    where
        T: Eq + Hash + Debug + Clone,
{
    values: HashSet<T>,
    indices: Vec<T>,
}

#[derive(Debug)]
struct NodeTable {
    // local names and return names such as [%out : i64] or whatever
    local: Table<String>,
    returns: Table<String>,
}

#[derive(Debug)]
pub struct ValueFuncletData {
    // information about allocated value elements
    // explication locations within the scheduling world
    explicated_allocations: HashMap<String, Option<assembly_ast::RemoteNodeId>>,
    // map from call index to output name for each call
    call_outputs: HashMap<NodeId, HashMap<usize, NodeId>>,
}

#[derive(Debug, Clone)]
pub enum FuncletLocation {
    Local,
    Value,
    Cpu,
    Gpu,
}

#[derive(Debug, Clone)]
pub enum Location {
    FFI(usize),
    Local(usize),
}

#[derive(Debug, Clone)]
pub enum NodeType {
    // Keeps track of internal names vs return names
    Local(usize),
    Return(usize),
}

impl Location {
    pub fn unpack(&self) -> usize {
        match self {
            Location::Local(u) => *u,
            Location::FFI(u) => *u,
        }
    }
}

pub struct FuncletInformation {
    location: FuncletLocation,
    index: usize,
}

#[derive(Debug)]
struct MetaData {
    variable_index: usize,
}

#[derive(Debug)]
pub struct Context<'a> {
    program: &'a assembly_ast::Program, // reference to the whole program for lookups
    pub schedule_extras: HashMap<String, ir::SchedulingFuncletExtra>,

    value_function_explication_data: HashMap<String, ValueFuncletData>, // table of explicated funclets
    meta_data: MetaData,
    // map from schedule to value
    value_map: HashMap<assembly_ast::FuncletId, assembly_ast::FuncletId>,

    pub ffi_type_table: Table<assembly_ast::FFIType>,
    pub local_type_table: Table<String>,
    pub external_funclet_table: Table<String>,
    pub local_funclet_table: Table<String>,
    pub value_function_table: Table<String>,
    pub remote_map: HashMap<String, NodeTable>,
    pub command_var_name: Option<String>,
    pub location: Option<ir::RemoteNodeId>,
}

// a Table is basically a vector with no dupes
impl<T> Table<T>
    where
        T: Eq + Hash + Debug + Clone,
{
    pub fn new() -> Table<T> {
        Table {
            values: HashSet::new(),
            indices: Vec::new(),
        }
    }

    pub fn contains(&mut self, val: &T) -> bool {
        self.values.contains_key(val)
    }

    pub fn dummy_push(&mut self, val: T) {
        // Add unnamed element for indexing
        self.indices.push(val);
    }

    pub fn push(&mut self, val: T) {
        if self.values.contains_key(&val) {
            panic!("Duplicate add of {:?}", val)
        }
        self.values.insert(val.clone());
        self.indices.push(val);
    }

    pub fn insert(&mut self, index: usize, val: T, data: U) {
        if self.values.contains_key(&val) {
            panic!("Duplicate add of {:?}", val)
        }
        self.values.insert(val.clone());
        self.indices.insert(index, val);
    }

    pub fn get(&self, val: &T) -> Option<usize> {
        // no need to actually check the Hashset, that's just to avoid dupes
        for item in itertools::enumerate(&self.indices) {
            if item.1 == val {
                return Some(item.0));
            }
        }
        return None;
    }

    pub fn get_index(&self, val: &T) -> Option<usize> {
        self.get(val).map(|x| x.1)
    }

    pub fn get_at_index(&self, index: usize) -> Option<&T> {
        if index >= self.indices.len() {
            None
        } else {
            Some(&self.indices[index])
        }
    }

    pub fn len(&mut self) -> usize {
        return self.indices.len();
    }
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
            ffi_type_table: Table::new(),
            local_type_table: Table::new(),
            funclet_kind_map: HashMap::new(),
            local_funclet_table: Table::new(),
            external_funclet_table: Table::new(),
            value_function_table: Table::new(),
            remote_map: HashMap::new(),
            command_var_name: None,
            location: None,
        }
    }

    pub fn new() -> Context {
        Context {
        }
    }

    pub fn current_funclet_name(&self) -> String {
        self.local_funclet_table
            .get_at_index(self.location.unwrap().funclet_id)
            .unwrap()
            .clone()
    }

    pub fn current_node_name(&self) -> String {
        self.node_from_id(self.location.unwrap().node_id)
    }

    // for use by the explicator
    pub fn add_ffi_type(&mut self, t: assembly_ast::FFIType) {
        self.ffi_type_table.push(t, ());
    }

    pub fn add_local_type(&mut self, name: String) {
        self.local_type_table.push(name, ());
    }

    pub fn add_cpu_funclet(&mut self, name: String) {
        self.funclet_kind_map
            .insert(name.clone(), FuncletLocation::Cpu);
        self.external_funclet_table.push(name, ());
    }

    pub fn add_gpu_funclet(&mut self, name: String) {
        self.funclet_kind_map
            .insert(name.clone(), FuncletLocation::Gpu);
        self.external_funclet_table.push(name, ());
    }

    pub fn add_local_funclet(&mut self, name: String) {
        self.funclet_kind_map
            .insert(name.clone(), FuncletLocation::Local);
        self.local_funclet_table.push(name.clone(), ());
        self.remote_map.insert(
            name,
            NodeTable {
                local: Table::new(),
                returns: Table::new(),
            },
        );
        self.advance_current_funclet();
    }

    pub fn add_value_function(&mut self, name: String) {
        self.funclet_kind_map
            .insert(name.clone(), FuncletLocation::Value);
        self.value_function_table.push(name, ());
    }

    pub fn clear_current_funclet(&mut self) {
        self.location = None;
    }

    pub fn initialize_current_funclet(&mut self) {
        self.location = Some(ir::RemoteNodeId {
            funclet_id: 0,
            node_id: 0,
        });
    }

    pub fn set_current_funclet(&mut self, index: usize) {
        self.location = Some(ir::RemoteNodeId {
            funclet_id: index,
            node_id: 0,
        });
    }

    pub fn set_current_node(&mut self, funclet: usize, index: usize) {
        self.location = Some(ir::RemoteNodeId {
            funclet_id: funclet,
            node_id: index,
        });
    }

    pub fn advance_current_funclet(&mut self) {
        let index = match self.location {
            None => 0,
            Some(v) => v.funclet_id + 1,
        };
        self.set_current_funclet(index)
    }

    pub fn advance_current_node(&mut self) {
        let (funclet, node) = match self.location {
            None => panic!("Cannot advance empty current node"),
            Some(v) => (v.funclet_id, v.node_id + 1),
        };
        self.set_current_node(funclet, node)
    }

    pub fn add_node(&mut self, name: String) {
        let mut advance = false;
        match self.remote_map.get_mut(&self.current_funclet_name()) {
            None => panic!("Invalid funclet name {:?}", name),
            Some(table) => {
                advance = table.local.len() > 0;
                if name == "_" {
                    table.local.dummy_push(name)
                } else {
                    table.local.push(name, ())
                }
            }
        }
        if advance {
            self.advance_current_node();
        }
    }

    pub fn add_return(&mut self, name: String) {
        match self.remote_map.get_mut(&self.current_funclet_name()) {
            None => panic!("Invalid funclet name {:?}", name),
            Some(table) => table.returns.push(name, ()),
        }
    }

    // pub fn insert_node(&mut self, name: String) {
    //     match self.remote_map.get_mut(&self.funclet_name()) {
    //         None => panic!("Invalid funclet name {:?}", name),
    //         Some(table) => table.insert(self.location.unwrap().node_id, name)
    //     }
    //     self.location.unwrap().node_id += 1;
    // }

    pub fn ffi_type_id(&self, name: &assembly_ast::FFIType) -> usize {
        match self.ffi_type_table.get_index(name) {
            Some(i) => i,
            None => panic!("Un-indexed FFI type {:?}", name),
        }
    }

    pub fn local_type_id(&self, name: &String) -> usize {
        match self.local_type_table.get_index(name) {
            Some(t) => t,
            None => panic!("Unknown local type {:?}", name),
        }
    }

    pub fn loc_type_id(&self, typ: assembly_ast::Type) -> usize {
        match typ {
            assembly_ast::Type::FFI(ft) => self.ffi_type_id(&ft),
            assembly_ast::Type::Local(s) => self.local_type_id(&s),
        }
    }

    pub fn funclet_location(&self, name: &String) -> &FuncletLocation {
        match self.funclet_kind_map.get(name) {
            Some(f) => f,
            None => panic!("Unknown funclet name {:?}", name),
        }
    }

    pub fn funclet_id(&self, name: &String) -> usize {
        match self.funclet_location(name) {
            FuncletLocation::Local => self.local_funclet_table.get(name).unwrap().1,
            FuncletLocation::Value => self.value_function_table.get(name).unwrap().1,
            _ => self.external_funclet_table.get(name).unwrap().1,
        }
    }

    pub fn local_funclet_id(&self, name: String) -> usize {
        match self.funclet_location(&name) {
            FuncletLocation::Local => self.local_funclet_table.get(&name).unwrap().1,
            _ => panic!("Not a local funclet {}", name),
        }
    }
    pub fn cpu_funclet_id(&self, name: String) -> usize {
        match self.funclet_location(&name) {
            FuncletLocation::Cpu => self.external_funclet_table.get(&name).unwrap().1,
            _ => panic!("Not a cpu funclet {}", name),
        }
    }
    pub fn gpu_funclet_id(&self, name: String) -> usize {
        match self.funclet_location(&name) {
            FuncletLocation::Gpu => self.external_funclet_table.get(&name).unwrap().1,
            _ => panic!("Not a gpu funclet {}", name),
        }
    }
    pub fn value_function_id(&self, name: String) -> usize {
        match self.funclet_location(&name) {
            FuncletLocation::Value => self.value_function_table.get(&name).unwrap().1,
            _ => panic!("Not a value function {}", name),
        }
    }

    pub fn remote_node_id(&self, funclet: String, var: String) -> usize {
        match self.remote_map.get(funclet.as_str()) {
            Some(f) => match f.local.get(&var) {
                Some(v) => v.1,
                None => panic!("Unknown local name {} in funclet {}", var, funclet),
            },
            None => panic!("Unknown funclet name {}", funclet),
        }
    }

    pub fn remote_return_id(&self, funclet: String, var: String) -> usize {
        match self.remote_map.get(funclet.as_str()) {
            Some(f) => match f.returns.get_index(&var) {
                Some(v) => v,
                None => panic!("Unknown return name {} in funclet {}", var, funclet),
            },
            None => panic!("Unknown funclet name {}", funclet),
        }
    }

    pub fn node_from_id(&self, index: usize) -> String {
        self.remote_map
            .get(self.current_funclet_name().as_str())
            .unwrap()
            .local
            .get_at_index(index)
            .unwrap()
            .clone()
    }

    pub fn node_id(&self, var: String) -> usize {
        let funclet = self.current_funclet_name();
        match self.remote_map.get(&funclet).unwrap().local.get_index(&var) {
            Some(v) => v,
            None => panic!("Unknown variable name {:?} in funclet {:?}", var, &funclet),
        }
    }

    pub fn return_id(&self, var: String) -> usize {
        let funclet = self.current_funclet_name();
        match self
            .remote_map
            .get(&funclet)
            .unwrap()
            .returns
            .get_index(&var)
        {
            Some(v) => v,
            None => panic!("Unknown return name {:?} in funclet {:?}", var, &funclet),
        }
    }

    pub fn remote_id(&self, funclet: String, var: String) -> ir::RemoteNodeId {
        ir::RemoteNodeId {
            funclet_id: self.local_funclet_id(funclet.clone()).clone(),
            node_id: self.remote_node_id(funclet, var),
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
