use super::*;
use crate::assembly::explication::util::*;

impl<'context> Context<'context> {
    pub fn new(program: &'context mut ast::Program) -> Context<'context> {
        let mut context = Context {
            program: DebugIgnore(program),
            type_declarations: HashMap::new(),
            spec_explication_data: HashMap::new(),
            schedule_explication_data: HashMap::new(),
            scopes: Vec::new(),
            meta_data: MetaData::new(),
        };
        context.initialize();
        context
    }

    fn initialize(&mut self) {
        self.corrections();
        self.initialize_declarations();
    }

    fn corrections(&mut self) {
        for declaration in self.program.declarations.iter_mut() {
            match declaration {
                // add phi nodes
                ast::Declaration::Funclet(f) => {
                    let mut index = 0;
                    for arg in &f.header.args {
                        f.commands.insert(
                            index,
                            ast::NamedCommand {
                                name: arg.name.clone(),
                                command: ast::Command::Node(ast::Node::Phi { index: Some(index) }),
                            },
                        );
                        index += 1;
                    }
                    for command in f.commands.iter_mut() {
                        // give names to unnamed things (even tail edges, just in case)
                        command.name = match &command.name {
                            None => Some(NodeId(self.meta_data.next_name())),
                            n => n.clone(),
                        };
                    }
                }
                _ => {}
            }
        }
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
