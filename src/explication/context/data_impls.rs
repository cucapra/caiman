use super::*;
use crate::explication::util::*;

impl InstantiatedNodes {
    pub fn new(specs: &SpecLanguages, remotes: Vec<Location>) -> InstantiatedNodes {
        let mut result = InstantiatedNodes {
            value: None,
            timeline: None,
            spatial: None,
        };
        let error = format!("Duplicate value node definitions in {:?}", &remotes);
        for remote in remotes {
            let funclet = remote.funclet;
            let node = remote.node;
            if &funclet == &specs.value {
                if result.value.is_some() {
                    panic!(error);
                }
                result.value = Some(node);
            } else if &funclet == &specs.timeline {
                if result.timeline.is_some() {
                    panic!(error);
                }
                result.timeline = Some(node);
            } else if &funclet == &specs.spatial {
                if result.spatial.is_some() {
                    panic!(error);
                }
                result.spatial = Some(node);
            }
        }
        result
    }

    pub fn get(&self, spec: &SpecLanguage) -> Option<&NodeId> {
        match spec {
            SpecLanguage::Value => self.value.as_ref(),
            SpecLanguage::Timeline => self.timeline.as_ref(),
            SpecLanguage::Spatial => self.spatial.as_ref(),
        }
    }
}

impl ScheduleScopeData {
    pub fn new(funclet: FuncletId) -> ScheduleScopeData {
        ScheduleScopeData::new_inner(funclet, HashMap::new(), HashMap::new(), HashMap::new())
    }

    fn new_inner(
        funclet: FuncletId,
        instantiations: HashMap<Location, Vec<(ir::Place, NodeId)>>,
        allocations: HashMap<TypeId, Vec<(ir::Place, NodeId)>>,
        available_operations: HashMap<OpCode, Vec<NodeId>>,
    ) -> ScheduleScopeData {
        ScheduleScopeData {
            funclet,
            node: None,
            node_index: 0,
            instantiations,
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
        place: expir::Place,
        schedule_node: NodeId,
    ) {
        self.instantiations
            .entry(location)
            .or_insert(Vec::new())
            .push((place, schedule_node));
    }

    pub fn add_allocation(&mut self, typ: TypeId, place: expir::Place, schedule_node: NodeId) {
        self.allocations
            .entry(typ)
            .or_insert(Vec::new())
            .push((place, schedule_node));
    }

    pub fn add_available_operation(&mut self, node: NodeId, operation: OpCode) {
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
