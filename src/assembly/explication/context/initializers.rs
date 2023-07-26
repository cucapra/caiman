use super::*;
use crate::assembly::explication::util::*;

impl<'context> Context<'context> {
    pub fn new(program: &'context mut ast::Program) -> Context<'context> {
        let mut context = Context {
            program: DebugIgnore(program),
            location: Default::default(),
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
        let mut spec_funclets = Vec::new();
        let mut schedule_funclets = Vec::new();
        for declaration in &mut self.program.declarations {
            match declaration {
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
                    node_dependencies.insert(
                        reject_hole_clone(&command.name),
                        Context::identify_node_deps(node),
                    );
                }
                ast::Command::TailEdge(edge) => {
                    tail_dependencies = Context::identify_tailedge_deps(edge);
                }
            }
        }
        self.spec_explication_data.insert(
            funclet_name,
            SpecFuncletData {
                node_dependencies,
                tail_dependencies,
                connections: vec![],
            },
        );
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

    fn initialize_schedule_funclet_info(&mut self, funclet_name: ast::FuncletId) {
        let funclet = self.get_funclet_mut(&funclet_name);
        match &funclet.kind {
            ir::FuncletKind::ScheduleExplicit => match &funclet.header.binding {
                ast::FuncletBinding::ScheduleBinding(binding) => {
                    let value_funclet = reject_hole_clone(&binding.value);
                    let timeline_funclet = reject_hole_clone(&binding.timeline);
                    let spatial_funclet = reject_hole_clone(&binding.spatial);
                    self.get_spec_info_mut(&value_funclet)
                        .connections
                        .push(funclet_name.clone());
                    self.get_spec_info_mut(&timeline_funclet)
                        .connections
                        .push(funclet_name.clone());
                    self.get_spec_info_mut(&spatial_funclet)
                        .connections
                        .push(funclet_name.clone());
                    self.schedule_explication_data.insert(
                        funclet_name,
                        ScheduleFuncletData {
                            value_funclet,
                            timeline_funclet,
                            spatial_funclet,
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
