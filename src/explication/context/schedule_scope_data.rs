use std::thread::current;

use itertools::Itertools;

use super::*;
use crate::explication::util::*;

impl ScheduleScopeData {
    pub fn new_operation(funclet_id: FuncletId) -> ScheduleScopeData {
        ScheduleScopeData::new_inner_operation(funclet_id, HashSet::new(), HashSet::new())
    }

    pub fn new_storage(funclet_id: FuncletId) -> ScheduleScopeData {
        ScheduleScopeData::new_inner_storage(funclet_id, HashMap::new(), HashMap::new())
    }

    fn new_inner_operation(
        funclet_id: FuncletId,
        value_operations: HashSet<Location>,
        timeline_operations: HashSet<Location>,
    ) -> ScheduleScopeData {
        ScheduleScopeData {
            funclet_id,
            node_id: None,
            node_index: 0,
            explication_hole: false,
            pass_information: PassInformation::Operation(OperationPassInformation {
                value_operations,
                timeline_operations,
            }),
        }
    }

    fn new_inner_storage(
        funclet_id: FuncletId,
        instantiations: HashMap<Location, HashSet<NodeId>>,
        storage_node_information: HashMap<NodeId, StorageNodeInformation>,
    ) -> ScheduleScopeData {
        ScheduleScopeData {
            funclet_id,
            node_id: None,
            node_index: 0,
            explication_hole: false,
            pass_information: PassInformation::Storage(StoragePassInformation {
                instantiations,
                storage_node_information,
            }),
        }
    }

    // If we're not in the operation pass, panics
    pub fn as_operation(&self) -> &OperationPassInformation {
        match &self.pass_information {
            PassInformation::Operation(o) => o,
            PassInformation::Storage(_) => unreachable!(),
        }
    }

    // If we're not in the allocation pass, panics
    pub fn as_storage(&self) -> &StoragePassInformation {
        match &self.pass_information {
            PassInformation::Operation(_) => unreachable!(),
            PassInformation::Storage(s) => s,
        }
    }

    // If we're not in the operation pass, panics
    fn as_operation_mut(&mut self) -> &mut OperationPassInformation {
        match &mut self.pass_information {
            PassInformation::Operation(o) => o,
            PassInformation::Storage(_) => unreachable!(),
        }
    }

    // If we're not in the allocation pass, panics
    fn as_storage_mut(&mut self) -> &mut StoragePassInformation {
        match &mut self.pass_information {
            PassInformation::Operation(_) => unreachable!(),
            PassInformation::Storage(s) => s,
        }
    }

    pub fn next_node(&mut self) {
        self.node_id = match self.node_id {
            None => Some(0),
            Some(x) => Some(x + 1),
        }
    }

    pub fn add_value_operation(&mut self, operation: Location, context: &StaticContext) {
        self.as_operation_mut().value_operations.insert(operation);
    }

    pub fn has_value_operation(&self, operation: &Location, context: &StaticContext) -> bool {
        self.as_operation().value_operations.contains(operation)
    }

    pub fn add_timeline_operation(&mut self, operation: Location, context: &StaticContext) {
        self.as_operation_mut().timeline_operations.insert(operation);
    }

    pub fn has_timeline_operation(&self, operation: &Location, context: &StaticContext) -> bool {
        self.as_operation().timeline_operations.contains(operation)
    }

    pub fn add_storage_node(
        &mut self,
        schedule_node: NodeId,
        typ: Hole<expir::Type>,
        context: &StaticContext,
    ) {
        let check = self.as_storage_mut().storage_node_information.insert(
            schedule_node,
            StorageNodeInformation {
                instantiation: None,
                typ,
            },
        );
        assert!(
            check.is_none(),
            "Duplicate add of node {}",
            context.debug_info.node(&self.funclet_id, schedule_node)
        );
    }

    pub fn set_storage_type(
        &mut self,
        schedule_node: NodeId,
        typ: expir::Type,
        context: &StaticContext,
    ) {
        let funclet_id = self.funclet_id.clone();
        self.as_storage_mut()
            .storage_node_information
            .get_mut(&schedule_node)
            .expect(&format!(
                "Missing storage node {}",
                context.debug_info.node(&funclet_id, schedule_node)
            ))
            .typ = Hole::Filled(typ);
    }

