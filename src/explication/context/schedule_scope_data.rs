use super::*;
use crate::explication::util::*;

impl ScheduleScopeData {
    pub fn new(funclet_id: FuncletId) -> ScheduleScopeData {
        ScheduleScopeData::new_inner(
            funclet_id,
            HashMap::new(),
            HashMap::new(),
            Vec::new(),
            HashMap::new(),
        )
    }

    fn new_inner(
        funclet_id: FuncletId,
        instantiations: HashMap<Location, HashSet<NodeId>>,
        node_type_information: HashMap<NodeId, StorageNodeInformation>,
        allocations: Vec<(NodeId, expir::Type)>,
        available_operations: HashMap<OpCode, Vec<NodeId>>,
    ) -> ScheduleScopeData {
        ScheduleScopeData {
            funclet_id,
            node_id: None,
            node_index: 0,
            instantiations,
            storage_node_information: node_type_information,
            available_operations,
            explication_hole: false,
        }
    }

    pub fn next_node(&mut self) {
        self.node_id = match self.node_id {
            None => Some(0),
            Some(x) => Some(x + 1),
        }
    }

    pub fn add_storage_node(
        &mut self,
        schedule_node: NodeId,
        typ: expir::Type,
        context: &StaticContext,
    ) {
        let check = self.storage_node_information.insert(
            schedule_node,
            StorageNodeInformation {
                instantiation: LocationTriple::new(),
                typ,
                timeline_manager: None
            },
        );
        assert!(
            check.is_none(),
            "Duplicate add of node {}",
            context.debug_info.node(&self.funclet_id, schedule_node)
        );
    }

    pub fn set_instantiation(
        &mut self,
        schedule_node: NodeId,
        instantiation: LocationTriple,
        context: &StaticContext,
    ) {
        // note that this may overwrite what a node instantiates
        // this is, of course, completely fine mechanically
        // but is also why backtracking is needed/complicated
        let current_instantiation = &mut self
            .storage_node_information
            .get_mut(&schedule_node)
            .expect(&format!(
                "Attempting to update Node {} without already having an instantiation",
                context.debug_info.node(&self.funclet_id, schedule_node)
            )).instantiation;

        // potentially modified when checking the timeline
        match instantiation {
            LocationTriple {
                value,
                timeline,
                spatial,
            } => {
                match value {
                    None => {}
                    Some(value) => {
                        self.instantiations
                            .entry(value.clone())
                            .or_insert(HashSet::new())
                            .insert(schedule_node);
                        current_instantiation.value = Some(value);
                    }
                };
                match timeline {
                    // by default, every instantiation happens at the current time
                    // we can specify another time with the timeline triple
                    None => {}
                    Some(timeline) => {
                        self.instantiations
                            .entry(timeline.clone())
                            .or_insert(HashSet::new())
                            .insert(schedule_node);
                        current_instantiation.timeline = Some(timeline);
                    }
                };
                match spatial {
                    None => {}
                    Some(spatial) => {
                        self.instantiations
                            .entry(spatial.clone())
                            .or_insert(HashSet::new())
                            .insert(schedule_node);
                        current_instantiation.spatial = Some(spatial);
                    }
                };
            }
        }
    }

    // helper for match_triple
    fn intersect_instantiations(
        &self,
        current: Option<HashSet<usize>>,
        location: &Option<Location>,
        context: &StaticContext,
    ) -> Option<HashSet<NodeId>> {
        match location
            .as_ref()
            .map(|v| self.instantiations.get(v).cloned().unwrap_or_default())
        {
            None => current,
            Some(matches) => match current {
                None => Some(matches),
                Some(current_matches) => {
                    Some(matches.intersection(&current_matches).cloned().collect())
                }
            },
        }
    }

    pub fn set_timeline_manager(
        &mut self,
        schedule_node: &NodeId,
        timeline_manager: NodeId,
        context: &StaticContext,
    ) {
        self
            .storage_node_information
            .get_mut(&schedule_node)
            .expect(&format!(
                "Attempting to update Node {} without already having an instantiation",
                context.debug_info.node(&self.funclet_id, *schedule_node)
            )).timeline_manager = Some(timeline_manager);
    }

    pub fn clear_timeline_manager(
        &mut self,
        schedule_node: &NodeId,
        context: &StaticContext,
    ) {
        self
            .storage_node_information
            .get_mut(&schedule_node)
            .expect(&format!(
                "Attempting to update Node {} without already having an instantiation",
                context.debug_info.node(&self.funclet_id, *schedule_node)
            )).timeline_manager = None;
    }

    /*
     * Returns a list of all instantiations that match
     *   _all_ three non-empty members of the triple
     * empty members of the triple are ignored
     */
    pub fn match_triple(
        &self,
        triple: &LocationTriple,
        context: &StaticContext,
    ) -> HashSet<NodeId> {
        let mut result = None;
        result = self.intersect_instantiations(result, &triple.value, context);
        result = self.intersect_instantiations(result, &triple.timeline, context);
        result = self.intersect_instantiations(result, &triple.spatial, context);
        result.unwrap_or_default()
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
