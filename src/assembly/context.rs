use crate::assembly::ast;
use crate::assembly::ast::{
    EffectId, ExternalFunctionId, FFIType, FuncletId, FunctionClassId, MetaId, NodeId,
    RemoteNodeId, StorageTypeId, TypeId,
};
use crate::assembly::table::Table;
use crate::debug_info::{DebugInfo, FuncletDebugMap};
use crate::explication::expir;
use crate::explication::Hole;
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

pub fn reject_opt<T>(h: Option<T>) -> T {
    match h {
        Some(v) => v,
        None => unreachable!("Unimplemented Hole"),
    }
}

#[derive(Debug)]
pub struct Context {
    pub path: String,
    pub ffi_type_table: Table<FFIType>,
    pub local_type_table: Table<String>,
    // cause we need to know the storage value of the native value
    pub native_type_map: HashMap<String, FFIType>,
    pub ffi_to_native_map: HashMap<FFIType, String>,
    pub variable_map: HashMap<FuncletId, HashMap<NodeId, expir::Quotient>>,
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
    location: expir::Place,
    index: usize,
}

#[derive(Debug)]
pub struct FuncletIndices {
    external_funclet_table: Table<ExternalFunctionId>,
    local_funclet_table: Table<FuncletId>,
    funclet_kind_map: HashMap<String, expir::Place>,
}

#[derive(Debug, Clone)]
pub struct OperationSet {
    pub value: Hole<expir::Quotient>,
    pub timeline: Hole<expir::Quotient>,
    pub spatial: Hole<expir::Quotient>,
}

#[derive(Debug, Clone)]
pub struct TagSet {
    pub value: Hole<expir::Tag>,
    pub timeline: Hole<expir::Tag>,
    pub spatial: Hole<expir::Tag>,
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

    pub fn insert(&mut self, name: String, location: expir::Place) {
        match location {
            expir::Place::Local => self.local_funclet_table.push(FuncletId(name.clone())),
            expir::Place::Cpu => self
                .external_funclet_table
                .push(ExternalFunctionId(name.clone())),
            expir::Place::Gpu => self
                .external_funclet_table
                .push(ExternalFunctionId(name.clone())),
        }
        self.funclet_kind_map.insert(name, location);
    }

    pub fn get_index(&self, name: &FuncletId) -> Option<usize> {
        self.local_funclet_table.get(name)
    }

    pub fn get_loc(&self, name: &String) -> Option<&expir::Place> {
        self.funclet_kind_map.get(name)
    }

    pub fn get_funclet(&self, name: &String) -> Option<usize> {
        self.funclet_kind_map.get(name).and_then(|x| match x {
            expir::Place::Local => self.local_funclet_table.get(&FuncletId(name.clone())),
            expir::Place::Cpu => self
                .external_funclet_table
                .get(&ExternalFunctionId(name.clone())),
            expir::Place::Gpu => self
                .external_funclet_table
                .get(&ExternalFunctionId(name.clone())),
        })
    }

    pub fn require_funclet(&self, name: &String) -> usize {
        self.get_funclet(name)
            .expect(&format!("Unknown funclet name {}", name))
    }
}

