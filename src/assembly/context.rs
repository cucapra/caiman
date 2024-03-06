use crate::assembly::ast;
use crate::assembly::ast::Hole;
use crate::assembly::ast::{
    EffectId, ExternalFunctionId, FFIType, FuncletId, FunctionClassId, MetaId, NodeId,
    RemoteNodeId, StorageTypeId, TypeId,
};
use crate::assembly::table::Table;
use crate::ir;
use crate::rust_wgpu_backend::ffi;
use debug_ignore::DebugIgnore;
use std::collections::{HashMap, HashSet};

// Utility stuff
pub fn reject_hole<T>(h: Hole<T>) -> T {
    match h {
        Hole::Filled(v) => v,
        Hole::Empty => unreachable!("Unimplemented Hole"),
    }
}

#[derive(Debug)]
pub struct Context {
    pub path: String,
    pub ffi_type_table: Table<FFIType>,
    pub local_type_table: Table<String>,
    // cause we need to know the storage value of the native value
    pub native_type_map: HashMap<String, FFIType>,
    pub variable_map: HashMap<FuncletId, HashMap<NodeId, ir::Quotient>>,
    // for keeping track of the meanings of meta names for the current scheduling funclet
    // is None when we aren't in a scheduling funclet
    pub meta_map: Option<ast::MetaMapping>,
    // where we currently are in the AST, using names
    // optional cause we may not have started traversal
    pub location: LocationNames,
    pub funclet_indices: FuncletIndices,
    pub function_classes: Table<FunctionClassId>,
    pub effects: Table<EffectId>,
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

#[derive(Debug, Clone)]
pub struct OperationSet {
    pub value: Hole<ir::Quotient>,
    pub timeline: Hole<ir::Quotient>,
    pub spatial: Hole<ir::Quotient>,
}

#[derive(Debug, Clone)]
pub struct TagSet {
    pub value: Hole<ir::Tag>,
    pub timeline: Hole<ir::Tag>,
    pub spatial: Hole<ir::Tag>,
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

    pub fn get_funclet(&self, name: &String) -> Hole<usize> {
        self.funclet_kind_map
            .get(name)
            .and_then(|x| match x {
                ir::Place::Local => self.local_funclet_table.get(&FuncletId(name.clone())),
                ir::Place::Cpu => self
                    .external_funclet_table
                    .get(&ExternalFunctionId(name.clone())),
                ir::Place::Gpu => self
                    .external_funclet_table
                    .get(&ExternalFunctionId(name.clone())),
            })
            .into()
    }

