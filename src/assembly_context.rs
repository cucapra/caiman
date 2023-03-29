// Explication Context

// So that we have a different context for exactly explication is a bit weird
// The intent is to separate logic here
// In our regular 'ol parsing context, we explicitly don't want to allow for re-ordering indices
// Here, we do want this ability, along with not wanting to allow messing with non-funclets

use crate::assembly_ast;
use crate::assembly_context;
use crate::ir;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Debug)]
struct Table<T>
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
struct FuncletIndices {
    // keep track of the different kinds of indices for now
    local: usize,
    external: usize,
    value: usize,
}

#[derive(Debug)]
pub struct Context {
    ffi_type_table: Table<assembly_ast::FFIType>,
    local_type_table: Table<String>,
    funclet_table: HashMap<String, FuncletLocation>,
    funclet_indices: FuncletIndices,
    remote_map: HashMap<String, NodeTable>,
    current_funclet_name: Option<String>,
    command_var_name: Option<String>,
    location: Option<ir::RemoteNodeId>,
}

#[derive(Debug, Clone)]
pub enum FuncletLocation {
    Local(usize),
    ValueFun(usize),
    CpuFun(usize),
    GpuFun(usize),
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

fn new_table<T>() -> Table<T>
where
    T: Eq + Hash + Debug + Clone,
{
    Table {
        values: HashSet::new(),
        indices: Vec::new(),
    }
}

// a Table is basically a vector with no dupes
impl<T> Table<T>
where
    T: Eq + Hash + Debug + Clone,
{
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
        self.values.insert(val.clone()); // laaaazy
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

    pub fn len(&mut self) -> usize {
        return self.indices.len();
    }
}

pub fn new_context() -> Context {
    Context {
        ffi_type_table: new_table(),
        local_type_table: new_table(),
        funclet_table: HashMap::new(),
        funclet_indices: FuncletIndices {
            local: 0,
            external: 0,
            value: 0,
        },
        remote_map: HashMap::new(),
        current_funclet_name: None,
        command_var_name: None,
        location: None,
    }
}

impl Context {
    pub fn reset_location(&mut self) {
        self.location = None;
        self.current_funclet_name = None;
    }

    // for use by the explicator
    pub fn add_ffi_type(&mut self, t: assembly_ast::FFIType) {
        self.ffi_type_table.push(t);
    }

    pub fn add_local_type(&mut self, name: String) {
        self.local_type_table.push(name);
    }

    pub fn add_cpu_funclet(&mut self, name: String) {
        self.funclet_table
            .insert(name, FuncletLocation::CpuFun(self.funclet_indices.external));
        self.funclet_indices.external += 1;
    }

    pub fn add_gpu_funclet(&mut self, name: String) {
        self.funclet_table
            .insert(name, FuncletLocation::GpuFun(self.funclet_indices.external));
        self.funclet_indices.external += 1;
    }

    pub fn add_local_funclet(&mut self, name: String) {
        self.current_funclet_name = Some(name.clone());
        self.funclet_table.insert(
            name.clone(),
            FuncletLocation::Local(self.funclet_indices.local),
        );
        self.funclet_indices.local += 1;
        self.remote_map.insert(
            name,
            NodeTable {
                local: new_table(),
                returns: new_table(),
            },
        );
    }

    pub fn add_value_function(&mut self, name: String) {
        self.funclet_table
            .insert(name, FuncletLocation::ValueFun(self.funclet_indices.value));
        self.funclet_indices.value += 1;
    }

    pub fn advance_local_funclet(&mut self, name: String) {
        self.location = match self.location {
            None => Some(ir::RemoteNodeId {
                funclet_id: 0,
                node_id: 0,
            }),
            Some(v) => Some(ir::RemoteNodeId {
                funclet_id: v.funclet_id + 1,
                node_id: 0,
            }),
        };
        self.current_funclet_name = Some(name);
    }

    pub fn clear_local_funclet(&mut self) {
        self.current_funclet_name = None;
    }

    pub fn funclet_name(&self) -> String {
        self.current_funclet_name.as_ref().unwrap().clone()
    }

    pub fn add_node(&mut self, name: String) {
        match self.remote_map.get_mut(&self.funclet_name()) {
            None => panic!("Invalid funclet name {:?}", name),
            Some(table) => {
                if name == "_" {
                    table.local.dummy_push(name)
                } else {
                    table.local.push(name)
                }
            }
        }
    }

