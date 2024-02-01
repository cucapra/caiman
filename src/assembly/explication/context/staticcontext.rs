use super::*;
use crate::ir;

// These are all getters designed to work with "original" program, before mutations touch things
// Specifically we want things like lists of funclet names up-front or node names up-front

impl StaticContext {
    pub fn new(program: ast::Program) -> Context {
        let mut context = Context { 
            program: DebugIgnore(program),
            type_declarations: HashMap::new(),
            spec_explication_data: HashMap::new(),
        }
        context.initialize_declarations();
    }

    fn initialize_declarations(&mut self) {
        let mut type_decls = HashMap::new();
        let mut spec_funclets = Vec::new();
        let mut schedule_funclets = Vec::new();
        for declaration in &self.program.declarations {
            match declaration {
                ast::Declaration::TypeDecl(ast::TypeDecl::Local(local)) => {
                    type_decls.insert(local.name.clone(), LocalTypeDeclaration::new(&local.data));
                }
                ast::Declaration::Funclet(funclet) => {
                    let name = funclet.header.name.clone();
                    match &funclet.kind {
                        ir::FuncletKind::Value => {
                            spec_funclets.push(name);
                        }
                        ir::FuncletKind::Timeline => {
                            spec_funclets.push(name);
                        }
                        ir::FuncletKind::Spatial => {
                            spec_funclets.push(name);
                        }
                        ir::FuncletKind::ScheduleExplicit => {
                            schedule_funclets.push(name);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        self.type_declarations = type_decls;
        for spec_funclet in spec_funclets {
            self.initialize_spec_funclet_info(spec_funclet);
        }
        for schedule_funclet in schedule_funclets {
            self.initialize_schedule_funclet_info(schedule_funclet);
        }
    }

    fn initialize_spec_funclet_info(&mut self, funclet_name: ast::FuncletId) {
        let funclet = self.get_funclet_mut(&funclet_name);
        let mut node_dependencies = HashMap::new();
        let mut tail_dependencies = Vec::new();
        for command in &funclet.commands {
            match &command.command {
                ast::Command::Hole => {}
                ast::Command::Node(node) => {
                    node_dependencies
                        .insert(reject_hole_clone(&command.name), identify_node_deps(node));
                }
                ast::Command::TailEdge(edge) => {
                    tail_dependencies = identify_tailedge_deps(edge);
                }
                ast::Command::ExplicationHole => {
                    panic!(
                        "Encountered an explication hole when initializing the context in {:?}",
                        funclet_name
                    );
                }
            }
        }

        // second loop for type information
        let mut deduced_types = HashMap::new();
        match &funclet.kind {
            ir::FuncletKind::Value => {
                for command in &funclet.commands {
                    match &command.command {
                        ast::Command::Node(nodeid) => {}
                        _ => {}
                    }
                }
            }
            _ => {}
        }

        self.spec_explication_data.insert(
            funclet_name,
            SpecFuncletData {
                node_dependencies,
                tail_dependencies,
                deduced_types,
                connections: vec![],
            },
        );
    }

    fn initialize_schedule_funclet_info(&mut self, funclet_name: ast::FuncletId) {
        let funclet = self.get_funclet_mut(&funclet_name);
        match &funclet.kind {
            ir::FuncletKind::ScheduleExplicit => match &funclet.header.binding {
                ast::FuncletBinding::ScheduleBinding(binding) => {
                    let specs = SpecLanguages {
                        value: reject_hole_clone(&binding.value),
                        timeline: reject_hole_clone(&binding.timeline),
                        spatial: reject_hole_clone(&binding.spatial),
                    };
                    self.get_spec_info_mut(&specs.value)
                        .connections
                        .push(funclet_name.clone());
                    self.get_spec_info_mut(&specs.timeline)
                        .connections
                        .push(funclet_name.clone());
                    self.get_spec_info_mut(&specs.spatial)
                        .connections
                        .push(funclet_name.clone());
                    self.schedule_explication_data.insert(
                        funclet_name,
                        ScheduleFuncletData {
                            specs,
                            type_instantiations: Default::default(),
                        },
                    );
                }
                _ => {
                    unreachable!("Expected schedule binding for {:?}", funclet);
                }
            },
            _ => {}
        }
    }

    // for quick and dirty things
    pub fn program(&self) -> &ast::Program {
        &self.program
    }

    pub fn schedule_funclet_ids(&self) -> Vec<FuncletId> {
        let mut result = Vec::new();
        for declaration in &self.program.declarations {
            match declaration {
                ast::Declaration::Funclet(f) => match f.kind {
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

    pub fn command_ids(&self, funclet: &ast::FuncletId) -> Vec<NodeId> {
        let mut result = Vec::new();
        for command in &self.get_funclet(funclet).commands {
            match &command.command {
                ast::Command::Node(_) => {
                    result.push(command.name.as_ref().unwrap().clone());
                }
                ast::Command::Hole => {
                    result.push(command.name.as_ref().unwrap().clone());
                }
                ast::Command::TailEdge(_) => {}
                ast::Command::ExplicationHole => unreachable!(),
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

    fn get_type_decl(&self, typ: &ast::TypeId) -> Option<&LocalTypeDeclaration> {
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
                    ast::Command::Node(ast::Node::ExtractResult { node_id, index }) => {
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

    pub fn get_type_place(&self, typ: &ast::TypeId) -> Option<&ir::Place> {
        self.get_type_decl(typ).and_then(|t| (&t.place).as_ref())
    }

    pub fn get_type_ffi(&self, typ: &ast::TypeId) -> Option<&ast::FFIType> {
        self.get_type_decl(typ).and_then(|t| (&t.ffi).as_ref())
    }

    pub fn get_latest_explication_hole(&self) -> Option<&FuncletId> {
        for scope in self.scopes.iter().rev() {
            if scope.explication_hole {
                return Some(scope.funclet_name)
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

    pub fn get_funclet(&self, funclet: &FuncletId) -> &ast::Funclet {
        for declaration in &self.program.declarations {
            match declaration {
                ast::Declaration::Funclet(f) => {
                    if &f.header.name == funclet {
                        return f;
                    }
                }
                _ => {}
            }
        }
        panic!("Unknown funclet {:?}", funclet);
    }

    pub fn get_command(&self, funclet: &FuncletId, name: &NodeId) -> &ast::Command {
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

    pub fn get_node(&self, funclet: &FuncletId, name: &NodeId) -> &ast::Node {
        match self.get_command(funclet, name) {
            ast::Command::Node(n) => n,
            _ => panic!(
                "Attempted to treat command {:?} in funclet {:?} as a node",
                name, funclet
            ),
        }
    }

    pub fn get_tail_edge(&self, funclet_name: &FuncletId) -> Option<&ast::TailEdge> {
        for command in &self.get_funclet(funclet_name).commands {
            match &command.name {
                None => {}
                Some(n) => match &command.command {
                    ast::Command::TailEdge(edge) => {
                        return Some(edge);
                    }
                    _ => {}
                },
            }
        }
        None
    }
}

fn identify_node_deps(node: &ast::Node) -> Vec<NodeId> {
    let dependencies = match node {
        ast::Node::Phi { index } => {
            vec![]
        }
        ast::Node::ExtractResult { node_id, index } => {
            vec![reject_hole_clone(node_id)]
        }
        ast::Node::Constant { value, type_id } => vec![],
        ast::Node::CallFunctionClass {
            function_id,
            arguments,
        } => reject_hole(arguments.as_ref())
            .iter()
            .map(|n| reject_hole_clone(n))
            .collect(),
        ast::Node::Select {
            condition,
            true_case,
            false_case,
        } => {
            vec![
                reject_hole_clone(condition),
                reject_hole_clone(true_case),
                reject_hole_clone(false_case),
            ]
        }
        ast::Node::EncodingEvent {
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
        ast::Node::SubmissionEvent { local_past } => {
            vec![reject_hole_clone(local_past)]
        }
        ast::Node::SynchronizationEvent {
            local_past,
            remote_local_past,
        } => {
            vec![
                reject_hole_clone(local_past),
                reject_hole_clone(remote_local_past),
            ]
        }
        ast::Node::SeparatedBufferSpaces { count, space } => {
            vec![reject_hole_clone(space)]
        }
        _ => {
            unreachable!("Unsupported named specification node type {:?}", &node);
        }
    };
    dependencies
}

// helper methods for reading information
fn identify_tailedge_deps(edge: &ast::TailEdge) -> Vec<NodeId> {
    let dependencies = match edge {
        ast::TailEdge::DebugHole { inputs } => inputs.iter().map(|n| n.clone()).collect(),
        ast::TailEdge::Return { return_values } => reject_hole(return_values.as_ref())
            .iter()
            .map(|n| reject_hole_clone(n))
            .collect(),
        ast::TailEdge::Jump { join, arguments } => vec![reject_hole_clone(join)]
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

fn get_spec_node<'a>(funclet: &'a ast::Funclet, node: &NodeId) -> &'a ast::Node {
    for command in &funclet.commands {
        if command.name.as_ref().unwrap() == node {
            match &command.command {
                ast::Command::Node(node) => {
                    return node;
                }
                _ => unreachable!("Attempting to treat spec element {:?} as a node", command),
            }
        }
    }
    unreachable!("Unknown spec node dependency {:?}", node)
}

fn get_type_info<'a>(
    funclet: &'a ast::Funclet,
    nodeid: &NodeId,
    node_dependencies: &HashMap<NodeId, Vec<NodeId>>,
    type_map: &mut HashMap<NodeId, Vec<TypeId>>,
) -> Vec<&'a ast::TypeId> {
    if let Some(result) = type_map.get(nodeid) {
        result
    } else {
        let result = deduce_type(funclet, nodeid, node_dependencies, type_map);
        type_map.insert(nodeid.clone(), result.iter().cloned().collect());
        result
    }
}

fn deduce_type<'a>(
    funclet: &'a ast::Funclet,
    check_id: &NodeId,
    node_dependencies: &HashMap<NodeId, Vec<NodeId>>,
    type_map: &mut HashMap<NodeId, Vec<TypeId>>,
) -> Vec<&'a TypeId> {
    let names = node_dependencies.get(&check_id).unwrap_or_else(|| {
        unreachable!(unreachable!("Unknown spec node dependency {:?}", check_id))
    });
    let node = get_spec_node(funclet, &check_id);
    let typ = match node {
        ast::Node::Phi { index } => vec![
            &funclet
                .header
                .args
                .get(index.unwrap())
                .as_ref()
                .unwrap()
                .typ,
        ],
        ast::Node::ExtractResult { node_id, index } => {
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
        ast::Node::Constant { value, type_id } => {}
        ast::Node::CallFunctionClass {
            function_id,
            arguments,
        } => {}
        ast::Node::Select {
            condition,
            true_case,
            false_case,
        } => {}
        _ => unreachable!("Not a value node {:?}", node),
    };
    typ
}
