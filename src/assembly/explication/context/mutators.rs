use super::*;

impl<'context> Context<'context> {
    pub fn corrections(&mut self) {
        for declaration in self.program.declarations.iter_mut() {
            match declaration {
                ast::Declaration::Funclet(f) => {
                    let mut index = 0;
                    for arg in &f.header.args {
                        f.commands.insert(
                            index,
                            Some(ast::Command::Node(ast::NamedNode {
                                name: None,
                                node: ast::Node::Phi { index: Some(index) },
                            })),
                        );
                        index += 1;
                    }
                    for command in f.commands.iter_mut() {
                        match command {
                            Some(ast::Command::Node(ast::NamedNode { node, name })) => {
                                match name {
                                    None => {}
                                    Some(n) => {
                                        if n.0 == "_" {
                                            n.0 = self.meta_data.next_name()
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn forcibly_replace_commands(
        &mut self,
        funclet: &ast::FuncletId,
        commands: Vec<Hole<ast::Command>>,
    ) {
        self.get_funclet_mut(funclet).unwrap().commands = commands;
    }

    pub fn add_remote_node(&mut self, funclet: &ast::FuncletId, node: ast::NamedNode) {}

    pub fn add_node(&mut self, node: ast::NamedNode) {
        match node {
            _ => {}
        }
    }

    pub fn add_allocation(&mut self, value_node: NodeId, schedule_node: NodeId) {
        let schedule_funclet = self.location.funclet_name.clone();
        let value_funclet = &self
            .schedule_explication_data
            .get(&schedule_funclet)
            .unwrap()
            .value_funclet;

        // unwrap explicitly cause we assume funclet data are setup earlier
        self.value_explication_data
            .get_mut(value_funclet)
            .unwrap()
            .allocate(
                value_node.clone(),
                schedule_funclet.clone(),
                schedule_node.clone(),
            );

        self.schedule_explication_data
            .get_mut(&schedule_funclet)
            .unwrap()
            .allocate(schedule_node, value_node);
    }
}

// The weird stuff that didn't have a clear way to autogenerate

impl<'context> Context<'context> {
    fn get_current_schedule_allocation_mut(&mut self, node: &NodeId) -> Option<&mut NodeId> {
        let funclet_name = self.location.funclet_name.clone();
        match self.get_current_value_funclet() {
            None => None,
            Some(vf) => self
                .get_schedule_allocations_mut(&vf.clone(), node)
                .and_then(|sf| sf.get_mut(&funclet_name)),
        }
    }
}

// THIS AND THE FOLLOWING CODE IS GENERATED WITH evil.py, DO NOT TOUCH

impl<'context> Context<'context> {
    fn get_value_allocation_mut(
        &mut self,
        funclet: &FuncletId,
        node: &NodeId,
    ) -> Option<&mut ast::NodeId> {
        self.schedule_explication_data
            .get_mut(funclet)
            .and_then(|f| f.allocations.get_mut(node))
    }

    fn get_current_value_funclet_mut(&mut self) -> Option<&mut FuncletId> {
        self.schedule_explication_data
            .get_mut(&mut self.location.funclet_name)
            .map(|f| &mut f.value_funclet)
    }

    fn get_schedule_allocations_mut(
        &mut self,
        funclet: &FuncletId,
        node: &NodeId,
    ) -> Option<&mut HashMap<FuncletId, NodeId>> {
        self.value_explication_data.get_mut(funclet).and_then(|f| {
            f.explication_information
                .get_mut(node)
                .map(|n| &mut n.scheduled_allocations)
        })
    }

    fn get_funclet_mut(&mut self, funclet_name: &FuncletId) -> Option<&mut ast::Funclet> {
        for declaration in &mut self.program.declarations {
            match declaration {
                ast::Declaration::Funclet(f) => {
                    if &mut f.header.name == funclet_name {
                        return Some(f);
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn get_node_mut(
        &mut self,
        funclet_name: &FuncletId,
        node_name: &NodeId,
    ) -> Option<&mut ast::Node> {
        self.get_funclet_mut(funclet_name).and_then(|f| {
            for command in &mut f.commands {
                match command {
                    Some(ast::Command::Node(ast::NamedNode { name, node })) => {
                        match name {
                            None => {}
                            Some(n) => if n == node_name {
                                return Some(node);
                            }
                        }
                    }
                    _ => {}
                }
            }
            None
        })
    }

    fn get_tail_edge_mut(&mut self, funclet_name: &FuncletId) -> Option<&mut ast::TailEdge> {
        self.get_funclet_mut(funclet_name).and_then(|f| {
            for command in &mut f.commands {
                match command {
                    Some(ast::Command::TailEdge(t)) => return Some(t),
                    _ => {}
                }
            }
            None
        })
    }
}
