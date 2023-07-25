use super::*;

// built to use evil.py to generate stuff
// any function to be generated must be `pub\s+fn` with any whitespace `\s+`
//   and must start with the word `get`
// skips any function that has // SKIP above it
// completely legit programming, I promise
// note that this only applies to _top level_ functions
// inner functions are just grabbed as normal
// note that I assume `->` is on the same line as the return type and `{`

impl<'context> Context<'context> {
    pub fn program(&self) -> &ast::Program {
        self.program
    }

    // get what the associated schedule node is allocating
    pub fn get_value_allocation(
        &self,
        funclet: &FuncletId,
        node: &NodeId,
    ) -> Option<&ast::RemoteNodeId> {
        self.schedule_explication_data
            .get(funclet)
            .and_then(|f| f.allocations.get(node))
    }

    pub fn get_current_value_funclet(&self) -> Option<&FuncletId> {
        self.schedule_explication_data
            .get(&self.location.funclet.as_ref().unwrap())
            .map(|f| &f.value_funclet)
    }

    // get allocations of the associated value node
    pub fn get_schedule_allocations(
        &self,
        funclet: &FuncletId,
        node: &NodeId,
    ) -> Option<&HashMap<AllocationInfo, NodeId>> {
        self.spec_explication_data.get(funclet).and_then(|f| {
            f.explication_information
                .get(node)
                .map(|n| &n.scheduled_allocations)
        })
    }

    // SKIP
    pub fn get_current_schedule_allocation(&self, node: &NodeId) -> Option<&NodeId> {
        self.get_current_value_funclet().and_then(|vf| {
            self.get_schedule_allocations(vf, node)
                .unwrap()
                .get(&AllocationInfo {
                    schedule_funclet: self.location_funclet().clone(),
                    place: ir::Place::Gpu, // todo: make actually correct
                })
        })
    }

    pub fn get_funclet(&self, funclet_name: &FuncletId) -> &ast::Funclet {
        for declaration in &self.program.declarations {
            match declaration {
                ast::Declaration::Funclet(f) => {
                    if &f.header.name == funclet_name {
                        return f;
                    }
                }
                _ => {}
            }
        }
        panic!("Unknown funclet {:?}", funclet_name);
    }

    pub fn get_node(&self, funclet_name: &FuncletId, node_name: &NodeId) -> &ast::Node {
        for command in &self.get_funclet(funclet_name).commands {
            match &command.name {
                None => {}
                Some(n) => {
                    if n == node_name {
                        match &command.command {
                            ast::Command::Node(node) => {
                                return node;
                            }
                            _ => unreachable!("Attempting to treat {} as a node", n.0),
                        }
                    }
                }
            }
        }
        panic!("Unknown node {:?} in funclet {:?}", node_name, funclet_name);
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