impl Context {
    pub fn new(program: &ast::Program) -> Context {
        let mut context = Context {
            path: "".to_string(),
            ffi_type_table: Table::new(),
            local_type_table: Table::new(),
            native_type_map: HashMap::new(),
            ffi_to_native_map: HashMap::new(),
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
                                        let check = self
                                            .ffi_to_native_map
                                            .insert(f.clone(), t.name.clone());
                                        match check {
                                            None => {}
                                            Some(old) => {
                                                panic!(
                                                    "Duplicate native values for FFI type {:?}, {} and {}",
                                                    f.clone(),
                                                    old,
                                                    &t.name
                                                );
                                            }
                                        }
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
                        .insert(f.header.name.0.clone(), expir::Place::Local);
                    let mut var_map = HashMap::new();
                    for (index, arg) in f.header.args.iter().enumerate() {
                        match &arg.name {
                            None => {}
                            Some(name) => {
                                var_map.insert(name.clone(), expir::Quotient::Input { index });
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
                                        var_map
                                            .insert(n.clone(), expir::Quotient::Node { node_id });
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
                                var_map.insert(name.clone(), expir::Quotient::Output { index });
                            }
                        };
                    }
                    self.variable_map.insert(f.header.name.clone(), var_map);
                }
                ast::Declaration::ExternalFunction(f) => {
                    let location = match f.kind {
                        ast::ExternalFunctionKind::CPUPure => expir::Place::Cpu,
                        ast::ExternalFunctionKind::CPUEffect => expir::Place::Cpu,
                        ast::ExternalFunctionKind::GPU(_) => expir::Place::Gpu,
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

    pub fn external_lookup(&self, id: &ExternalFunctionId) -> expir::ExternalFunctionId {
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

    pub fn ffi_type_id(&self, name: &ast::FFIType) -> crate::rust_wgpu_backend::ffi::TypeId {
        match self.ffi_type_table.get_index(name) {
            Some(i) => crate::rust_wgpu_backend::ffi::TypeId(i),
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
            ast::TypeId::FFI(ft) => self.local_type_id(
                &self
                    .ffi_to_native_map
                    .get(&ft)
                    .expect(&format!("No storage type found for ffi type {:?}", ft)),
            ),
            ast::TypeId::Local(s) => self.local_type_id(&s),
        }
    }

    pub fn explicit_node_id(&self, funclet: &FuncletId, node: &Hole<NodeId>) -> expir::Quotient {
        match self.variable_map.get(funclet) {
            Some(f) => match node {
                Hole::Empty => expir::Quotient::None,
                Hole::Filled(var) => f
                    .get(var)
                    .expect(format!("Unknown node {} in funclet {}", var, funclet).as_str())
                    .clone(),
            },
            None => panic!("Unknown funclet name {}", funclet),
        }
    }

    pub fn function_class_id(&self, f: &FunctionClassId) -> expir::FunctionClassId {
        self.function_classes
            .get(f)
            .expect(format!("Unknown function class {:?}", f).as_str())
    }

    // TODO: Note that the funclet name gets thrown out here, is this a problem?
    pub fn remote_id(&self, f: &RemoteNodeId) -> expir::Quotient {
        match &f.node {
            None => expir::Quotient::None,
            Some(node) => {
                let funclet_id = self.meta_lookup(&f.funclet);
                self.explicit_node_id(&funclet_id, node)
            }
        }
    }

    pub fn funclet_id(&self, f: &FuncletId) -> expir::FuncletId {
        self.funclet_indices
            .get_funclet(&f.0)
            .expect(format!("Unknown funclet {:?}", f).as_str())
    }

    pub fn external_funclet_id(&self, f: &ExternalFunctionId) -> expir::ExternalFunctionId {
        expir::ExternalFunctionId {
            0: self
                .funclet_indices
                .get_funclet(&f.0)
                .expect(format!("Unknown funclet {:?}", f).as_str()),
        }
    }

    pub fn node_id(&self, var: &NodeId) -> expir::NodeId {
        let funclet = &self.location.funclet_name;
        let var_error = format!("Unknown variable name {:?} in funclet {:?}", var, &funclet);
        match self
            .variable_map
            .get(funclet)
            .unwrap()
            .get(var)
            .expect(&var_error)
        {
            expir::Quotient::None => {
                panic!("Invalid None node {:?} in funclet {:?}", var, &funclet)
            }
            expir::Quotient::Input { index }
            | expir::Quotient::Output { index }
            | expir::Quotient::Node { node_id: index } => *index,
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
            let unwrapped = operation.as_ref().opt().expect(&error);
            let remote = unwrapped.quot.as_ref().opt().expect(&error);
            let (fnid, kind) = self.meta_lookup_loc(&remote.funclet);
            let quot = self.explicit_node_id(
                &fnid,
                &remote
                    .node
                    .as_ref()
                    .cloned()
                    .map(|n| n.opt().expect(&error))
                    .into(),
            );
            let tag = Hole::Filled(expir::Tag {
                quot,
                flow: unwrapped.flow.clone(),
            });
            match kind {
                expir::FuncletKind::Value => match result.value {
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
                expir::FuncletKind::Timeline => match result.timeline {
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
                expir::FuncletKind::Spatial => match result.spatial {
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
    pub fn operational_lookup(
        &self,
        operations: &Hole<Vec<Hole<ast::RemoteNodeId>>>,
    ) -> OperationSet {
        match operations.as_ref() {
            Hole::Empty => OperationSet {
                value: Hole::Empty,
                timeline: Hole::Empty,
                spatial: Hole::Empty,
            },
            Hole::Filled(ops) => {
                let tags = ops.iter().map(|quot| {
                    Hole::Filled(ast::Tag {
                        quot: quot.clone(),
                        // the dumb part, this doesn't matter
                        flow: Hole::Empty,
                    })
                });
                let result = self.tag_lookup(&tags.collect());
                OperationSet {
                    value: result.value.opt().map(|t| t.quot).into(),
                    timeline: result.timeline.opt().map(|t| t.quot).into(),
                    spatial: result.spatial.opt().map(|t| t.quot).into(),
                }
            }
        }
    }

    fn meta_lookup_loc(&self, meta: &MetaId) -> (FuncletId, expir::FuncletKind) {
        let error = format!("{} doesn't have a meta map", &self.location.funclet_name);
        let mapping = self.meta_map.as_ref().unwrap_or_else(|| panic!(error));
        if mapping.value.0 == *meta {
            (mapping.value.1.clone(), expir::FuncletKind::Value)
        } else if mapping.timeline.0 == *meta {
            (mapping.timeline.1.clone(), expir::FuncletKind::Timeline)
        } else if mapping.spatial.0 == *meta {
            (mapping.spatial.1.clone(), expir::FuncletKind::Spatial)
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

    pub fn drain_into_debug_info(mut self) -> DebugInfo {
        match self {
            Context {
                path,
                ffi_type_table,
                local_type_table,
                native_type_map,
                ffi_to_native_map,
                mut variable_map,
                meta_map,
                location,
                funclet_indices,
                function_classes,
                effects,
            } => {
                let type_map = local_type_table
                    .drain("_UNNAMED_TYPE_".to_string())
                    .into_iter()
                    .enumerate()
                    .collect();
                let ffi_type_map = ffi_type_table
                    .drain(FFIType::Unknown)
                    .into_iter()
                    .enumerate()
                    .collect();
                let function_class_map = function_classes
                    .drain(FunctionClassId("_UNNAMED_CLASS_".to_string()))
                    .into_iter()
                    .map(|s| s.0)
                    .enumerate()
                    .collect();
                let external_function_map = funclet_indices
                    .external_funclet_table
                    .drain(ExternalFunctionId("_UNNAMED_EXTERNAL_".to_string()))
                    .into_iter()
                    .map(|s| s.0)
                    .enumerate()
                    .collect();
                let mut funclet_map = HashMap::new();
                for (index, funclet) in funclet_indices
                    .local_funclet_table
                    .drain(FuncletId("_UNNAMED_FUNCLET_".to_string()))
                    .into_iter()
                    .enumerate()
                {
                    let mut node_map = HashMap::new();
                    for (node, quot) in variable_map.get_mut(&funclet).unwrap().drain() {
                        node_map.insert(quot, node.0);
                    }
                    funclet_map.insert(
                        index,
                        FuncletDebugMap {
                            name: funclet.0,
                            node_map,
                        },
                    );
                }
                DebugInfo {
                    type_map,
                    ffi_type_map,
                    function_class_map,
                    external_function_map,
                    funclet_map,
                }
            }
        }
    }
}
