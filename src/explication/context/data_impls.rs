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
        node_type_information: HashMap<NodeId, (Location, expir::Type)>,
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

    pub fn add_instantiation(
        &mut self,
        location: Location,
        typ: expir::Type,
        schedule_node: NodeId,
        context: &StaticContext,
    ) {
        self.instantiations
            .entry(location.clone())
            .or_insert(Vec::new())
            .push(schedule_node);
        let check = self
            .node_type_information
            .insert(schedule_node, (location, typ));
        assert!(
            check.is_none(),
            "duplicate add of scheduling node index {}, aka node {}",
            schedule_node,
            context.debug_info.node(&self.funclet, schedule_node)
        );
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
