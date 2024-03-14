use self::expir::FuncletKind;

use super::*;
use crate::ir;

// These are all getters designed to work with "original" program, before mutations touch things
// Specifically we want things like lists of funclet names up-front or node names up-front

impl StaticContext {
    pub fn new(program: expir::Program) -> StaticContext {
        let mut context = StaticContext {
            program,
            type_declarations: HashMap::new(),
            spec_explication_data: HashMap::new(),
        };
        context.initialize_declarations();
        context
    }

    fn initialize_declarations(&mut self) {
        let mut type_decls = HashMap::new();

        self.type_declarations = type_decls;
        for (index, funclet) in self.program.funclets.iter() {
            match &funclet.kind {
                FuncletKind::Value | FuncletKind:: Unknown => {},
                _ => {
                    self.initialize_spec_funclet_info(index, funclet);
                }
            }
        }
    }

    fn initialize_spec_funclet_info(&mut self, index: usize, funclet: &expir::Funclet) {
        let mut node_dependencies = HashMap::new();
        let mut tail_dependencies = Vec::new();
        for (index, node) in funclet.nodes.as_ref().iter().enumerate() {
            match &node {
                None => {}
                Some(node) => {
                    node_dependencies
                        .insert(index, identify_node_deps(node));
                }
            }
        }
        match &funclet.tail_edge {
            None => {}
            Some(t) => {
                identify_tailedge_deps(t);
            }
        }

        // second loop for type information
        let mut deduced_types = HashMap::new();
        match &funclet.kind {
            ir::FuncletKind::Value => {
                for node in funclet.nodes.as_ref() {
                    match node {
                        None => {}
                        // TODO: ???
                        Some(n) => {}
                    }
                }
            }
            _ => {}
        }

        self.spec_explication_data.insert(
            index,
            SpecFuncletData {
                node_dependencies,
                tail_dependencies,
                deduced_types,
                connections: vec![],
            },
        );
    }

    // for quick and dirty things
    pub fn program(&self) -> &expir::Program {
        &self.program
    }