    pub fn require_funclet(&self, name: &String) -> usize {
        match self.get_funclet(name) {
            Hole::Filled(f) => f,
            Hole::Empty => panic!("Unknown funclet name {}", name),
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
            effects: Table::new(),
            variable_map: HashMap::new(),
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
                    for (index, arg) in f.header.args.iter().enumerate() {
                        match &arg.name {
                            None => {}
                            Some(name) => {
                                var_map.insert(name.clone(), ir::Quotient::Input { index });
                            }
                        };
                    }
                    let mut node_id = 0; // used for skipping tail edges
                    for command in f.commands.iter() {
                        match command {
                            Hole::Filled(ast::Command::Node(ast::NamedNode { node, name })) => {
                                // a bit sketchy, but if we only correct this here, we should be ok
                                // basically we never rebuild the context
                                // and these names only matter for this context anyway
                                match name {
                                    None => {}
                                    Some(n) => {
                                        if (n.0 != "_") {
                                            var_map
                                                .insert(n.clone(), ir::Quotient::Node { node_id });
                                        }
                                    }
                                }

                                node_id += 1; // if it's a "real node" increment the id
                            }
                            _ => {}
                        }
                    }
                    for (index, ret_arg) in f.header.ret.iter().enumerate() {
                        match &ret_arg.name {
                            None => {}
                            Some(name) => {
                                var_map.insert(name.clone(), ir::Quotient::Output { index });
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
                ast::Declaration::Effect(f) => {
                    self.effects.push(f.name.clone());
                }
                _ => {}
            }
        }
    }

    pub fn external_lookup(&self, id: &ExternalFunctionId) -> ir::ExternalFunctionId {
        ffi::ExternalFunctionId(
            self.funclet_indices
                .external_funclet_table
                .get(id)
                .expect(format!("Unknown external funclet {:?}", id).as_str()),
        )
    }

    pub fn effect_lookup(&self, effect: &EffectId) -> ffi::EffectId {
        ffi::EffectId(
            self.effects
                .get(effect)
                .expect(format!("Unknown effect {:?}", effect).as_str()),
        )
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

    pub fn explicit_node_id(&self, funclet: &FuncletId, node: &Option<NodeId>) -> ir::Quotient {
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

    pub fn remote_node_id(&self, remote: &RemoteNodeId) -> ir::Quotient {
        self.explicit_node_id(
            &self.meta_lookup(reject_hole(remote.funclet.as_ref())),
            &remote.node.as_ref().cloned().map(|n| reject_hole(n)),
        )
    }

    pub fn node_id(&self, var: &NodeId) -> usize {
        let funclet = &self.location.funclet_name;
        let var_error = format!("Unknown variable name {:?} in funclet {:?}", var, &funclet);
        match self
            .variable_map
            .get(funclet)
            .unwrap()
            .get(var)
            .expect(&var_error)
        {
            ir::Quotient::None => panic!("Invalid None node {:?} in funclet {:?}", var, &funclet),
            ir::Quotient::Input { index }
            | ir::Quotient::Output { index }
            | ir::Quotient::Node { node_id: index } => *index,
        }
    }

    // gets the associated value, timeline, and spatial results from the given list of tags
    // note that this will return holes for any missing result
    pub fn tag_lookup(&self, operations: &Vec<Hole<ast::Tag>>) -> TagSet {
        let mut result = TagSet {
            value: Hole::Empty,
            timeline: Hole::Empty,
            spatial: Hole::Empty,
        };
        let error = "Holes in operational lists unsupported";
        for operation in operations {
            let unwrapped = operation.as_ref().unwrap_or_else(|| panic!(error));
            let remote = unwrapped.quot.as_ref().unwrap_or_else(|| panic!(error));
            let (fnid, kind) =
                self.meta_lookup_loc(remote.funclet.as_ref().unwrap_or_else(|| panic!(error)));
            let quot = self.explicit_node_id(
                &fnid,
                &remote
                    .node
                    .as_ref()
                    .cloned()
                    .map(|n| n.unwrap_or_else(|| panic!(error))),
            );
            let tag = Hole::Filled(ir::Tag {
                quot,
                flow: unwrapped.flow.clone(),
            });
            match kind {
                ir::FuncletKind::Value => match result.value {
                    Hole::Empty => {
                        result.value = tag;
                    }
                    Hole::Filled(old) => {
                        panic!(
                            "Duplicate definitions using value: {:?} and {:?}",
                            old, quot
                        )
                    }
                },
                ir::FuncletKind::Timeline => match result.timeline {
                    Hole::Empty => {
                        result.timeline = tag;
                    }
                    Hole::Filled(old) => {
                        panic!(
                            "Duplicate definitions using timeline: {:?} and {:?}",
                            old, quot
                        )
                    }
                },
                ir::FuncletKind::Spatial => match result.spatial {
                    Hole::Empty => {
                        result.spatial = tag;
                    }
                    Hole::Filled(old) => {
                        panic!(
                            "Duplicate definitions using spatial: {:?} and {:?}",
                            old, quot
                        )
                    }
                },
                _ => {
                    unreachable!()
                }
            }
        }
        result
    }

    // extremely stupid, but it works
    pub fn operational_lookup(&self, operations: &Vec<Hole<ast::RemoteNodeId>>) -> OperationSet {
        let tags = operations.iter().map(|quot| {
            Hole::Filled(ast::Tag {
                quot: quot.clone(),
                // the dumb part, this doesn't matter
                flow: ir::Flow::Dead,
            })
        });
        let result = self.tag_lookup(&tags.collect());
        OperationSet {
            value: result.value.map(|t| t.quot),
            timeline: result.timeline.map(|t| t.quot),
            spatial: result.spatial.map(|t| t.quot),
        }
    }

    fn meta_lookup_loc(&self, meta: &MetaId) -> (FuncletId, ir::FuncletKind) {
        let error = format!("{} doesn't have a meta map", &self.location.funclet_name);
        let mapping = self.meta_map.as_ref().unwrap_or_else(|| panic!(error));
        if mapping.value.0 == *meta {
            (mapping.value.1.clone(), ir::FuncletKind::Value)
        } else if mapping.timeline.0 == *meta {
            (mapping.timeline.1.clone(), ir::FuncletKind::Timeline)
        } else if mapping.spatial.0 == *meta {
            (mapping.spatial.1.clone(), ir::FuncletKind::Spatial)
        } else {
            panic!("Invalid meta name {}", meta)
        }
    }

    pub fn meta_lookup(&self, meta: &MetaId) -> FuncletId {
        self.meta_lookup_loc(meta).0
    }

    pub fn set_meta_map(&mut self, meta_map: ast::MetaMapping) {
        self.meta_map = Some(meta_map)
    }

    pub fn reset_meta_map(&mut self) {
        self.meta_map = None
    }
}
