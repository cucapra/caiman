// Explication Context

// So that we have a different context for exactly explication is a bit weird
// The intent is to separate logic here
// In our regular 'ol parsing context, we explicitly don't want to allow for re-ordering indices
// Here, we do want this ability, along with not wanting to allow messing with non-funclets

use crate::assembly_ast;
use crate::assembly_context;
use crate::ir;
use std::collections::{HashSet, HashMap};
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Debug)]
struct Table<T> where T: Eq + Hash + Debug + Clone {
    values : HashSet<T>,
    indices : Vec<T>
}

#[derive(Debug)]
pub struct Context {
    ffi_type_table: Table<assembly_ast::FFIType>,
    local_type_table: Table<String>,
    funclet_table: Table<String>,
    funclet_location_map: HashMap<String, FuncletLocation>,
    remote_map: HashMap<String, Table<String>>,
    current_funclet_name: Option<String>,
    command_var_name: Option<String>,
    location: ir::RemoteNodeId,
}

#[derive(Debug, Clone)]
pub enum FuncletLocation {
    Local,
    ValueFun,
    CpuFun,
    GpuFun,
}

pub struct FuncletInformation {
    location: FuncletLocation,
    index: usize
}

fn new_table<T>() -> Table<T> where T: Eq + Hash + Debug + Clone {
    Table {
        values: HashSet::new(),
        indices: Vec::new()
    }
}

// a Table is basically a vector with no dupes
impl<T> Table<T> where T: Eq + Hash + Debug + Clone {
    pub fn contains(&mut self, val : &T) -> bool {
        self.values.contains(val)
    }

    pub fn push(&mut self, val : T) {
        if self.values.contains(&val) {
            panic!("Duplicate add of {:?}", val)
        }
        self.values.insert(val.clone()); // laaaazy
        self.indices.push(val);
    }

    pub fn insert(&mut self, index : usize, val : T) {
        if self.values.contains(&val) {
            panic!("Duplicate add of {:?}", val)
        }
        self.values.insert(val.clone());
        self.indices.insert(index, val);
    }

    pub fn get(&self, val : &T) -> Option<usize> {
        // no need to actually check the Hashset, that's just to avoid dupes
        for item in itertools::enumerate(&self.indices) {
            if item.1 == val {
                return Some(item.0)
            }
        }
        return None
    }

    pub fn len(&mut self) -> usize {
        return self.indices.len()
    }
}

pub fn new_context() -> Context {
    Context {
        ffi_type_table: new_table(),
        local_type_table: new_table(),
        funclet_table: new_table(),
        funclet_location_map: HashMap::new(),
        remote_map: HashMap::new(),
        current_funclet_name: None,
        command_var_name: None,
        location: ir::RemoteNodeId{ funclet_id: 0, node_id: 0 },
    }
}

impl Context {

    pub fn reset_indices(&mut self) {
        self.location = ir::RemoteNodeId{ funclet_id: 0, node_id: 0 };
        self.current_funclet_name = None;
    }

    pub fn add_funclet(&mut self) -> String {
        let name = self.funclet_table.len().to_string() + "$Gen"; // free name
        self.funclet_table.push(name.clone());
        name
    }

    pub fn advance_local_funclet(&mut self, name: String) {
        self.location.funclet_id += 1;
        assert_eq!(self.location.funclet_id, self.funclet_table.get(&name).unwrap());
        self.current_funclet_name = Some(name);
        self.location.node_id = 0;
    }

    pub fn clear_local_funclet(&mut self) {
        self.current_funclet_name = None;
    }

    pub fn funclet_name(&self) -> String {
        self.current_funclet_name.as_ref().unwrap().clone()
    }

    pub fn insert_node(&mut self, name: String) {
        match self.remote_map.get_mut(&self.funclet_name()) {
            None => panic!("Invalid funclet name {:?}", name),
            Some(table) => table.insert(self.location.node_id, name)
        }
        self.location.node_id += 1;
    }

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

    pub fn funclet_id(&mut self, name: String) -> usize {
        match self.funclet_table.get(&name) {
            Some(f) => f,
            None => panic!("Unknown funclet name {:?}", name),
        }
    }

    pub fn local_funclet_id(&mut self, name: String) -> usize {
        match self.funclet_location_map.get(&name) {
            Some(FuncletLocation::Local) => self.funclet_table.get(&name).unwrap(),
            _ => panic!("Not a local funclet {}", name),
        }
    }
    pub fn cpu_funclet_id(&mut self, name: String) -> usize {
        match self.funclet_location_map.get(&name) {
            Some(FuncletLocation::CpuFun) => self.funclet_table.get(&name).unwrap(),
            _ => panic!("Not a cpu funclet {}", name),
        }
    }
    pub fn gpu_funclet_id(&mut self, name: String) -> usize {
        match self.funclet_location_map.get(&name) {
            Some(FuncletLocation::GpuFun) => self.funclet_table.get(&name).unwrap(),
            _ => panic!("Not a gpu funclet {}", name),
        }
    }
    pub fn value_function_id(&mut self, name: String) -> usize {
        match self.funclet_location_map.get(&name) {
            Some(FuncletLocation::ValueFun) => self.funclet_table.get(&name).unwrap(),
            _ => panic!("Not a value functin {}", name),
        }
    }

    pub fn remote_node_id(&mut self, funclet: String, var: String) -> usize {
        match self.remote_map.get(funclet.as_str()) {
            Some(f) => match f.get(&var) {
                Some(v) => v,
                None => panic!("Unknown var name {} in funclet {}", var, funclet),
            },
            None => panic!("Unknown funclet name {}", funclet),
        }
    }

    pub fn node_id(&mut self, var: String) -> usize {
        match self
            .remote_map
            .get(self.funclet_name().as_str())
            .unwrap()
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

    pub fn remote_id(&mut self, funclet: String, var: String) -> ir::RemoteNodeId {
        ir::RemoteNodeId {
            funclet_id: self.local_funclet_id(funclet.clone()),
            node_id: self.remote_node_id(funclet, var),
        }
    }
}