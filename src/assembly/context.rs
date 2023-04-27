use crate::assembly;
use crate::assembly::ast::Hole;
use crate::assembly::ast::{
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
pub struct NodeTable {
    // local names and return names such as [%out : i64] or whatever
    local: Table<String>,
    returns: Table<String>,
}

#[derive(Debug)]
pub struct ValueFuncletData {
    // information about allocated value elements
    // explication locations within the scheduling world
    explicated_allocations: HashMap<String, Option<assembly::ast::RemoteNodeId>>,
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
pub struct FuncletIndices {
    external_funclet_table: Table<String>,
    local_funclet_table: Table<String>,
    value_function_table: Table<String>,
    funclet_kind_map: HashMap<String, FuncletLocation>,
}

#[derive(Debug)]
struct MetaData {
    variable_index: usize,
}

#[derive(Debug)]
pub struct Context<'a> {
    pub schedule_extras: HashMap<assembly::ast::FuncletId, ir::SchedulingFuncletExtra>,
    pub ffi_type_table: Table<assembly::ast::FFIType>,
    pub local_type_table: Table<String>,
    pub remote_map: HashMap<assembly::ast::FuncletId, NodeTable>,
    // where we currently are in the AST, using names
    // optional cause we may not have started traversal
    pub location: assembly::ast::RemoteNodeId,
    pub funclet_indices: FuncletIndices,

    // reference to the whole program for lookups
    // avoid making public cause we want to control this with the context
    program: &'a assembly::ast::Program,
    // information found about value funclets
    value_function_explication_data: HashMap<String, ValueFuncletData>,
    meta_data: MetaData,
    // map from schedule to value
    value_map: HashMap<assembly::ast::FuncletId, assembly::ast::FuncletId>,
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
        self.values.contains(val)
    }

    pub fn dummy_push(&mut self, val: T) {
        // Add unnamed element for indexing
        self.indices.push(val);
    }

    pub fn push(&mut self, val: T) {
        if self.values.contains(&val) {
            panic!("Duplicate add of {:?}", val)
        }
        self.values.insert(val.clone());
        self.indices.push(val);
    }

    pub fn insert(&mut self, index: usize, val: T) {
        if self.values.contains(&val) {
            panic!("Duplicate add of {:?}", val)
        }
        self.values.insert(val.clone());
        self.indices.insert(index, val);
    }

    pub fn get(&self, val: &T) -> Option<usize> {
        // no need to actually check the Hashset, that's just to avoid dupes
        for item in itertools::enumerate(&self.indices) {
            if item.1 == val {
                return Some(item.0);
            }
        }
        return None;
    }

    pub fn get_index(&self, val: &T) -> Option<usize> {
        self.get(val)
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

pub fn fresh_location() -> assembly::ast::RemoteNodeId {
    assembly::ast::RemoteNodeId {
        funclet_id: "".to_string(),
        node_id: "".to_string(),
    }
}

impl ValueFuncletData {
    pub fn new() -> ValueFuncletData {
        ValueFuncletData {
            explicated_allocations: HashMap::new(),
            call_outputs: HashMap::new(),
        }
    }
    pub fn allocate(&mut self, name: String, allocation: Option<assembly::ast::RemoteNodeId>) {
        self.explicated_allocations.insert(name, allocation);
    }
    pub fn get_allocation(&self, name: String) -> Option<&assembly::ast::RemoteNodeId> {
        self.explicated_allocations
            .get(name.as_str())
            .and_then(|x| x.as_ref())
    }
}

impl FuncletIndices {
    pub fn new() -> FuncletIndices {
        FuncletIndices {
            external_funclet_table: Table::new(),
            local_funclet_table: Table::new(),
            value_function_table: Table::new(),
            funclet_kind_map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: String, location: FuncletLocation) {
        self.funclet_kind_map.insert(name.clone(), location.clone());
        match location {
            FuncletLocation::Local => self.local_funclet_table.push(name),
            FuncletLocation::Value => self.value_function_table.push(name),
            FuncletLocation::Cpu => self.external_funclet_table.push(name),
            FuncletLocation::Gpu => self.external_funclet_table.push(name),
        };
    }

    pub fn get_loc(&self, name: &str) -> Option<&FuncletLocation> {
        self.funclet_kind_map.get(name)
    }

    pub fn get(&self, name: &String) -> Option<usize> {
        self.funclet_kind_map.get(name).and_then(|x| match x {
            FuncletLocation::Local => self.local_funclet_table.get(name),
            FuncletLocation::Value => self.value_function_table.get(name),
            FuncletLocation::Cpu => self.external_funclet_table.get(name),
            FuncletLocation::Gpu => self.external_funclet_table.get(name),
        })
    }
}

impl<'a> Context<'a> {
    pub fn new(program: &'a assembly::ast::Program) -> Context<'a> {
        let mut context = Context {
            program,
            value_function_explication_data: HashMap::new(),
            meta_data: MetaData { variable_index: 0 },
            schedule_extras: HashMap::new(),
            value_map: HashMap::new(),
            ffi_type_table: Table::new(),
            local_type_table: Table::new(),
            funclet_indices: FuncletIndices::new(),
            remote_map: HashMap::new(),
            location: fresh_location(),
        };
        context.setup_context();
        context
    }

    // Take a pass over the program to construct the initial context
    // We only do this once and assume the invariants are maintained by construction
    // Note that a context without this makes little sense, so we can't build an "empty context"
    fn setup_context(&mut self) {
        for typ in &self.program.types {
            match typ {
                assembly::ast::TypeDecl::FFI(t) => self.ffi_type_table.push(t.clone()),
                assembly::ast::TypeDecl::Local(t) => self.local_type_table.push(t.name.clone()),
            }
        }
    }

    pub fn reset_location(&mut self) {
        self.location = fresh_location()
    }

    pub fn ffi_type_id(&self, name: &assembly::ast::FFIType) -> usize {
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

    pub fn loc_type_id(&self, typ: &assembly::ast::Type) -> usize {
        match typ {
            assembly::ast::Type::FFI(ft) => self.ffi_type_id(&ft),
            assembly::ast::Type::Local(s) => self.local_type_id(&s),
        }
    }

    pub fn remote_node_id(&self, funclet: &String, var: &String) -> usize {
        match self.remote_map.get(funclet) {
            Some(f) => match f.local.get(var) {
                Some(v) => v,
                None => panic!("Unknown local name {} in funclet {}", var, funclet),
            },
            None => panic!("Unknown funclet name {}", funclet),
        }
    }

    pub fn remote_return_id(&self, funclet: &String, var: &String) -> usize {
        match self.remote_map.get(funclet) {
            Some(f) => match f.returns.get_index(var) {
                Some(v) => v,
                None => panic!("Unknown return name {} in funclet {}", var, funclet),
            },
            None => panic!("Unknown funclet name {}", funclet),
        }
    }

    pub fn node_from_id(&self, index: usize) -> String {
        self.remote_map
            .get(self.location.funclet_id.as_str())
            .unwrap()
            .local
            .get_at_index(index)
            .unwrap()
            .clone()
    }

    pub fn node_id(&self, var: &String) -> usize {
        let funclet = &self.location.funclet_id;
        match self.remote_map.get(funclet).unwrap().local.get_index(var) {
            Some(v) => v,
            None => panic!("Unknown variable name {:?} in funclet {:?}", var, &funclet),
        }
    }

    pub fn return_id(&self, var: &String) -> usize {
        let funclet = &self.location.funclet_id;
        match self.remote_map.get(funclet).unwrap().returns.get_index(var) {
            Some(v) => v,
            None => panic!("Unknown return name {:?} in funclet {:?}", var, &funclet),
        }
    }

    pub fn remote_id(&self, funclet: &String, var: &String) -> ir::RemoteNodeId {
        ir::RemoteNodeId {
            funclet_id: self
                .funclet_indices
                .local_funclet_table
                .get(funclet)
                .unwrap()
                .clone(),
            node_id: self.remote_node_id(funclet, var),
        }
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

    pub fn add_allocation(&mut self, remote: &assembly::ast::RemoteNodeId) {
        let allocation = self.location.clone();
        self.value_function_explication_data
            .get_mut(remote.funclet_id.as_str())
            .unwrap()
            .allocate(remote.node_id.clone(), Some(allocation));
    }

    pub fn get_allocation(
        &self,
        remote: &assembly::ast::RemoteNodeId,
    ) -> Option<&assembly::ast::RemoteNodeId> {
        self.value_function_explication_data
            .get(&remote.funclet_id)
            .and_then(|f| f.explicated_allocations.get(&remote.node_id))
            .and_then(|hole| hole.as_ref())
    }

    pub fn get_funclet_data(&self, funclet: &str) -> Option<&ValueFuncletData> {
        self.value_function_explication_data.get(funclet)
    }

    pub fn get_current_funclet(&self) -> Option<&ValueFuncletData> {
        self.get_funclet_data(&self.location.funclet_id)
    }

    pub fn explicate_funclet(&mut self, name: String) {
        self.value_function_explication_data
            .insert(name, ValueFuncletData::new()); // dupes are whatever here
    }

    pub fn funclet_explicated(&mut self, name: String) -> bool {
        self.value_function_explication_data
            .contains_key(name.as_str())
    }

    pub fn get_current_extra(&self) -> Option<&ir::SchedulingFuncletExtra> {
        self.schedule_extras.get(&self.location.funclet_id)
    }

    pub fn get_value(&self, funclet: &assembly::ast::FuncletId) -> &assembly::ast::FuncletId {
        self.value_map.get(funclet).unwrap()
    }

    pub fn set_value(
        &mut self,
        schedule: assembly::ast::FuncletId,
        value: assembly::ast::FuncletId,
    ) {
        assert!(!self.value_map.contains_key(&schedule));
        self.value_map.insert(schedule, value);
    }

    pub fn node_lookup(
        &self,
        location: &assembly::ast::RemoteNodeId,
    ) -> Option<&assembly::ast::NamedNode> {
        let funclet_id = self.funclet_indices.get(&location.funclet_id);
        let node_id = self.remote_node_id(&location.funclet_id, &location.node_id);
        funclet_id.and_then(|loc| match &self.program.funclets[loc] {
            assembly::ast::FuncletDef::Local(f) => f.commands.get(loc).map(|x| x.as_ref().unwrap()),
            _ => panic!(
                "attempted to access non-local node in {}",
                location.funclet_id.clone()
            ),
        })
    }

    pub fn get_cpu_funclet(
        &self,
        name: &ExternalCpuFunctionId,
    ) -> &assembly::ast::ExternalCpuFunction {
        for funclet in &self.program.funclets {
            match funclet {
                assembly::ast::FuncletDef::ExternalCPU(f) => return f,
                _ => {}
            }
        }
        panic!("CPU funclet {} not found", name);
    }

    pub fn get_gpu_funclet(
        &self,
        name: &ExternalGpuFunctionId,
    ) -> &assembly::ast::ExternalGpuFunction {
        for funclet in &self.program.funclets {
            match funclet {
                assembly::ast::FuncletDef::ExternalGPU(f) => return f,
                _ => {}
            }
        }
        panic!("GPU funclet {} not found", name);
    }

    pub fn get_value_function(&self, name: &ValueFunctionId) -> &assembly::ast::ValueFunction {
        for funclet in &self.program.funclets {
            match funclet {
                assembly::ast::FuncletDef::ValueFunction(f) => return f,
                _ => {}
            }
        }
        panic!("Value function {} not found", name);
    }
}
