use crate::assembly::ast;
use crate::assembly::ast::Hole;
use crate::assembly::ast::{
    ExternalFunctionId, FFIType, FuncletId, FunctionClassId, MetaId, NodeId, RemoteNodeId,
    StorageTypeId, TypeId,
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
    pub variable_map: HashMap<FuncletId, HashMap<NodeId, ir::Quotient>>,
    pub node_id_map: HashMap<FuncletId, Table<NodeId>>,
    // for keeping track of the meanings of meta names for the current scheduling funclet
    // is None when we aren't in a scheduling funclet
    pub meta_map: Option<ast::MetaMapping>,
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

#[derive(Debug, Clone)]
pub enum LocalFFI {
    FFI(usize),
    Local(usize),
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

#[derive(Debug)]
pub struct OperationSet {
    pub value: Hole<ir::Quotient>,
    pub timeline: Hole<ir::Quotient>,
    pub spatial: Hole<ir::Quotient>,
}

impl LocationNames {
    pub fn new() -> LocationNames {
        LocationNames {
            funclet_name: FuncletId("".to_string()),
            node_name: None,
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
            node_id_map: HashMap::new(),
            meta_map: None,
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
                    let mut var_map = HashMap::new();
                    let mut id_table = Table::new();
                    // added because phi nodes themselves are unnamed
                    let mut rets = vec![];
                    for (index, ret_arg) in f.header.args.iter().enumerate() {
                        match &ret_arg.name {
                            None => {}
                            Some(name) => {
                                var_map.insert(name, ir::Quotient::Input { index });
                            }
                        };
                    }
                    for (node_id, command) in f.commands.iter().enumerate() {
                        match command {
                            Some(ast::Command::Node(ast::NamedNode { node, name })) => {
                                // a bit sketchy, but if we only correct this here, we should be ok
                                // basically we never rebuild the context
                                // and these names only matter for this context anyway
                                match name {
                                    None => {
                                        id_table.dummy_push();
                                    }
                                    Some(n) => {
                                        id_table.push(n);
                                        var_map.insert(n, ir::Quotient::Node { node_id });
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    for (index, ret_arg) in f.header.ret.iter().enumerate() {
                        match &ret_arg.name {
                            None => {}
                            Some(name) => {
                                var_map.insert(name, ir::Quotient::Output { index });
                            }
                        };
                    }
                    self.variable_map.insert(f.header.name.clone(), var_map);
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

    pub fn remote_node_id(&self, funclet: &FuncletId, node: &Option<NodeId>) -> ir::Quotient {
        match self.variable_map.get(funclet) {
            Some(f) => match node {
                None => ir::Quotient::None,
                Some(var) => match f.get(var) {
                    Some(v) => v.clone(),
                    None => {
                        panic!("Unknown node {} in funclet {}", var, funclet)
                    }
                },
            },
            None => panic!("Unknown funclet name {}", funclet),
        }
    }

    pub fn node_id(&self, var: &NodeId) -> usize {
        let funclet = &self.location.funclet_name;
        match self.node_id_map.get(funclet).unwrap().get_index(var) {
            Some(v) => v,
            None => panic!("Unknown variable name {:?} in funclet {:?}", var, &funclet),
        }
    }

    // gets the associated value, timeline, and spatial results from the given list of operations
    // note that this will return holes for any missing result
    pub fn operational_lookup(&self, operations: Vec<Hole<&RemoteNodeId>>) -> OperationSet {
        let mut result = OperationSet {
            value: None,
            timeline: None,
            spatial: None,
        };
        let error = "Holes in operational lists unsupported";
        for operation in operations {
            let unwrapped = operation.unwrap_or_else(|| panic!(error));
            let (fnid, kind) =
                self.meta_lookup_loc(&unwrapped.funclet.unwrap_or_else(|| panic!(error)));
            let quot = self.remote_node_id(
                &fnid,
                &unwrapped.node.as_ref().map(|n| n.unwrap_or_else(|| panic!(error))),
            );
            match kind {
                ir::FuncletKind::Value => match result.value {
                    None => { result.value = Some(quot) },
                    Some(old) => { 
                        panic!("Duplicate definitions using value: {:?} and {:?}", old, quot) 
                    }
                }
                ir::FuncletKind::Timeline => match result.timeline {
                    None => { result.timeline = Some(quot) },
                    Some(old) => { 
                        panic!("Duplicate definitions using timeline: {:?} and {:?}", old, quot) 
                    }
                } 
                ir::FuncletKind::Spatial => match result.spatial {
                    None => { result.spatial = Some(quot) },
                    Some(old) => { 
                        panic!("Duplicate definitions using spatial: {:?} and {:?}", old, quot) 
                    }
                }
                _ => {unreachable!()}
            }
        }
        result
    }

    fn meta_lookup_loc(&self, meta: &MetaId) -> (FuncletId, ir::FuncletKind) {
        let error = format!("{} doesn't have a meta map", &self.location.funclet_name);
        let mapping = self.meta_map.unwrap_or_else(|| panic!(error));
        if mapping.value.0 == *meta {
            (mapping.value.1, ir::FuncletKind::Value)
        } else if mapping.timeline.0 == *meta {
            (mapping.timeline.1, ir::FuncletKind::Timeline)
        } else if mapping.spatial.0 == *meta {
            (mapping.spatial.1, ir::FuncletKind::Spatial)
        } else {
            panic!("Invalid meta name {}", meta)
        }
    }

    pub fn meta_lookup(&self, meta: &MetaId) -> FuncletId {
        self.meta_lookup_loc(meta).0
    }

    pub fn set_meta_map(&self, meta: ast::MetaMapping) {
        self.meta_map = Some(meta)
    }

    pub fn reset_meta_map(&self) {
        self.meta_map = None
    }
}
