// Context

use std::collections::HashMap;
use crate::ir;

#[derive(Debug, Clone)]
struct TypeIndices {
    ffi_index : usize,
    local_index : usize
}

#[derive(Debug, Clone)]
struct FuncletIndices {
    ffi_index: usize,
    local_index: ir::RemoteNodeId,
    local_funclet_name: Option<String>,
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
    ffi_funclet_map : HashMap<String, usize>,
    local_funclet_map : HashMap<String, usize>,
    remote_map : HashMap<String, HashMap<String, usize>>,
    indices : Indices
}

#[derive(Debug, Clone)]
pub enum Location {
    Local(usize),
    FFI(usize)
}

pub fn new_context() -> Context {
    Context {
        ffi_type_map: HashMap::new(),
        local_type_map: HashMap::new(),
        ffi_funclet_map: HashMap::new(),
        local_funclet_map: HashMap::new(),
        remote_map: HashMap::new(),
        indices: Indices::None
    }
}

impl Context {

    pub fn initiate_type_indices(&mut self) {
        self.indices = Indices::TypeIndices(TypeIndices {
            ffi_index: 0,
            local_index: 0,
        })
    }

    pub fn initiate_funclet_indices(&mut self) {
        self.indices = Indices::FuncletIndices(FuncletIndices {
            ffi_index: 0,
            local_index: ir::RemoteNodeId { funclet_id: 0, node_id: 0 },
            local_funclet_name: None,
        });
    }

    pub fn clear_indices(&mut self) {
        self.indices = Indices::None;
    }

    pub fn add_ffi_type(&mut self, name : String) {
        let indices = match &mut self.indices {
            Indices::TypeIndices(t) => t,
            _ => panic!("Invalid access attempt")
        };
        let index = indices.ffi_index;
        self.ffi_type_map.insert(name, index);
        indices.ffi_index += 1;
    }

    pub fn add_local_type(&mut self, name : String) {
        let indices = match &mut self.indices {
            Indices::TypeIndices(t) => t,
            _ => panic!("Invalid access attempt")
        };
        let index = indices.local_index;
        self.local_type_map.insert(name, index);
        indices.local_index += 1;
    }

    pub fn add_ffi_funclet(&mut self, name : String) {
        let indices = match &mut self.indices {
            Indices::FuncletIndices(t) => t,
            _ => panic!("Invalid access attempt")
        };
        let index = indices.ffi_index;
        self.ffi_funclet_map.insert(name.clone(), index);
        indices.ffi_index += 1;
    }

    pub fn add_local_funclet(&mut self, name : String) {
        let indices = match &mut self.indices {
            Indices::FuncletIndices(t) => t,
            _ => panic!("Invalid access attempt")
        };
        let index = indices.local_index.funclet_id;
        self.local_funclet_map.insert(name.clone(), index);
        indices.local_index.node_id = 0;
        indices.local_index.funclet_id += 1;
        indices.local_funclet_name = Some(name);
    }

    pub fn funclet_name(&self) -> String {
        let indices = match &self.indices {
            Indices::FuncletIndices(t) => t,
            _ => panic!("Invalid access attempt")
        };
        indices.local_funclet_name.as_ref().unwrap().clone()
    }

    pub fn add_node(&mut self, name : String) {
        let indices = match &mut self.indices {
            Indices::FuncletIndices(t) => t,
            _ => panic!("Invalid access attempt")
        };
        let index = indices.local_index.node_id;
        let funclet = indices.local_funclet_name.as_ref().unwrap();
        let map = self.remote_map
            .entry(funclet.clone())
            .or_insert(HashMap::new());
        map.insert(name, index);
        indices.local_index.node_id += 1;
    }

    pub fn ffi_type_id(&mut self, name : String) -> &usize { self.ffi_type_map.get(name.as_str()).unwrap() }

    pub fn local_type_id(&mut self, name : String) -> &usize { self.local_type_map.get(name.as_str()).unwrap() }

    pub fn funclet_id(&mut self, name : String) -> Location {
        match self.local_funclet_map.get(name.as_str()) {
            Some(id) => Location::Local(*id),
            None => Location::FFI(*self.ffi_funclet_map.get(name.as_str()).unwrap())
        }
    }

    pub fn local_funclet_id(&mut self, name : String) -> &usize { self.local_funclet_map.get(name.as_str()).unwrap() }

    pub fn ffi_funclet_id(&mut self, name : String) -> &usize { self.ffi_funclet_map.get(name.as_str()).unwrap() }

    pub fn node_id(&mut self, funclet : String, var : String) -> &usize {
        self.remote_map.get(funclet.as_str()).unwrap().get(var.as_str()).unwrap()
    }

    pub fn current_node_id(&mut self, var : String) -> &usize {
        self.remote_map.get(self.funclet_name().as_str()).unwrap().get(var.as_str()).unwrap()
    }

    pub fn remote_id(&mut self, funclet : String, var : String) -> ir::RemoteNodeId {
        ir::RemoteNodeId {
            funclet_id: *self.local_funclet_id(funclet.clone()),
            node_id: *self.node_id(funclet, var)
        }
    }
}