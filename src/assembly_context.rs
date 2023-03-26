// Context

use crate::assembly_ast;
use crate::ir;
use std::collections::HashMap;
use itertools::Itertools;

#[derive(Debug, Clone)]
struct TypeIndices {
    ffi_type_index: usize,
    local_type_index: usize,
}

#[derive(Debug, Clone)]
struct FuncletIndices {
    cpu_index: usize,
    gpu_index: usize,
    local_index: ir::RemoteNodeId,
    return_index: usize,
    value_function_index: usize,
}

#[derive(Debug, Clone)]
enum Indices {
    TypeIndices(TypeIndices),
    FuncletIndices(FuncletIndices),
    None,
}

#[derive(Debug)]
pub struct Context {
    ffi_type_map: HashMap<assembly_ast::FFIType, usize>,
    local_type_map: HashMap<String, usize>,
    funclet_map: HashMap<String, FuncletLocation>,
    remote_map: HashMap<String, HashMap<String, usize>>,
    local_funclet_name: Option<String>,
    command_var_name: Option<String>,
    indices: Indices,
}

#[derive(Debug, Clone)]
pub enum Location {
    Local(usize),
    FFI(usize),
}

#[derive(Debug, Clone)]
pub enum FuncletLocation {
    Local(usize),
    ValueFun(usize),
    CpuFun(usize),
    GpuFun(usize),
}

impl Location {
    pub fn unpack(&self) -> usize {
        match self {
            Location::Local(u) => *u,
            Location::FFI(u) => *u,
        }
    }
}

pub fn new_context() -> Context {
    Context {
        ffi_type_map: HashMap::new(),
        local_type_map: HashMap::new(),
        funclet_map: HashMap::new(),
        remote_map: HashMap::new(),
        local_funclet_name: None,
        command_var_name: None,
        indices: Indices::None,
    }
}

impl Context {
    pub fn initiate_type_indices(&mut self) {
        self.indices = Indices::TypeIndices(TypeIndices {
            ffi_type_index: 0,
            local_type_index: 0,
        })
    }

    pub fn initiate_funclet_indices(&mut self) {
        self.indices = Indices::FuncletIndices(FuncletIndices {
            cpu_index: 0,
            gpu_index: 0,
            local_index: ir::RemoteNodeId {
                funclet_id: 0,
                node_id: 0,
            },
            return_index: 0,
            value_function_index: 0,
        });
    }

    pub fn clear_indices(&mut self) {
        self.indices = Indices::None;
        self.local_funclet_name = None;
    }

    fn setup_add(&mut self, name: String) -> &mut FuncletIndices {
        let indices = match &mut self.indices {
            Indices::FuncletIndices(t) => t,
            _ => panic!("Invalid access attempt {:?}", name),
        };
        if self.funclet_map.contains_key(name.as_str()) {
            panic!("Duplicate funclet name {} ", name);
        };
        indices
    }

    pub fn add_ffi_type(&mut self, name: assembly_ast::FFIType) {
        let indices = match &mut self.indices {
            Indices::TypeIndices(t) => t,
            _ => panic!("Invalid access attempt {:?}", name),
        };
        let index = indices.ffi_type_index;
        self.ffi_type_map.insert(name, index);
        indices.ffi_type_index += 1;
    }

    pub fn add_local_type(&mut self, name: String) {
        let indices = match &mut self.indices {
            Indices::TypeIndices(t) => t,
            _ => panic!("Invalid access attempt {:?}", name),
        };
        let index = indices.local_type_index;
        self.local_type_map.insert(name, index);
        indices.local_type_index += 1;
    }

    pub fn add_cpu_funclet(&mut self, name: String) {
        // stupid borrow check, can't easily make a helper
        let indices = match &mut self.indices {
            Indices::FuncletIndices(t) => t,
            _ => panic!("Invalid access attempt {:?}", name),
        };
        if self.funclet_map.contains_key(name.as_str()) {
            panic!("Duplicate funclet name {} ", name);
        };
        // let indices = self.setup_add(name.clone());
        let index = indices.cpu_index;
        // cause of this tbc
        self.funclet_map
            .insert(name.clone(), FuncletLocation::CpuFun(index));
        indices.cpu_index += 1;
    }

    pub fn add_gpu_funclet(&mut self, name: String) {
        let indices = match &mut self.indices {
            Indices::FuncletIndices(t) => t,
            _ => panic!("Invalid access attempt {:?}", name),
        };
        if self.funclet_map.contains_key(name.as_str()) {
            panic!("Duplicate funclet name {} ", name);
        };
        let index = indices.gpu_index;
        self.funclet_map
            .insert(name.clone(), FuncletLocation::GpuFun(index));
        indices.gpu_index += 1;
    }

    pub fn add_local_funclet(&mut self, name: String) {
        let indices = match &mut self.indices {
            Indices::FuncletIndices(t) => t,
            _ => panic!("Invalid access attempt {:?}", name),
        };
        if self.funclet_map.contains_key(name.as_str()) {
            panic!("Duplicate funclet name {} ", name);
        };
        let index = indices.local_index.funclet_id;
        self.funclet_map
            .insert(name.clone(), FuncletLocation::Local(index));
        indices.local_index.node_id = 0;
        indices.return_index = 0;
        indices.local_index.funclet_id += 1;
        self.update_local_funclet(name);
    }

    pub fn add_value_function(&mut self, name: String) {
        let indices = match &mut self.indices {
            Indices::FuncletIndices(t) => t,
            _ => panic!("Invalid access attempt {:?}", name),
        };
        if self.funclet_map.contains_key(name.as_str()) {
            panic!("Duplicate funclet name {} ", name);
        };
        let index = indices.value_function_index;
        self.funclet_map
            .insert(name.clone(), FuncletLocation::ValueFun(index));
        indices.value_function_index += 1;
    }

