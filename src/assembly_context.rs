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
pub struct Table<T, U>
where
    T: Eq + Hash + Debug + Clone,
{
    values: HashMap<T, U>,
    indices: Vec<T>,
}

#[derive(Debug)]
struct NodeTable {
    // local names and return names such as [%out : i64] or whatever
    local: Table<String, ()>,
    returns: Table<String, ()>,
}

#[derive(Debug)]
pub struct Context {
    ffi_type_table: Table<assembly_ast::FFIType, ()>,
    local_type_table: Table<String, ()>,
    funclet_kind_map: HashMap<String, FuncletLocation>,
    external_funclet_table: Table<String, ()>,
    local_funclet_table: Table<String, ()>,
    value_function_table: Table<String, ()>,
    remote_map: HashMap<String, NodeTable>,
    current_funclet_name: Option<String>,
    command_var_name: Option<String>,
    location: Option<ir::RemoteNodeId>,
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

// a Table is basically a vector with no dupes
impl<T, U> Table<T, U>
where
    T: Eq + Hash + Debug + Clone,
{
    pub fn new() -> Table<T, U> {
        Table {
            values: HashMap::new(),
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

    pub fn push(&mut self, val: T, data: U) {
        if self.values.contains_key(&val) {
            panic!("Duplicate add of {:?}", val)
        }
        self.values.insert(val.clone(), data);
        self.indices.push(val);
    }

    pub fn insert(&mut self, index: usize, val: T, data: U) {
        if self.values.contains_key(&val) {
            panic!("Duplicate add of {:?}", val)
        }
        self.values.insert(val.clone(), data);
        self.indices.insert(index, val);
    }

    pub fn get(&self, val: &T) -> Option<(&U, usize)> {
        // no need to actually check the Hashset, that's just to avoid dupes
        for item in itertools::enumerate(&self.indices) {
            if item.1 == val {
                return Some((self.values.get(val).unwrap(), item.0));
            }
        }
        return None;
    }

    pub fn get_index(&self, val: &T) -> Option<usize> {
        self.get(val).map(|x| x.1)
    }

    pub fn len(&mut self) -> usize {
        return self.indices.len();
    }
}

impl Context {
    pub fn new() -> Context {
        Context {
            ffi_type_table: Table::new(),
            local_type_table: Table::new(),
            funclet_kind_map: HashMap::new(),
            local_funclet_table: Table::new(),
            external_funclet_table: Table::new(),
            value_function_table: Table::new(),
            remote_map: HashMap::new(),
            current_funclet_name: None,
            command_var_name: None,
            location: None,
        }
    }

    pub fn reset_location(&mut self) {
        self.location = None;
        self.current_funclet_name = None;
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
        self.current_funclet_name = Some(name.clone());
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
    }

    pub fn add_value_function(&mut self, name: String) {
        self.funclet_kind_map
            .insert(name.clone(), FuncletLocation::Value);
        self.value_function_table.push(name, ());
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
                    table.local.push(name, ())
                }
            }
        }
    }

    pub fn add_return(&mut self, name: String) {
        match self.remote_map.get_mut(&self.funclet_name()) {
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

    pub fn node_id(&self, var: String) -> usize {
        match self
            .remote_map
            .get(self.funclet_name().as_str())
            .unwrap()
            .local
            .get_index(&var)
        {
            Some(v) => v,
            None => panic!(
                "Unknown variable name {:?} in funclet {:?}",
                var,
                self.funclet_name()
            ),
        }
    }

    pub fn return_id(&self, var: String) -> usize {
        match self
            .remote_map
            .get(self.funclet_name().as_str())
            .unwrap()
            .returns
            .get_index(&var)
        {
            Some(v) => v,
            None => panic!(
                "Unknown return name {:?} in funclet {:?}",
                var,
                self.funclet_name()
            ),
        }
    }

    pub fn remote_id(&self, funclet: String, var: String) -> ir::RemoteNodeId {
        ir::RemoteNodeId {
            funclet_id: self.local_funclet_id(funclet.clone()).clone(),
            node_id: self.remote_node_id(funclet, var),
        }
    }
}
