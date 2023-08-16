use super::*;

// built to use evil.py to generate stuff
// any function to be generated must be `pub\s+fn` with any whitespace `\s+`
//   and must start with the word `get`
// skips any function that has // IMMUTABLE above it
// completely legit programming, I promise
// note that this only applies to _top level_ functions
// inner functions are just grabbed as normal
// note that I assume `->` is on the same line as the return type and `{`

impl<'context> Context<'context> {
    pub fn program(&self) -> &ast::Program {
        &self.program
    }

    // IMMUTABLE
    pub fn get_spec_funclet(&self, funclet: &FuncletId, spec: &SpecLanguage) -> &FuncletId {
        self.get_schedule_info(funclet).specs.get(spec)
    }

    // get the specification type of the value node (if known)
    // IMMUTABLE
    pub fn get_spec_instantiation(
        &self,
        funclet: &FuncletId,
        node: &NodeId,
        spec: &SpecLanguage,
    ) -> Option<&NodeId> {
        self.get_schedule_info(funclet)
            .type_instantiations
            .get(node)
            .and_then(|inst| inst.get(spec))
    }

    pub fn get_value_funclet(&self, schedule: &FuncletId) -> Option<&FuncletId> {
        self.schedule_explication_data
            .get(schedule)
            .map(|f| &f.specs.value)
    }

    // IMMUTABLE
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

    fn get_type_decl(&self, typ: &ast::TypeId) -> Option<&LocalTypeDeclaration> {
        match typ {
            TypeId::FFI(_) => None,
            TypeId::Local(type_name) => Some(
                self.type_declarations
                    .get(type_name)
                    .unwrap_or_else(|| panic!("Unknown type_name {:?}", type_name)),
            ),
        }
    }

    // IMMUTABLE
    pub fn get_matching_operation(
        &self,
        funclet: &FuncletId,
        returns: Vec<Hole<&NodeId>>,
    ) -> Option<&NodeId> {
        let mut result = None;
        let mut index_map = HashMap::new();
        for (index, ret) in returns.into_iter().enumerate() {
            match ret {
                None => {}
                Some(name) => {
                    index_map.insert(name.clone(), index);
                }
            }
        }
        let spec_data = self.get_spec_data(funclet);
        for command in &self.get_funclet(funclet).commands {
            let name = command.name.as_ref().unwrap();
            if index_map.contains_key(name) {
                match &command.command {
                    ast::Command::Node(ast::Node::ExtractResult { node_id, index }) => {
                        // make sure that the index matches the given argument
                        assert_eq!(index.as_ref().unwrap(), index_map.get(name).unwrap());
                        // lots of potentially panicking unwraps here
                        // none of these should be `None` at this point
                        let dependency = spec_data
                            .node_dependencies
                            .get(node_id.as_ref().unwrap())
                            .unwrap()
                            .first()
                            .unwrap();
                        // every extraction of one function should match to that function
                        result = assign_or_compare(result, dependency);
                    }
                    _ => panic!("Attempted to treat {:?} as an extract operation", command),
                }
            }
        }
        result
    }

    // IMMUTABLE
    pub fn get_type_place(&self, typ: &ast::TypeId) -> Option<&ir::Place> {
        self.get_type_decl(typ).and_then(|t| (&t.place).as_ref())
    }

    // IMMUTABLE
    pub fn get_type_ffi(&self, typ: &ast::TypeId) -> Option<&ast::FFIType> {
        self.get_type_decl(typ).and_then(|t| (&t.ffi).as_ref())
    }

    // get a value instantiation for the spec nodes at the given location
    pub fn get_type_instantiations(
        &self,
        funclet: FuncletId,
        node: NodeId,
        place: Option<ir::Place>,
    ) -> Option<&Vec<NodeId>> {
        let info = ScheduledInstantiationInfo {
            funclet,
            node,
            place,
        };
        self.get_scoped(info, |s| &s.instantiations)
    }

    // IMMUTABLE
    pub fn get_latest_type_instantiation(
        &self,
        funclet: FuncletId,
        node: NodeId,
        place: Option<ir::Place>,
    ) -> Option<&NodeId> {
        // returns the latest type instantiation if one exists in any scope
        let info = ScheduledInstantiationInfo {
            funclet,
            node,
            place,
        };
        for scope in self.scopes.iter().rev() {
            match scope.instantiations.get(&info) {
                None => {}
                Some(v) => match v.first() {
                    None => {}
                    Some(n) => {
                        return Some(n);
                    }
                },
            }
        }
        None
    }

    // IMMUTABLE
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

    // IMMUTABLE
    fn get_spec_data(&self, funclet: &FuncletId) -> &SpecFuncletData {
        match self.spec_explication_data.get(funclet) {
            Some(spec) => spec,
            None => panic!("Unknown specification function {:?}", funclet),
        }
    }

    fn get_schedule_info(&self, funclet: &FuncletId) -> &ScheduleFuncletData {
        match self.schedule_explication_data.get(funclet) {
            Some(data) => data,
            None => panic!("Unknown schedule funclet {:?}", funclet),
        }
    }

    pub fn get_funclet(&self, funclet: &FuncletId) -> &ast::Funclet {
        for declaration in &self.program.declarations {
            match declaration {
                ast::Declaration::Funclet(f) => {
                    if &f.header.name == funclet {
                        return f;
                    }
                }
                _ => {}
            }
        }
        panic!("Unknown funclet {:?}", funclet);
    }

    pub fn get_command(&self, funclet: &FuncletId, name: &NodeId) -> &ast::Command {
        for command in &self.get_funclet(funclet).commands {
            match &command.name {
                None => {}
                Some(n) => {
                    if n == name {
                        {
                            return &command.command;
                        }
                    }
                }
            }
        }
        panic!("Unknown command {:?} in funclet {:?}", name, funclet);
    }

    pub fn get_node(&self, funclet: &FuncletId, name: &NodeId) -> &ast::Node {
        match self.get_command(funclet, name) {
            ast::Command::Node(n) => n,
            _ => panic!(
                "Attempted to treat command {:?} in funclet {:?} as a node",
                name, funclet
            ),
        }
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
