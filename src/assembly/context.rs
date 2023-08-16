use crate::assembly::ast;
use crate::assembly::ast::Hole;
use crate::assembly::ast::{
    ExternalFunctionId, FFIType, FuncletId, FunctionClassId, NodeId, RemoteNodeId, StorageTypeId,
    TypeId,
};
use crate::assembly::table::Table;
use crate::ir;
use debug_ignore::DebugIgnore;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct Context {
    pub path: String,
    pub ffi_type_table: Table<FFIType>,
    pub local_type_table: Table<String>,
    // cause we need to know the storage value of the native value
    pub native_type_map: HashMap<String, FFIType>,
    pub variable_map: HashMap<FuncletId, NodeTable>,
    // where we currently are in the AST, using names
    // optional cause we may not have started traversal
    pub location: LocationNames,
    pub funclet_indices: FuncletIndices,
    pub function_classes: Table<FunctionClassId>,
}

#[derive(Debug)]
pub struct LocationNames {
    pub funclet_name: FuncletId,
    pub node_name: Option<NodeId>,
}

#[derive(Debug)]
pub struct NodeTable {
    // local names and return names such as [%out : i64] or whatever
    pub local: Table<NodeId>,
    pub returns: Table<NodeId>,
}

#[derive(Debug, Clone)]
pub enum LocalFFI {
    FFI(usize),
    Local(usize),
}

#[derive(Debug, Clone)]
pub enum NodeType {
    // Keeps track of internal names vs return names
    Local(usize),
    Return(usize),
}

impl LocalFFI {
    pub fn unpack(&self) -> usize {
        match self {
            LocalFFI::Local(u) => *u,
            LocalFFI::FFI(u) => *u,
            LocalFFI::FFI(u) => *u,
        }
    }
}

pub struct FuncletInformation {
    location: ir::Place,
    index: usize,
}

#[derive(Debug)]
pub struct FuncletIndices {
    external_funclet_table: Table<ExternalFunctionId>,
    local_funclet_table: Table<FuncletId>,
    funclet_kind_map: HashMap<String, ir::Place>,
}

impl LocationNames {
    pub fn new() -> LocationNames {
        LocationNames {
            funclet_name: FuncletId("".to_string()),
            node_name: None,
        }
    }
}

impl NodeTable {
    pub fn new() -> NodeTable {
        NodeTable {
            local: Table::new(),
            returns: Table::new(),
        }
    }
}

impl FuncletIndices {
    pub fn new() -> FuncletIndices {
        FuncletIndices {
            external_funclet_table: Table::new(),
            local_funclet_table: Table::new(),
            funclet_kind_map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: String, location: ir::Place) {
        match location {
            ir::Place::Local => self.local_funclet_table.push(FuncletId(name.clone())),
            ir::Place::Cpu => self
                .external_funclet_table
                .push(ExternalFunctionId(name.clone())),
            ir::Place::Gpu => self
                .external_funclet_table
                .push(ExternalFunctionId(name.clone())),
        }
        self.funclet_kind_map.insert(name, location);
    }

    pub fn get_index(&self, name: &FuncletId) -> Option<usize> {
        self.local_funclet_table.get(name)
    }

    pub fn get_loc(&self, name: &String) -> Option<&ir::Place> {
        self.funclet_kind_map.get(name)
    }

    pub fn get_funclet(&self, name: &String) -> Option<usize> {
        self.funclet_kind_map.get(name).and_then(|x| match x {
            ir::Place::Local => self.local_funclet_table.get(&FuncletId(name.clone())),
            ir::Place::Cpu => self
                .external_funclet_table
                .get(&ExternalFunctionId(name.clone())),
            ir::Place::Gpu => self
                .external_funclet_table
                .get(&ExternalFunctionId(name.clone())),
        })
    }

    pub fn require_funclet(&self, name: &String) -> usize {
        match self.get_funclet(name) {
            Some(f) => f,
            None => panic!("Unknown funclet name {}", name),
        }
    }
}

impl Context {
    pub fn new(program: &ast::Program) -> Context {
        let mut context = Context {
            path: "".to_string(),
            ffi_type_table: Table::new(),
            local_type_table: Table::new(),
            native_type_map: HashMap::new(),
            funclet_indices: FuncletIndices::new(),
            function_classes: Table::new(),
            variable_map: HashMap::new(),
            location: LocationNames::new(),
        };
        context.setup_context(program);
        context
    }

