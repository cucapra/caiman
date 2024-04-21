use super::*;
use crate::explication::util;
use crate::explication::util::{Location, LocationTriple};
use itertools::Itertools;
use paste::paste;

impl InState {
    pub fn new(funclet_id: FuncletId, context: &StaticContext) -> InState {
        let scopes = vec![ScheduleScopeData::new(funclet_id)];
        InState { scopes }
    }

    // most of these functions are mutable
    // note that we are expected to clone the instate for each recursion
    // this way we avoid _problems_

    pub fn enter_funclet(&mut self, funclet_id: FuncletId, context: &StaticContext) {
        self.scopes.push(ScheduleScopeData::new(funclet_id));
    }
    pub fn exit_funclet(&mut self) -> bool {
        // returns if we have popped the lexpir element of the scope
        match self.scopes.pop() {
            None => panic!("Cannot leave a scope when there is no scope to leave"),
            Some(_) => {}
        }
        self.scopes.len() == 0
    }

    pub fn add_storage_node(
        &mut self,
        schedule_node: NodeId,
        typ: expir::Type,
        context: &StaticContext,
    ) {
        self.get_latest_scope_mut()
            .add_storage_node(schedule_node, typ, context);
    }

    pub fn set_instantiation(
        &mut self,
        schedule_node: NodeId,
        instantiation: LocationTriple,
        context: &StaticContext,
    ) {
        self.get_latest_scope_mut()
            .set_instantiation(schedule_node, instantiation, context);
    }

    pub fn set_timeline_manager(
        &mut self,
        schedule_node: &NodeId,
        timeline_manager: NodeId,
        context: &StaticContext,
    ) {
        self.get_latest_scope_mut()
            .set_timeline_manager(schedule_node, timeline_manager, context);
    }

    pub fn clear_timeline_manager(&mut self, schedule_node: &NodeId, context: &StaticContext) {
        self.get_latest_scope_mut()
            .clear_timeline_manager(schedule_node, context);
    }

    pub fn get_managed_by_timeline(
        &self,
        timeline_manager: NodeId,
        context: &StaticContext,
    ) -> Vec<NodeId> {
        self.get_latest_scope()
            .storage_node_information
            .iter()
            .filter(|(_, info)| {
                info.timeline_manager
                    .map(|o| o == timeline_manager)
                    .unwrap_or(false)
            })
            .map(|v| v.0.clone())
            .collect_vec()
    }

    pub fn expect_location(&self) -> Location {
        Location::new(
            self.get_latest_scope().funclet_id,
            self.get_latest_scope().node_id.unwrap(),
        )
    }

    pub fn get_latest_scope(&self) -> &ScheduleScopeData {
        self.scopes.last().unwrap()
    }

    pub fn get_latest_scope_mut(&mut self) -> &mut ScheduleScopeData {
        self.scopes.last_mut().unwrap()
    }

    pub fn add_explication_hole(&mut self) {
        self.get_latest_scope_mut().add_explication_hole()
    }

    pub fn get_current_funclet_id(&self) -> FuncletId {
        self.get_latest_scope().funclet_id
    }

