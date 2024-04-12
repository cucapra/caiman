use super::*;
use crate::explication::util::*;

impl ScheduleScopeData {
    pub fn new(funclet: FuncletId) -> ScheduleScopeData {
        ScheduleScopeData::new_inner(
            funclet,
            HashMap::new(),
            HashMap::new(),
            Vec::new(),
            HashMap::new(),
        )
    }

    fn new_inner(
        funclet: FuncletId,
        instantiations: HashMap<Location, Vec<NodeId>>,
        node_type_information: HashMap<NodeId, (LocationTriple, expir::Type)>,
        allocations: Vec<(NodeId, expir::Type)>,
        available_operations: HashMap<OpCode, Vec<NodeId>>,
    ) -> ScheduleScopeData {
        ScheduleScopeData {
            funclet,
            node: None,
            node_index: 0,
            instantiations,
            node_type_information,
            allocations,
            available_operations,
            explication_hole: false,
        }
    }

    pub fn next_node(&mut self) {
        self.node = match self.node {
            None => Some(0),
            Some(x) => Some(x + 1),
        }
    }

    pub fn set_instantiation(
        &mut self,
        schedule_node: NodeId,
        location_triple: LocationTriple,
        typ: expir::Type,
        context: &StaticContext,
    ) {
        match &location_triple.value {
            None => {},
            Some(value) => {
                self.instantiations
                    .entry(value.clone())
                    .or_insert(Vec::new())
                    .push(schedule_node);
            }
        }
        match &location_triple.timeline {
            None => {},
            Some(timeline) => {
                self.instantiations
                    .entry(timeline.clone())
                    .or_insert(Vec::new())
                    .push(schedule_node);
            }
        }
        match &location_triple.spatial {
            None => {},
            Some(spatial) => {
                self.instantiations
                    .entry(spatial.clone())
                    .or_insert(Vec::new())
                    .push(schedule_node);
            }
        }

        // note that this may overwrite what an allocation instantiates
        // this is, of course, completely fine mechanically
        // but is also why backtracking is needed/complicated
        self.node_type_information
            .insert(schedule_node, (location_triple, typ));
    }

    pub fn add_allocation(
        &mut self,
        schedule_node: NodeId,
        typ: expir::Type,
        context: &StaticContext,
    ) {
        self.allocations.push((schedule_node, typ));
    }

    pub fn add_available_operation(
        &mut self,
        node: NodeId,
        operation: OpCode,
        context: &StaticContext,
    ) {
        let vec = self
            .available_operations
            .entry(operation)
            .or_insert_with(|| Vec::new());
        // safety check that the algorithm isn't reinserting operations
        assert!(!vec.contains(&node));
        vec.push(node);
    }

    pub fn add_explication_hole(&mut self) {
        self.explication_hole = true;
    }
}

macro_rules! op_code_initialization {
    ($($_lang:ident $name:ident ($($_arg:ident : $_arg_type:tt,)*) -> $_output:ident;)*) => {
        impl OpCode {
            pub fn new(node: &expir::Node) -> OpCode {
                match node {
                    $(expir::Node::$name { .. } => OpCode::$name,)*
                }
            }
        }
    };
}

with_operations!(op_code_initialization);
