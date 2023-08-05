use super::*;

impl<'context> Context<'context> {
    pub fn get_spec_info_mut(&mut self, funclet: &FuncletId) -> &mut SpecFuncletData {
        match self.spec_explication_data.get_mut(funclet) {
            Some(data) => data,
            None => {
                panic!("Unknown specification functlet {:?}", funclet);
            }
        }
    }
    pub fn get_latest_scope(&mut self) -> &mut ScheduleScopeData {
        match self.scopes.last_mut() {
            None => unreachable!("Should be in a scope when doing explication"),
            Some(scope) => scope,
        }
    }
}

// weird stuff that's not nice to autogenerate
impl<'context> Context<'context> {
    fn get_scoped_mut<'a, T, U, V>(&'a mut self, info: T, map: U) -> Option<&mut V>
    where
        T: std::hash::Hash + PartialEq + Eq + 'a,
        U: Fn(&mut ScheduleScopeData) -> &mut HashMap<T, V>,
    {
        // takes advantage of the invariant that vectors of a key remove that key when emptied
        for scope in self.scopes.iter_mut().rev() {
            match map(scope).get_mut(&info) {
                None => {}
                Some(result) => {
                    return Some(result);
                }
            }
        }
        None
    }
}

// THIS AND THE FOLLOWING CODE IS GENERATED WITH evil.py, DO NOT TOUCH

impl<'context> Context<'context> {
    pub fn get_spec_instantiation_mut(
        &mut self,
        funclet: &FuncletId,
        node: &NodeId,
    ) -> Option<&mut Vec<ast::RemoteNodeId>> {
        self.schedule_explication_data
            .get_mut(funclet)
            .and_then(|f| f.type_instantiations.get_mut(node))
    }

    pub fn get_value_funclet_mut(&mut self, schedule: &FuncletId) -> Option<&mut FuncletId> {
        self.schedule_explication_data
            .get_mut(schedule)
            .map(|f| &mut f.value_funclet)
    }

    pub fn get_type_instantiations_mut(
        &mut self,
        funclet: FuncletId,
        node: NodeId,
        place: Option<ir::Place>,
    ) -> Option<&mut Vec<NodeId>> {
        let info = ScheduledInstantiationInfo {
            funclet,
            node,
            place,
        };
        self.get_scoped_mut(info, |s| &mut s.instantiations)
    }

    pub fn get_funclet_mut(&mut self, funclet_name: &FuncletId) -> &mut ast::Funclet {
        for declaration in &mut self.program.declarations {
            match declaration {
                ast::Declaration::Funclet(f) => {
                    if &mut f.header.name == funclet_name {
                        return f;
                    }
                }
                _ => {}
            }
        }
        panic!("Unknown funclet {:?}", funclet_name);
    }

    pub fn get_node_mut(&mut self, funclet_name: &FuncletId, node_name: &NodeId) -> &mut ast::Node {
        for command in &mut self.get_funclet_mut(funclet_name).commands {
            match &mut command.name {
                None => {}
                Some(n) => {
                    if n == node_name {
                        match &mut command.command {
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

    pub fn get_tail_edge_mut(&mut self, funclet_name: &FuncletId) -> Option<&mut ast::TailEdge> {
        for command in &mut self.get_funclet_mut(funclet_name).commands {
            match &mut command.name {
                None => {}
                Some(n) => match &mut command.command {
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