// Context

use std::collections::HashMap;
use crate::ir;
use crate::assembly::ast;

#[derive(Debug, Clone)]
struct TypeIndices {
    type_index : usize
}

#[derive(Debug, Clone)]
struct FuncletIndices {
    cpu_index: usize,
    gpu_index: usize,
    local_index: ir::RemoteNodeId,
}

#[derive(Debug, Clone)]
enum Indices {
    TypeIndices(TypeIndices),
    FuncletIndices(FuncletIndices),
    None
}

#[derive(Debug)]
pub struct Context {
    ffi_type_map : HashMap<String, usize>,
    local_type_map : HashMap<String, usize>,
    cpu_funclet_map : HashMap<String, usize>,
    gpu_funclet_map : HashMap<String, usize>,
    local_funclet_map : HashMap<String, usize>,
    remote_map : HashMap<String, HashMap<String, usize>>,
    local_funclet_name: Option<String>,
    indices : Indices
}

#[derive(Debug, Clone)]
pub enum Location {
    Local(usize),
    FFI(usize)
}

impl Location {
    pub fn unpack(&self) -> usize {
        match self {
            Location::Local(u) => *u,
            Location::FFI(u) => *u
        }
    }
}

pub fn new_context() -> Context {
    Context {
        ffi_type_map: HashMap::new(),
        local_type_map: HashMap::new(),
        cpu_funclet_map: HashMap::new(),
        gpu_funclet_map: HashMap::new(),
        local_funclet_map: HashMap::new(),
        remote_map: HashMap::new(),
        local_funclet_name: None,
        indices: Indices::None
    }
}

impl Context {

    pub fn initiate_type_indices(&mut self) {
        self.indices = Indices::TypeIndices(TypeIndices {
            type_index : 0
        })
    }

    pub fn initiate_funclet_indices(&mut self) {
        self.indices = Indices::FuncletIndices(FuncletIndices {
            cpu_index: 0,
            gpu_index: 0,
            local_index: ir::RemoteNodeId { funclet_id: 0, node_id: 0 },
        });
    }

    pub fn clear_indices(&mut self) {
        self.indices = Indices::None;
        self.local_funclet_name = None;
    }

    pub fn add_ffi_type(&mut self, name : String) {
        let indices = match &mut self.indices {
            Indices::TypeIndices(t) => t,
            _ => panic!(format!("Invalid access attempt {:?}", name))
        };
        let index = indices.type_index;
        self.ffi_type_map.insert(name, index);
        indices.type_index += 1;
    }

    pub fn add_local_type(&mut self, name : String) {
        let indices = match &mut self.indices {
            Indices::TypeIndices(t) => t,
            _ => panic!(format!("Invalid access attempt {:?}", name))
        };
        let index = indices.type_index;
        self.local_type_map.insert(name, index);
        indices.type_index += 1;
    }

    pub fn add_cpu_funclet(&mut self, name : String) {
        let indices = match &mut self.indices {
            Indices::FuncletIndices(t) => t,
            _ => panic!(format!("Invalid access attempt {:?}", name))
        };
        let index = indices.cpu_index;
        self.cpu_funclet_map.insert(name.clone(), index);
        indices.cpu_index += 1;
    }

    pub fn add_gpu_funclet(&mut self, name : String) {
        let indices = match &mut self.indices {
            Indices::FuncletIndices(t) => t,
            _ => panic!(format!("Invalid access attempt {:?}", name))
        };
        let index = indices.gpu_index;
        self.gpu_funclet_map.insert(name.clone(), index);
        indices.gpu_index += 1;
    }

    pub fn add_local_funclet(&mut self, name : String) {
        let indices = match &mut self.indices {
            Indices::FuncletIndices(t) => t,
            _ => panic!(format!("Invalid access attempt {:?}", name))
        };
        let index = indices.local_index.funclet_id;
        self.local_funclet_map.insert(name.clone(), index);
        indices.local_index.node_id = 0;
        indices.local_index.funclet_id += 1;
        self.update_local_funclet(name);
    }

    pub fn update_local_funclet(&mut self, name : String) {
        self.local_funclet_name = Some(name);
    }

    pub fn clear_local_funclet(&mut self) {
        self.local_funclet_name = None;
    }

    pub fn funclet_name(&self) -> String {
        self.local_funclet_name.as_ref().unwrap().clone()
    }

    pub fn add_node(&mut self, name : String) {
        let indices = match &mut self.indices {
            Indices::FuncletIndices(t) => t,
            _ => panic!(format!("Invalid access attempt {:?}", name))
        };
        let index = indices.local_index.node_id;
        let funclet = self.local_funclet_name.as_ref().unwrap();
        let map = self.remote_map
            .entry(funclet.clone())
            .or_insert(HashMap::new());
        map.insert(name, index);
        indices.local_index.node_id += 1;
    }

    pub fn ffi_type_id(&mut self, name : String) -> &usize { self.ffi_type_map.get(name.as_str()).unwrap() }

    pub fn local_type_id(&mut self, name : String) -> &usize { self.local_type_map.get(name.as_str()).unwrap() }

    pub fn loc_type_id(&mut self, typ : ast::Type) -> &usize {
        match typ {
            ast::Type::FFI(s) => self.ffi_type_id(s),
            ast::Type::Local(s) => self.local_type_id(s)
        }
    }

    pub fn funclet_id(&mut self, name : String) -> Location {
        match self.local_funclet_map.get(name.as_str()) {
            Some(id) => Location::Local(*id),
            None => match self.cpu_funclet_map.get(name.as_str()) {
                Some (id) => Location::FFI(*id),
                None => Location::FFI(*self.cpu_funclet_map.get(name.as_str()).unwrap())
            }
        }
    }

    pub fn local_funclet_id(&mut self, name : String) -> &usize {
        self.local_funclet_map.get(name.as_str()).unwrap()
    }
    pub fn cpu_funclet_id(&mut self, name : String) -> &usize { self.cpu_funclet_map.get(name.as_str()).unwrap() }
    pub fn gpu_funclet_id(&mut self, name : String) -> &usize { self.cpu_funclet_map.get(name.as_str()).unwrap() }

    pub fn remote_node_id(&mut self, funclet : String, var : String) -> &usize {
        self.remote_map.get(funclet.as_str()).unwrap().get(var.as_str()).unwrap()
    }

    pub fn node_id(&mut self, var : String) -> &usize {
        self.remote_map.get(self.funclet_name().as_str()).unwrap().get(var.as_str()).unwrap()
    }

    pub fn remote_id(&mut self, funclet : String, var : String) -> ir::RemoteNodeId {
        ir::RemoteNodeId {
            funclet_id: *self.local_funclet_id(funclet.clone()),
            node_id: *self.remote_node_id(funclet, var)
        }
    }
}