    pub fn update_local_funclet(&mut self, name: String) {
        self.local_funclet_name = Some(name);
    }

    pub fn clear_local_funclet(&mut self) {
        self.local_funclet_name = None;
    }

    pub fn funclet_name(&self) -> String {
        self.local_funclet_name.as_ref().unwrap().clone()
    }

    pub fn add_node(&mut self, name: String) {
        let indices = match &mut self.indices {
            Indices::FuncletIndices(t) => t,
            _ => panic!("Invalid access attempt {:?}", name),
        };
        if name == "_" {
            // ignore throwaway except to update index
            indices.local_index.node_id += 1;
            return;
        }
        let index = indices.local_index.node_id;
        let funclet = self.local_funclet_name.as_ref().unwrap();
        let map = self
            .remote_map
            .entry(funclet.clone())
            .or_insert(HashMap::new());
        map.insert(name, index);
        indices.local_index.node_id += 1;
    }

    pub fn add_return_node(&mut self, name: String) {
        let indices = match &mut self.indices {
            Indices::FuncletIndices(t) => t,
            _ => panic!("Invalid access attempt {:?}", name),
        };
        if name == "_" {
            // ignore throwaway except to update index
            indices.return_index += 1;
            return;
        }
        let index = indices.return_index;
        let funclet = self.local_funclet_name.as_ref().unwrap();
        let map = self
            .remote_map
            .entry(funclet.clone())
            .or_insert(HashMap::new());
        map.insert(name, index);
        indices.return_index += 1;
    }

    pub fn ffi_type_id(&mut self, name: &assembly_ast::FFIType) -> &usize {
        match self.ffi_type_map.get(name) {
            Some(i) => i,
            None => panic!("Un-indexed FFI type {:?}", name),
        }
    }

    pub fn local_type_id(&mut self, name: String) -> &usize {
        match self.local_type_map.get(name.as_str()) {
            Some(t) => t,
            None => panic!("Unknown local type {:?}", name),
        }
    }

    pub fn loc_type_id(&mut self, typ: assembly_ast::Type) -> &usize {
        match typ {
            assembly_ast::Type::FFI(ft) => self.ffi_type_id(&ft),
            assembly_ast::Type::Local(s) => self.local_type_id(s),
        }
    }

    pub fn funclet_id_unwrap(&self, name: String) -> &usize {
        match self.funclet_id(name) {
            FuncletLocation::Local(n) => n,
            FuncletLocation::GpuFun(n) => n,
            FuncletLocation::CpuFun(n) => n,
            FuncletLocation::ValueFun(n) => n,
        }
    }

    pub fn funclet_id(&self, name: String) -> &FuncletLocation {
        match self.funclet_map.get(name.as_str()) {
            Some(f) => f,
            None => panic!("Unknown funclet name {:?}", name),
        }
    }

    pub fn local_funclet_id(&mut self, name: String) -> &usize {
        match self.funclet_id(name.clone()) {
            FuncletLocation::Local(u) => u,
            _ => panic!("Not a local funclet {}", name),
        }
    }
    pub fn cpu_funclet_id(&mut self, name: String) -> &usize {
        match self.funclet_id(name.clone()) {
            FuncletLocation::CpuFun(u) => u,
            _ => panic!("Not a cpu funclet {}", name),
        }
    }
    pub fn gpu_funclet_id(&mut self, name: String) -> &usize {
        match self.funclet_id(name.clone()) {
            FuncletLocation::GpuFun(u) => u,
            _ => panic!("Not a gpu funclet {}", name),
        }
    }
    pub fn value_function_id(&mut self, name: String) -> &usize {
        match self.funclet_id(name.clone()) {
            FuncletLocation::ValueFun(u) => u,
            _ => panic!("Not a value function {}", name),
        }
    }

    pub fn remote_node_id(&mut self, funclet: String, var: String) -> &usize {
        match self.remote_map.get(funclet.as_str()) {
            Some(f) => match f.get(var.as_str()) {
                Some(v) => v,
                None => panic!("Unknown var name {} in funclet {}", var, funclet),
            },
            None => panic!("Unknown funclet name {}", funclet),
        }
    }

    pub fn node_id(&mut self, var: String) -> &usize {
        match self
            .remote_map
            .get(self.funclet_name().as_str())
            .unwrap()
            .get(var.as_str())
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
            funclet_id: *self.local_funclet_id(funclet.clone()),
            node_id: *self.remote_node_id(funclet, var),
        }
    }

    // ffi_type_map: HashMap<assembly_ast::FFIType, usize>,
    // local_type_map: HashMap<String, usize>,
    // funclet_map: HashMap<String, FuncletLocation>,
    // remote_map: HashMap<String, HashMap<String, usize>>,

    pub fn ffi_type_map_dump(&self) -> Vec<&assembly_ast::FFIType> {
        self.ffi_type_map.keys().collect_vec()
    }

    pub fn local_type_map_dump(&self) -> Vec<&String> {
        self.local_type_map.keys().collect_vec()
    }

    pub fn funclet_map_dump(&self) -> Vec<&String> {
        self.funclet_map.keys().collect_vec()
    }

    pub fn remote_map_dump(&self) -> Vec<&String> {
        self.remote_map.keys().collect_vec()
    }

    pub fn remote_map_inner_dump(&self, name : &String) -> Vec<&String> {
        self.remote_map.get(name).unwrap().keys().collect_vec()
    }
}
