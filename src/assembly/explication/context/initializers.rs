use super::*;
use crate::assembly::explication::util::*;

impl<'context> Context<'context> {
    pub fn new(program: &'context mut ast::Program) -> Context<'context> {
        let mut context = Context {
            program,
            location: Default::default(),
            value_explication_data: HashMap::new(),
            schedule_explication_data: HashMap::new(),
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
        for declaration in &self.program.declarations {
            match declaration {
                ast::Declaration::TypeDecl(_) => {}
                ast::Declaration::ExternalFunction(_) => {}
                ast::Declaration::FunctionClass(_) => {}
                ast::Declaration::Funclet(funclet) => self.initialize_funclet_info(funclet),
                ast::Declaration::Pipeline(_) => {}
            }
        }
    }

    fn initialize_funclet_info(&mut self, funclet: &ast::Funclet) {
        match &funclet.kind {
            ir::FuncletKind::Value => self.initialize_spec_funclet_info(&funclet.commands),
            ir::FuncletKind::Timeline => self.initialize_spec_funclet_info(&funclet.commands),
            ir::FuncletKind::Spatial => self.initialize_spec_funclet_info(&funclet.commands),
            ir::FuncletKind::ScheduleExplicit => {}
            ir::FuncletKind::Unknown => {}
        };
    }

    fn initialize_spec_funclet_info(&mut self, commands: &Vec<ast::NamedCommand>) {
        let mut node_dependencies = HashMap::new();
        let mut tail_dependencies = Vec::new();
        for command in commands {
            match &command.command {
                ast::Command::Hole => {}
                ast::Command::Node(node) => {
                    node_dependencies.insert(
                        reject_hole_clone(&command.name),
                        Context::initialize_node_deps(node),
                    );
                }
                ast::Command::TailEdge(edge) => {
                    tail_dependencies = Context::initialize_tailedge_deps(edge);
                }
            }
        }
        let spec_funclet_data = SpecFuncletData {
            node_dependencies,
            tail_dependencies,
            connections: vec![],
            explication_information: Default::default(),
            call_outputs: Default::default(),
        };
    }

    fn initialize_node_deps(node: &ast::Node) -> Vec<NodeId> {
        let dependencies = match node {
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

    fn initialize_tailedge_deps(edge: &ast::TailEdge) -> Vec<NodeId> {
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

    fn initialize_allocations(&mut self) {}
}
