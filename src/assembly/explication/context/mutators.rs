use super::*;

impl<'context> Context<'context> {
    fn allocation_name(&mut self) -> String {
        self.meta_data.name_index += 1;
        format!("${}", self.meta_data.name_index)
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

    pub fn next_name(&mut self) -> String {
        self.meta_data.next_name()
    }
}

// THIS AND THE FOLLOWING CODE IS GENERATED WITH evil.py, DO NOT TOUCH

impl<'context> Context<'context> {
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
    fn get_current_schedule_allocation_mut(&mut self, node: &NodeId) -> Option<&mut NodeId> {
        self.get_current_value_funclet_mut()
            .as_ref()
            .and_then(|vf| {
                self.get_schedule_allocations_mut(vf, node)
                    .unwrap()
                    .get_mut(&mut self.location.funclet_name)
            })
    }
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
                        if name == node_name {
                            return Some(node);
                        }
                    }
                    _ => {}
                }
            }
            None
        })
    }
}