    pub fn get_current_funclet<'a>(&self, context: &'a StaticContext) -> &'a expir::Funclet {
        &context.get_funclet(&self.get_latest_scope().funclet_id)
    }

    pub fn get_current_node_id(&self) -> Option<NodeId> {
        self.get_latest_scope().node_id
    }

    pub fn get_current_node<'a>(&self, context: &'a StaticContext) -> &'a Hole<expir::Node> {
        let scope = self.get_latest_scope();
        get_expect_box(
            &context.get_funclet(&scope.funclet_id).nodes,
            scope.node_id.unwrap(),
        )
    }

    pub fn get_funclet_spec_triple<'a>(
        &self,
        funclet: FuncletId,
        context: &'a StaticContext,
    ) -> (
        &'a expir::FuncletSpec,
        &'a expir::FuncletSpec,
        &'a expir::FuncletSpec,
    ) {
        match &context.get_funclet(&funclet).spec_binding {
            expir::FuncletSpecBinding::ScheduleExplicit {
                value,
                timeline,
                spatial,
            } => (value, timeline, spatial),
            _ => unreachable!(
                "{} is not a scheduling funclet",
                context.debug_info.funclet(&funclet)
            ),
        }
    }

    pub fn get_funclet_spec<'a>(
        &self,
        funclet: FuncletId,
        spec_kind: &SpecLanguage,
        context: &'a StaticContext,
    ) -> &'a expir::FuncletSpec {
        let results = self.get_funclet_spec_triple(funclet, context);
        match spec_kind {
            SpecLanguage::Value => results.0,
            SpecLanguage::Timeline => results.1,
            SpecLanguage::Spatial => results.2,
        }
    }

    pub fn get_triple_for_spec(
        &self,
        funclet: FuncletId,
        spec_kind: &SpecLanguage,
        quot: expir::Quotient,
        context: &StaticContext,
    ) -> LocationTriple {
        let results = self.get_funclet_spec_triple(funclet, context);
        fn build_location(spec: &expir::FuncletSpec, quot: expir::Quotient) -> Location {
            Location {
                funclet_id: spec.funclet_id_opt.unwrap(),
                quot,
            }
        }
        match spec_kind {
            SpecLanguage::Value => LocationTriple::new_value(build_location(&results.0, quot)),
            SpecLanguage::Timeline => {
                LocationTriple::new_timeline(build_location(&results.1, quot))
            }
            SpecLanguage::Spatial => LocationTriple::new_spatial(build_location(&results.2, quot)),
        }
    }

    pub fn get_spec_node<'a>(
        &self,
        funclet: FuncletId,
        node_id: NodeId,
        spec_kind: &SpecLanguage,
        context: &'a StaticContext,
    ) -> &'a expir::Node {
        let spec_funclet_id = self
            .get_funclet_spec(funclet, spec_kind, context)
            .funclet_id_opt
            .expect(&format!(
                "Spec {:?} for funclet {} missing id",
                spec_kind,
                context.debug_info.funclet(&funclet),
            ));
        get_expect_box(&context.get_funclet(&spec_funclet_id).nodes, node_id)
            .as_ref()
            .opt()
            .expect(&format!(
                "Spec funclet {} has an unexpected hole",
                context.debug_info.funclet(&funclet)
            ))
    }

    pub fn get_current_tail_edge<'a>(
        &self,
        context: &'a StaticContext,
    ) -> &'a Hole<expir::TailEdge> {
        &context
            .get_funclet(&self.get_latest_scope().funclet_id)
            .tail_edge
    }

    pub fn is_end_of_funclet<'a>(&self, context: &'a StaticContext) -> bool {
        let scope = self.get_latest_scope();
        scope.node_id.unwrap() >= context.get_funclet(&scope.funclet_id).nodes.len()
    }

    pub fn next_node(&mut self) {
        self.get_latest_scope_mut().next_node();
    }

    pub fn get_instantiations(
        &self,
        location: &Location,
        context: &StaticContext,
    ) -> HashSet<NodeId> {
        // Gets every instantiation of the location explicitly
        // Useful for update-related stuff

        self.get_latest_scope()
            .instantiations
            .iter()
            .filter(|(loc, _)| **loc == *location)
            .fold(HashSet::new(), |acc, (_, hs)| {
                acc.intersection(hs).cloned().collect()
            })
    }

    pub fn get_node_information(
        &self,
        node_id: &NodeId,
        context: &StaticContext,
    ) -> &StorageNodeInformation {
        self.get_latest_scope().get_node_information(node_id, context)
    }

    // Returns an ordered list of storage nodes in any scope (most to least recent)
    // The order is as follows (tiebreaks are in node index order)
    //   1. fully realized storage nodes without an instantiation
    //   2. unrealized storage nodes (those still pending explication)
    //   3. fully realized storage nodes with an existing instantiation
    pub fn find_all_storage_nodes(&self, target_type: &expir::Type, context: &StaticContext) -> Vec<Location> {
        let mut result = Vec::new();
        for scope in self.scopes.iter().rev() {
            let mut empty_nodes = Vec::new();
            let mut filled_nodes = Vec::new();
            // sort the results so we go top to bottom of the funclet
            for node_id in scope.storage_of_type(target_type, context).iter().sorted_by(|x, y| x.cmp(y))
            {
                // this is ok because we can just use the phi associated with an input
                let location = Location::new(scope.funclet_id.clone(), node_id.clone());
                match scope.get_node_information(node_id, context).instantiation {
                    Some(_) => filled_nodes.push(location),
                    None => empty_nodes.push(location),
                }
            }
            result.append(&mut empty_nodes);
            result.append(&mut filled_nodes);
        };
        result
    }

    // Returns an ordered list of storage nodes in any scope (most to least recent)
    // The order is as follows ()
    //   1. fully realized storage nodes without an instantiation
    //   
    pub fn find_matching_storage_nodes(
        &self,
        target_location_triple: &LocationTriple,
        target_type: &expir::Type,
        context: &StaticContext,
    ) -> Vec<Location> {
        let mut result = Vec::new();
        for scope in self.scopes.iter().rev() {
            // sort the results so we go top to bottom of the funclet
            for node in scope
                .match_triple(target_location_triple, context)
                .iter()
                .sorted_by(|x, y| x.cmp(y))
            {
                let node_info = scope.storage_node_information.get(&node).unwrap();
                if is_of_type(&node_info.typ, target_type) {
                    result.push(Location::new(scope.funclet_id.clone(), node.clone()));
                }
            }
        }
        result
        // let nodes = vec![
        //     expir::Node::AllocTemporary {
        //         buffer_flags,
        //         place: Some(target_place.clone()),
        //         storage_type: None,
        //     }
        // ];
        // self.pop_best_operation(&nodes)
    }

    // Pops and returns the best match for the given list of operations (if one exists)
    // Returns as an index to make recursion more clear
    // Finds the operation with the following preferences (higher numbers are tiebreakers):
    //   1. maximum matching heuristic value
    //   2. inner-most scope
    //   3. order of the list of `nodes`
    //   4. most recently added node
    // if no such operation exists, returns the most recent explication hole
    // if there is also no explication hole, panics
    pub fn pop_best_operation(&mut self, nodes: &Vec<&expir::Node>) -> Location {
        todo!("I have no idea what to do here right now");
        // struct HeuristicResults {
        //     pub opcode: OpCode,
        //     pub scope_index: usize,
        //     pub operation_index: usize,
        //     pub heuristic_value: usize,
        // }
        // let mut best_found: Option<HeuristicResults> = None;
        // // enumerate before reversing for later access
        // for (scope_index, scope) in self.scopes.iter().enumerate().rev() {
        //     for node in nodes {
        //         let opcode = OpCode::new(node);
        //         match scope.allocations.get(&opcode) {
        //             None => {}
        //             Some(operations) => {
        //                 // 0 --> index, 1 --> value
        //                 for (operation_index, comp_node) in operations.iter().enumerate() {
        //                     best_found =
        //                         match compare_ops(node, self.get_node(&scope.name, comp_node)) {
        //                             None => best_found,
        //                             // this is the "magic heuristic"
        //                             Some(heuristic_value) => {
        //                                 let new_found = Some(HeuristicResults {
        //                                     opcode: opcode.clone(),
        //                                     scope_index,
        //                                     operation_index,
        //                                     heuristic_value,
        //                                 });
        //                                 match best_found {
        //                                     None => new_found,
        //                                     Some(old_result) => {
        //                                         if heuristic_value > old_result.heuristic_value {
        //                                             new_found
        //                                         } else {
        //                                             Some(old_result)
        //                                         }
        //                                     }
        //                                 }
        //                             }
        //                         }
        //                 }
        //             }
        //         }
        //     }
        // }
        // match best_found {
        //     None => {}
        //     Some(result) => {
        //         let scope = &mut self.scopes[result.scope_index];
        //         return Location {
        //             funclet: scope.name.clone(),
        //             node: scope
        //                 .available_operations
        //                 .get_mut(&result.opcode)
        //                 .unwrap()
        //                 .remove(result.operation_index),
        //         };
        //     }
        // }
        // for scope in self.scopes.iter().rev() {
        //     match &scope.explication_hole {
        //         None => {}
        //         Some(hole) => {
        //             return Location {
        //                 funclet: scope.name.clone(),
        //                 node: hole.clone(),
        //             };
        //         }
        //     }
        // }
        // panic!("No resource found for any of {:?}", nodes);
    }
}