    pub fn schedule_funclet_ids(&self) -> Vec<FuncletId> {
        let mut result = Vec::new();
        for declaration in &self.program.declarations {
            match declaration {
                expir::Declaration::Funclet(f) => match f.kind {
                    ir::FuncletKind::ScheduleExplicit => {
                        result.push(f.header.name.clone());
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        result
    }

    pub fn command_ids(&self, funclet: &expir::FuncletId) -> Vec<NodeId> {
        let mut result = Vec::new();
        for command in &self.get_funclet(funclet).commands {
            match &command.command {
                expir::Command::Node(_) => {
                    result.push(command.name.as_ref().unwrap().clone());
                }
                expir::Command::Hole => {
                    result.push(command.name.as_ref().unwrap().clone());
                }
                expir::Command::TailEdge(_) => {}
                expir::Command::ExplicationHole => unreachable!(),
            }
        }
        result
    }

    pub fn get_spec_funclet(&self, funclet: &FuncletId, spec: &SpecLanguage) -> &FuncletId {
        self.get_schedule_info(funclet).specs.get(spec)
    }

    // get the specification type of the value node (if known)
    pub fn get_spec_instantiation(
        &self,
        funclet: &FuncletId,
        node: &NodeId,
        spec: &SpecLanguage,
    ) -> Option<&NodeId> {
        self.get_schedule_info(funclet)
            .type_instantiations
            .get(node)
            .and_then(|inst| inst.get(spec))
    }

    pub fn get_value_funclet(&self, schedule: &FuncletId) -> Option<&FuncletId> {
        self.schedule_explication_data
            .get(schedule)
            .map(|f| &f.specs.value)
    }

    fn get_scoped<'a, T, U, V>(&'a self, info: T, map: U) -> Option<&V>
    where
        T: std::hash::Hash + PartialEq + Eq + 'a,
        U: Fn(&ScheduleScopeData) -> &HashMap<T, V>,
    {
        // takes advantage of the invariant that vectors of a key remove that key when emptied
        for scope in self.scopes.iter().rev() {
            match map(scope).get(&info) {
                None => {}
                Some(result) => {
                    return Some(result);
                }
            }
        }
        None
    }

    fn get_type_decl(&self, typ: &expir::TypeId) -> Option<&LocalTypeDeclaration> {
        match typ {
            TypeId::FFI(_) => None,
            TypeId::Local(type_name) => Some(
                self.type_declarations
                    .get(type_name)
                    .unwrap_or_else(|| panic!("Unknown type_name {:?}", type_name)),
            ),
        }
    }

    pub fn get_matching_operation(
        &self,
        funclet: &FuncletId,
        returns: Vec<Hole<&NodeId>>,
    ) -> Option<&NodeId> {
        let mut result = None;
        let mut index_map = HashMap::new();
        for (index, ret) in returns.into_iter().enumerate() {
            match ret {
                None => {}
                Some(name) => {
                    index_map.insert(name.clone(), index);
                }
            }
        }
        let spec_data = self.get_spec_data(funclet);
        for command in &self.get_funclet(funclet).commands {
            let name = command.name.as_ref().unwrap();
            if index_map.contains_key(name) {
                match &command.command {
                    expir::Command::Node(expir::Node::ExtractResult { node_id, index }) => {
                        // make sure that the index matches the given argument
                        assert_eq!(index.as_ref().unwrap(), index_map.get(name).unwrap());
                        // lots of potentially panicking unwraps here
                        // none of these should be `None` at this point
                        let dependency = spec_data
                            .node_dependencies
                            .get(node_id.as_ref().unwrap())
                            .unwrap()
                            .first()
                            .unwrap();
                        // every extraction of one function should match to that function
                        result = assign_or_compare(result, dependency);
                    }
                    _ => panic!("Attempted to treat {:?} as an extract operation", command),
                }
            }
        }
        result
    }

    pub fn get_type_place(&self, typ: &expir::TypeId) -> Option<&ir::Place> {
        self.get_type_decl(typ).and_then(|t| (&t.place).as_ref())
    }

    pub fn get_type_ffi(&self, typ: &expir::TypeId) -> Option<&expir::FFIType> {
        self.get_type_decl(typ).and_then(|t| (&t.ffi).as_ref())
    }

    pub fn get_latest_explication_hole(&self) -> Option<&FuncletId> {
        for scope in self.scopes.iter().rev() {
            if scope.explication_hole {
                return Some(scope.funclet_name);
            }
        }
        None
    }

    fn get_spec_data(&self, funclet: &FuncletId) -> &SpecFuncletData {
        match self.spec_explication_data.get(funclet) {
            Some(spec) => spec,
            None => panic!("Unknown specification function {:?}", funclet),
        }
    }

    fn get_schedule_info(&self, funclet: &FuncletId) -> &ScheduleFuncletData {
        match self.schedule_explication_data.get(funclet) {
            Some(data) => data,
            None => panic!("Unknown schedule funclet {:?}", funclet),
        }
    }

    pub fn get_funclet(&self, funclet: &FuncletId) -> &expir::Funclet {
        for declaration in &self.program.declarations {
            match declaration {
                expir::Declaration::Funclet(f) => {
                    if &f.header.name == funclet {
                        return f;
                    }
                }
                _ => {}
            }
        }
        panic!("Unknown funclet {:?}", funclet);
    }

    pub fn get_command(&self, funclet: &FuncletId, name: &NodeId) -> &expir::Command {
        for command in &self.get_funclet(funclet).commands {
            match &command.name {
                None => {}
                Some(n) => {
                    if n == name {
                        {
                            return &command.command;
                        }
                    }
                }
            }
        }
        panic!("Unknown command {:?} in funclet {:?}", name, funclet);
    }

    pub fn get_node(&self, funclet: &FuncletId, name: &NodeId) -> &expir::Node {
        match self.get_command(funclet, name) {
            expir::Command::Node(n) => n,
            _ => panic!(
                "Attempted to treat command {:?} in funclet {:?} as a node",
                name, funclet
            ),
        }
    }

    pub fn get_tail_edge(&self, funclet_name: &FuncletId) -> Option<&expir::TailEdge> {
        for command in &self.get_funclet(funclet_name).commands {
            match &command.name {
                None => {}
                Some(n) => match &command.command {
                    expir::Command::TailEdge(edge) => {
                        return Some(edge);
                    }
                    _ => {}
                },
            }
        }
        None
    }
}

fn identify_node_deps(node: &expir::Node) -> Vec<NodeId> {
    let dependencies = match node {
        expir::Node::Phi { index } => {
            vec![]
        }
        expir::Node::ExtractResult { node_id, index } => {
            vec![node_id]
        }
        expir::Node::Constant { value, type_id } => vec![],
        expir::Node::CallFunctionClass {
            function_id,
            arguments,
        } => reject_hole(arguments.as_ref())
            .iter()
            .map(|n| reject_hole_clone(n))
            .collect(),
        expir::Node::Select {
            condition,
            true_case,
            false_case,
        } => {
            vec![
                reject_hole(condition),
                reject_hole(true_case),
                reject_hole(false_case),
            ]
        }
        expir::Node::EncodingEvent {
            local_past,
            remote_local_pasts,
        } => vec![reject_hole_clone(local_past)]
            .into_iter()
            .chain(
                reject_hole(remote_local_pasts.as_ref())
                    .iter()
                    .map(|n| reject_hole_clone(n)),
            )
            .collect(),
        expir::Node::SubmissionEvent { local_past } => {
            vec![reject_hole_clone(local_past)]
        }
        expir::Node::SynchronizationEvent {
            local_past,
            remote_local_past,
        } => {
            vec![
                reject_hole_clone(local_past),
                reject_hole_clone(remote_local_past),
            ]
        }
        expir::Node::SeparatedBufferSpaces { count, space } => {
            vec![reject_hole_clone(space)]
        }
        _ => {
            unreachable!("Unsupported named specification node type {:?}", &node);
        }
    };
    dependencies
}

// helper methods for reading information
fn identify_tailedge_deps(edge: &expir::TailEdge) -> Vec<NodeId> {
    let dependencies = match edge {
        expir::TailEdge::DebugHole { inputs } => inputs.iter().map(|n| n.clone()).collect(),
        expir::TailEdge::Return { return_values } => reject_hole(return_values.as_ref())
            .iter()
            .map(|n| reject_hole_clone(n))
            .collect(),
        expir::TailEdge::Jump { join, arguments } => vec![reject_hole_clone(join)]
            .into_iter()
            .chain(
                reject_hole(arguments.as_ref())
                    .iter()
                    .map(|n| reject_hole_clone(n)),
            )
            .collect(),
        _ => {
            unreachable!("Unsupported specification tailedge type {:?}", &edge);
        }
    };
    dependencies
}

fn get_spec_node<'a>(funclet: &'a expir::Funclet, node: &NodeId) -> &'a expir::Node {
    for command in &funclet.commands {
        if command.name.as_ref().unwrap() == node {
            match &command.command {
                expir::Command::Node(node) => {
                    return node;
                }
                _ => unreachable!("Attempting to treat spec element {:?} as a node", command),
            }
        }
    }
    unreachable!("Unknown spec node dependency {:?}", node)
}

fn get_type_info<'a>(
    funclet: &'a expir::Funclet,
    nodeid: &NodeId,
    node_dependencies: &HashMap<NodeId, Vec<NodeId>>,
    type_map: &mut HashMap<NodeId, Vec<TypeId>>,
) -> Vec<&'a expir::TypeId> {
    if let Some(result) = type_map.get(nodeid) {
        result
    } else {
        let result = deduce_type(funclet, nodeid, node_dependencies, type_map);
        type_map.insert(nodeid.clone(), result.iter().cloned().collect());
        result
    }
}

