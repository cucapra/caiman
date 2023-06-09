use super::*;

// built to use evil.py to generate stuff
// any function to be generated must be `pub\s+fn` with any whitespace `\s+`
//   and must start with the word `get`
// completely legit programming, I promise
// note that this only applies to _top level_ functions
// inner functions are just grabbed as normal
// note that I assume `->` is on the same line as the return type and `{`

impl<'context> Context<'context> {
    pub fn new(program: &'context mut ast::Program) -> Context<'context> {
        Context {
            program,
            location: LocationNames::new(),
            value_explication_data: HashMap::new(),
            schedule_explication_data: HashMap::new(),
            meta_data: MetaData::new(),
        }
    }

    pub fn program(&self) -> &ast::Program {
        self.program
    }

    // get allocations of the associated value node
    pub fn get_schedule_allocations(
        &self,
        funclet: &FuncletId,
        node: &NodeId,
    ) -> Option<&HashMap<FuncletId, NodeId>> {
        self.value_explication_data.get(funclet).and_then(|f| {
            f.explication_information
                .get(node)
                .map(|n| &n.scheduled_allocations)
        })
    }

    pub fn get_current_schedule_allocation(&self, node: &NodeId) -> Option<&NodeId> {
        self.get_current_value_funclet().and_then(|vf| {
            self.get_schedule_allocations(vf, node)
                .unwrap()
                .get(&self.location.funclet_name)
        })
    }

    // get what the associated schedule node is allocating
    pub fn get_value_allocation(&self, funclet: &FuncletId, node: &NodeId) -> Option<&ast::NodeId> {
        self.schedule_explication_data
            .get(funclet)
            .and_then(|f| f.allocations.get(node))
    }

    pub fn get_current_value_funclet(&self) -> Option<&FuncletId> {
        self.schedule_explication_data
            .get(&self.location.funclet_name)
            .map(|f| &f.value_funclet)
    }

    pub fn get_funclet(&self, funclet_name: &FuncletId) -> Option<&ast::Funclet> {
        for declaration in &self.program.declarations {
            match declaration {
                ast::Declaration::Funclet(f) => {
                    if &f.header.name == funclet_name {
                        return Some(f);
                    }
                }
                _ => {}
            }
        }
        None
    }

    pub fn get_node(&self, funclet_name: &FuncletId, node_name: &NodeId) -> Option<&ast::Node> {
        self.get_funclet(funclet_name).and_then(|f| {
            for command in &f.commands {
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

    pub fn get_tail_edge(&self, funclet_name: &FuncletId) -> Option<&ast::TailEdge> {
        self.get_funclet(funclet_name).and_then(|f| {
            for command in &f.commands {
                match command {
                    Some(ast::Command::TailEdge(t)) => return Some(t),
                    _ => {}
                }
            }
            None
        })
    }
}