// extremely boring search algorithms for each operation
// since operations have a bunch of fields
// the most direct way to find the first available operation is just to search
//   for any that are either none or match the fields provided
// vectors are essentially unrolled as "arbitrary length fields"

macro_rules! match_op_args {
    ($arg1:ident $arg2:ident [$arg_type:ident] $n:ident) => {
        match_op_args!(@ $arg1 $arg2 true $n)
    };
    ($arg1:ident $arg2:ident $arg_type:ident $n:ident) => {
        match_op_args!(@ $arg1 $arg2 false $n)
    };
    (@ $arg1:ident $arg2:ident $nested:tt $n:ident) => {
        match ($arg1, $arg2) {
            // matching each arrangement
            // we want to check for more specific matches
            (Hole::Empty, Hole::Empty) => Some($n),
            (Hole::Filled(_), Hole::Empty) => None,
            (Hole::Empty, Hole::Filled(_)) => Some($n),
            (Hole::Filled(s1), Hole::Filled(s2)) => match_op_args!(@ $nested (s1, s2, $n))
        }
    };
    (@ false ($left:ident, $right:ident, $n:ident)) => {
        // this is where the constants for heuristics could be messed with
        if ($left == $right) {
            // TODO: +1 is arbitrary and untested
            Some($n+1)
        } else {
            None
        }
    };
    (@ true ($left:ident, $right:ident, $n:ident)) => {
        $left.iter().zip($right.iter())
            .fold(Some($n), |res, (val_one, val_two)| match res {
                None => None,
                Some(new_val) => {
                    // this is also where the constants for heuristics could be messed with
                    if val_one == val_two {
                        // TODO: +1 is arbitrary and untested
                        Some(new_val + 1)
                    } else {
                        None
                    }
                }
            })
    }
}

macro_rules! operation_iniatializations {
    ($($_lang:ident $name:ident ($($arg:ident : $arg_type:tt,)*) -> $_output:ident;)*) => {
        paste! {
            fn compare_ops(req_node: &expir::Node, target_node: &expir::Node) -> Option<usize> {
                match (req_node, target_node) {
                    $((expir::Node::$name { $($arg : [<$arg _one>],)* },
                    expir::Node::$name { $($arg : [<$arg _two>],)* }) => {
                        let mut matches = Some(0);
                        $(
                            matches = match matches {
                                None => None,
                                Some(n) => match_op_args!([<$arg _one>] [<$arg _two>] $arg_type n),
                            };
                        )*
                        matches
                    })*
                    _ => { unreachable!("Attempting to compare two nodes of different opcodes {:?} and {:?}", &req_node, &target_node)}
                }
            }
        }
    };
}

with_operations!(operation_iniatializations);