fn deduce_type<'a>(
    funclet: &'a expir::Funclet,
    check_id: &NodeId,
    node_dependencies: &HashMap<NodeId, Vec<NodeId>>,
    type_map: &mut HashMap<NodeId, Vec<TypeId>>,
) -> Vec<&'a TypeId> {
    let names = node_dependencies.get(&check_id).unwrap_or_else(|| {
        unreachable!(unreachable!("Unknown spec node dependency {:?}", check_id))
    });
    let node = get_spec_node(funclet, &check_id);
    let typ = match node {
        expir::Node::Phi { index } => vec![
            &funclet
                .header
                .args
                .get(index.unwrap())
                .as_ref()
                .unwrap()
                .typ,
        ],
        expir::Node::ExtractResult { node_id, index } => {
            let index = index.unwrap();
            vec![get_type_info(
                funclet,
                node_id.as_ref().unwrap(),
                node_dependencies,
                type_map,
            )
            .get(index)
            .unwrap_or_else(panic!(
                "Not enough arguments to extract index {} from {:?}",
                index, node_id
            ))]
        }
        expir::Node::Constant { value, type_id } => {}
        expir::Node::CallFunctionClass {
            function_id,
            arguments,
        } => {}
        expir::Node::Select {
            condition,
            true_case,
            false_case,
        } => {}
        _ => unreachable!("Not a value node {:?}", node),
    };
    typ
}
