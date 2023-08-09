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
    pub fn get_spec_funclet_mut(&mut self, funclet: &FuncletId, spec: &SpecLanguage) -> &mut FuncletId {
        self.get_schedule_info_mut(funclet).specs.get_mut(spec)
    }

    pub fn get_spec_instantiation_mut(
        &mut self,
        funclet: &FuncletId,
        node: &NodeId,
        spec: &SpecLanguage,
    ) -> Option<&mut NodeId> {
        self.get_schedule_info_mut(funclet)
            .type_instantiations
            .get_mut(node)
            .and_then(|inst| inst.get_mut(spec))
    }

    pub fn get_value_funclet_mut(&mut self, schedule: &FuncletId) -> Option<&mut FuncletId> {
        self.schedule_explication_data
            .get_mut(schedule)
            .map(|f| &mut f.specs.value)
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

    pub fn get_command_mut(&mut self, funclet: &FuncletId, name: &NodeId) -> &mut ast::Command {
        for command in &mut self.get_funclet_mut(funclet).commands {
            match &mut command.name {
                None => {}
                Some(n) => {
                    if n == name {
                        {
                            return &mut command.command;
                        }
                    }
                }
            }
        }
        panic!("Unknown command {:?} in funclet {:?}", name, funclet);
    }

    pub fn get_node_mut(&mut self, funclet: &FuncletId, name: &NodeId) -> &mut ast::Node {
        match self.get_command_mut(funclet, name) {
            ast::Command::Node(n) => n,
            _ => panic!(
                "Attempted to treat command {:?} in funclet {:?} as a node",
                name, funclet
            ),
        }
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