    pub fn add_return(&mut self, name: String) {
        match self.remote_map.get_mut(&self.funclet_name()) {
            None => panic!("Invalid funclet name {:?}", name),
            Some(table) => table.returns.push(name),
        }
    }

    // pub fn insert_node(&mut self, name: String) {
    //     match self.remote_map.get_mut(&self.funclet_name()) {
    //         None => panic!("Invalid funclet name {:?}", name),
    //         Some(table) => table.insert(self.location.unwrap().node_id, name)
    //     }
    //     self.location.unwrap().node_id += 1;
    // }

    pub fn ffi_type_id(&mut self, name: &assembly_ast::FFIType) -> usize {
        match self.ffi_type_table.get(name) {
            Some(i) => i,
            None => panic!("Un-indexed FFI type {:?}", name),
        }
    }

    pub fn local_type_id(&mut self, name: &String) -> usize {
        match self.local_type_table.get(name) {
            Some(t) => t,
            None => panic!("Unknown local type {:?}", name),
        }
    }

    pub fn loc_type_id(&mut self, typ: assembly_ast::Type) -> usize {
        match typ {
            assembly_ast::Type::FFI(ft) => self.ffi_type_id(&ft),
            assembly_ast::Type::Local(s) => self.local_type_id(&s),
        }
    }

    pub fn funclet_location(&mut self, name: &String) -> &FuncletLocation {
        match self.funclet_table.get(name) {
            Some(f) => f,
            None => panic!("Unknown funclet name {:?}", name),
        }
    }

    pub fn funclet_id(&mut self, name: &String) -> &usize {
        match self.funclet_location(name) {
            FuncletLocation::Local(n) => n,
            FuncletLocation::ValueFun(n) => n,
            FuncletLocation::CpuFun(n) => n,
            FuncletLocation::GpuFun(n) => n,
        }
    }

    pub fn local_funclet_id(&mut self, name: String) -> &usize {
        match self.funclet_location(&name) {
            FuncletLocation::Local(n) => n,
            _ => panic!("Not a local funclet {}", name),
        }
    }
    pub fn cpu_funclet_id(&mut self, name: String) -> &usize {
        match self.funclet_location(&name) {
            FuncletLocation::CpuFun(n) => n,
            _ => panic!("Not a cpu funclet {}", name),
        }
    }
    pub fn gpu_funclet_id(&mut self, name: String) -> &usize {
        match self.funclet_location(&name) {
            FuncletLocation::GpuFun(n) => n,
            _ => panic!("Not a gpu funclet {}", name),
        }
    }
    pub fn value_function_id(&mut self, name: String) -> &usize {
        match self.funclet_location(&name) {
            FuncletLocation::ValueFun(n) => n,
            _ => panic!("Not a value funclet {}", name),
        }
    }

    pub fn remote_node_id(&mut self, funclet: String, var: String) -> usize {
        match self.remote_map.get(funclet.as_str()) {
            Some(f) => match f.local.get(&var) {
                Some(v) => v,
                None => panic!("Unknown local name {} in funclet {}", var, funclet),
            },
            None => panic!("Unknown funclet name {}", funclet),
        }
    }

    pub fn remote_return_id(&mut self, funclet: String, var: String) -> usize {
        match self.remote_map.get(funclet.as_str()) {
            Some(f) => match f.returns.get(&var) {
                Some(v) => v,
                None => panic!("Unknown return name {} in funclet {}", var, funclet),
            },
            None => panic!("Unknown funclet name {}", funclet),
        }
    }

    pub fn node_id(&mut self, var: String) -> usize {
        match self
            .remote_map
            .get(self.funclet_name().as_str())
            .unwrap()
            .local
            .get(&var)
        {
            Some(v) => v,
            None => panic!(
                "Unknown variable name {:?} in funclet {:?}",
                var,
                self.funclet_name()
            ),
        }
    }

    pub fn return_id(&mut self, var: String) -> usize {
        match self
            .remote_map
            .get(self.funclet_name().as_str())
            .unwrap()
            .returns
            .get(&var)
        {
            Some(v) => v,
            None => panic!(
                "Unknown return name {:?} in funclet {:?}",
                var,
                self.funclet_name()
            ),
        }
    }

    pub fn remote_id(&mut self, funclet: String, var: String) -> ir::RemoteNodeId {
        ir::RemoteNodeId {
            funclet_id: self.local_funclet_id(funclet.clone()).clone(),
            node_id: self.remote_node_id(funclet, var),
        }
    }
}