    // Take a pass over the program to construct the initial context
    // We only do this once and assume the invariants are maintained by construction
    // Note that a context without this makes little sense, so we can't build an "empty context"
    fn setup_context(&mut self, program: &ast::Program) {
        self.path = program.path.clone();
        for declaration in &program.declarations {
            match declaration {
                ast::Declaration::TypeDecl(typ) => match typ {
                    ast::TypeDecl::FFI(t) => self.ffi_type_table.push(t.clone()),
                    ast::TypeDecl::Local(t) => {
                        // get native value types
                        self.local_type_table.push(t.name.clone());
                        match &t.data {
                            ast::LocalTypeInfo::NativeValue { storage_type } => {
                                match storage_type {
                                    ast::TypeId::FFI(f) => {
                                        self.native_type_map.insert(t.name.clone(), f.clone());
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }
                },
                ast::Declaration::Funclet(f) => {
                    self.funclet_indices
                        .insert(f.header.name.0.clone(), ir::Place::Local);
                    let mut node_table = NodeTable::new();
                    for command in &f.commands {
                        // assume that each node has a name at this point,
                        //   re: explication corrections
                        match command.command {
                            ast::Command::Node(_) => {
                                node_table.local.push(command.name.clone().unwrap());
                            }
                            ast::Command::TailEdge(_) => {} // don't add tail edges
                            _ => {
                                unreachable!("unimplemented hole");
                            }
                        }
                    }
                    for ret_arg in &f.header.ret {
                        match &ret_arg.name {
                            None => {}
                            Some(name) => {
                                node_table.returns.push(name.clone());
                            }
                        }
                    }
                    self.variable_map.insert(f.header.name.clone(), node_table);
                }
                ast::Declaration::ExternalFunction(f) => {
                    let location = match f.kind {
                        ast::ExternalFunctionKind::CPUPure => ir::Place::Cpu,
                        ast::ExternalFunctionKind::CPUEffect => ir::Place::Cpu,
                        ast::ExternalFunctionKind::GPU(_) => ir::Place::Gpu,
                    };
                    self.funclet_indices.insert(f.name.clone(), location);
                }
                ast::Declaration::FunctionClass(f) => {
                    self.function_classes.push(f.name.clone());
                }
                _ => {}
            }
        }
    }

    pub fn ffi_type_id(&self, name: &ast::FFIType) -> usize {
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

    pub fn loc_type_id(&self, typ: &ast::TypeId) -> usize {
        match typ {
            ast::TypeId::FFI(ft) => self.ffi_type_id(&ft),
            ast::TypeId::Local(s) => self.local_type_id(&s),
        }
    }

    pub fn remote_node_id(&self, funclet: &FuncletId, var: &NodeId) -> usize {
        match self.variable_map.get(funclet) {
            Some(f) => match f.local.get(var) {
                Some(v) => v,
                None => panic!("Unknown local name {} in funclet {}", var, funclet),
            },
            None => panic!("Unknown funclet name {}", funclet),
        }
    }

    pub fn remote_return_id(&self, funclet: &FuncletId, var: &NodeId) -> usize {
        match self.variable_map.get(funclet) {
            Some(f) => match f.returns.get_index(var) {
                Some(v) => v,
                None => panic!("Unknown return name {} in funclet {}", var, funclet),
            },
            None => panic!("Unknown funclet name {}", funclet),
        }
    }

    pub fn node_from_id(&self, index: usize) -> NodeId {
        self.variable_map
            .get(&self.location.funclet_name)
            .unwrap()
            .local
            .get_at_index(index)
            .unwrap()
            .clone()
    }

    pub fn node_id(&self, var: &NodeId) -> usize {
        let funclet = &self.location.funclet_name;
        match self.variable_map.get(funclet).unwrap().local.get_index(var) {
            Some(v) => v,
            None => panic!("Unknown variable name {:?} in funclet {:?}", var, &funclet),
        }
    }

    pub fn return_id(&self, var: &NodeId) -> usize {
        let funclet = &self.location.funclet_name;
        match self
            .variable_map
            .get(funclet)
            .unwrap()
            .returns
            .get_index(var)
        {
            Some(v) => v,
            None => panic!("Unknown return name {:?} in funclet {:?}", var, &funclet),
        }
    }
}
