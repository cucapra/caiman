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
        &self.program
    }

    // get the specification type of the value node (if known)
    pub fn get_spec_instantiation (
        &self,
        funclet: &FuncletId,
        node: &NodeId,
    ) -> Option<&ast::RemoteNodeId> {
        self.schedule_explication_data
            .get(funclet)
            .and_then(|f| f.type_instantiations.get(node))
    }

    pub fn get_current_value_funclet(&self) -> Option<&FuncletId> {
        self.schedule_explication_data
            .get(&self.location.funclet.as_ref().unwrap())
            .map(|f| &f.value_funclet)
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

    // get a reference of the spec node at the given location (if one exists)
    pub fn get_type_ref(
        &self,
        funclet: FuncletId,
        node: NodeId,
        place: ir::Place,
    ) -> Option<&NodeId> {
        let info = ScheduledInstantiationInfo {
            funclet,
            node,
            place,
            is_value: false,
        };
        self.get_scoped(info, |s| &s.instantiations)
    }

    // get a value instantiation for the spec node at the given location (if one exists)
    pub fn get_type_instantiation(
        &self,
        funclet: FuncletId,
        node: NodeId,
        place: ir::Place,
    ) -> Option<&NodeId> {
        let info = ScheduledInstantiationInfo {
            funclet,
            node,
            place,
            is_value: true,
        };
        self.get_scoped(info, |s| &s.instantiations)
    }

    // SKIP
    pub fn get_latest_explication_hole(&self) -> Option<&NodeId> {
        for scope in self.scopes.iter().rev() {
            match &scope.explication_hole {
                None => {}
                Some(hole) => {
                    return Some(hole);
                }
            }
        }
        None
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