    pub fn set_instantiation(
        &mut self,
        schedule_node: NodeId,
        instantiation: LocationTriple,
        context: &StaticContext,
    ) {
        let funclet_id = self.funclet_id;

        // note that this may overwrite what a node instantiates
        // this is, of course, completely fine mechanically
        // but is also why backtracking is needed/complicated
        let mut new_instantiations = self
            .as_storage_mut()
            .storage_node_information
            .get_mut(&schedule_node)
            .expect(&format!(
                "Attempting to update Node {} without already having an instantiation",
                context.debug_info.node(&funclet_id, schedule_node)
            ))
            .instantiation
            .take()
            .unwrap_or(LocationTriple::new());

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
                        self.as_storage_mut()
                            .instantiations
                            .entry(value.clone())
                            .or_insert(HashSet::new())
                            .insert(schedule_node);
                        match &new_instantiations.value {
                            Some(location) => {
                                self.as_storage_mut()
                                    .instantiations
                                    .get_mut(location)
                                    .unwrap()
                                    .remove(&schedule_node);
                            }
                            None => {}
                        }
                        new_instantiations.value = Some(value);
                    }
                };
                match timeline {
                    // by default, every instantiation happens at the current time
                    // we can specify another time with the timeline triple
                    None => {}
                    Some(timeline) => {
                        self.as_storage_mut()
                            .instantiations
                            .entry(timeline.clone())
                            .or_insert(HashSet::new())
                            .insert(schedule_node);
                        match &new_instantiations.timeline {
                            Some(location) => {
                                self.as_storage_mut()
                                    .instantiations
                                    .get_mut(location)
                                    .unwrap()
                                    .remove(&schedule_node);
                            }
                            None => {}
                        }
                        new_instantiations.timeline = Some(timeline);
                    }
                };
                match spatial {
                    None => {}
                    Some(spatial) => {
                        self.as_storage_mut()
                            .instantiations
                            .entry(spatial.clone())
                            .or_insert(HashSet::new())
                            .insert(schedule_node);
                        match &new_instantiations.spatial {
                            Some(location) => {
                                self.as_storage_mut()
                                    .instantiations
                                    .get_mut(location)
                                    .unwrap()
                                    .remove(&schedule_node);
                            }
                            None => {}
                        }
                        new_instantiations.spatial = Some(spatial);
                    }
                };
            }
        }

        self.as_storage_mut()
            .storage_node_information
            .get_mut(&schedule_node)
            .expect(&format!(
                "Attempting to update Node {} without already having an instantiation",
                context.debug_info.node(&funclet_id, schedule_node)
            ))
            .instantiation = Some(new_instantiations);
    }

    pub fn get_node_information(
        &self,
        node_id: &NodeId,
        context: &StaticContext,
    ) -> &StorageNodeInformation {
        self.as_storage()
            .storage_node_information
            .get(node_id)
            .expect(&format!(
                "Missing information for node {}",
                context.debug_info.node(&self.funclet_id, *node_id)
            ))
    }

    // unsorted vector of nodes of the given storage type
    pub fn storage_of_type(
        &self,
        target_type: &expir::Type,
        context: &StaticContext,
    ) -> Vec<NodeId> {
        self.as_storage()
            .storage_node_information
            .iter()
            .filter(|(_, info)| match &info.typ {
                Hole::Empty => true,
                Hole::Filled(typ) => is_of_type(target_type, typ),
            })
            .map(|(n, _)| n.clone())
            .collect()
    }

    // helper for match_triple
    fn intersect_instantiations(
        &self,
        current: Option<HashSet<usize>>,
        location: &Option<Location>,
        context: &StaticContext,
    ) -> Option<HashSet<NodeId>> {
        match location.as_ref().map(|v| {
            self.as_storage()
                .instantiations
                .get(v)
                .cloned()
                .unwrap_or_default()
        }) {
            None => current,
            Some(matches) => match current {
                None => Some(matches),
                Some(current_matches) => {
                    Some(matches.intersection(&current_matches).cloned().collect())
                }
            },
        }
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

    pub fn add_explication_hole(&mut self) {
        self.explication_hole = true;
    }
}
