use super::*;
use crate::explication::util::*;
use paste::paste;

impl InState {
    pub fn new(funclet: FuncletId) -> InState {
        let scopes = vec![ScheduleScopeData::new(funclet)];
        InState { scopes }
    }

    // most of these functions are mutable
    // note that we are expected to clone the instate for each recursion
    // this way we avoid _problems_

    pub fn enter_funclet(&mut self, funclet: FuncletId) {
        self.scopes.push(ScheduleScopeData::new(funclet));
    }
    pub fn exit_funclet(&mut self) -> bool {
        // returns if we have popped the lexpir element of the scope
        match self.scopes.pop() {
            None => panic!("Cannot leave a scope when there is no scope to leave"),
            Some(_) => {}
        }
        self.scopes.len() == 0
    }

    pub fn add_allocation(
        &mut self,
        schedule_node: NodeId,
        typ: expir::Type,
        context: &StaticContext,
    ) {
        let scope = self.get_latest_scope_mut();
        scope.add_allocation(schedule_node, typ, context);
    }

    // gets a list of the nodes of the available allocations in this scope (if there are any)
    pub fn available_allocations(&self, typ: TypeId, place: expir::Place) -> Vec<Location> {
        let compare_alloc = |alloc: &expir::Type| match alloc {
            expir::Type::Ref {
                storage_place,
                storage_type,
                buffer_flags,
            } => typ == storage_type.0 && place == storage_place.clone(),
            _ => unreachable!(),
        };
        for scope in self.scopes.iter().rev() {
            let matches: Vec<_> = scope
                .allocations
                .iter()
                .filter(|(_, alloc)| compare_alloc(alloc))
                .map(|v| Location {
                    funclet: scope.funclet,
                    node: v.0,
                })
                .collect();
            if matches.len() > 0 {
                return matches;
            }
        }
        vec![]
    }

    // Consume an allocation and return the type information
    pub fn consume_allocation(&mut self, location: Location) -> expir::Type {
        for scope in self.scopes.iter_mut().rev() {
            if scope.funclet == location.funclet {
                let mut result = None;
                let mut to_remove = None;
                for (index, allocation) in scope.allocations.iter().enumerate() {
                    if allocation.0 == location.node {
                        assert!(to_remove.is_none());
                        to_remove = Some(index);
                        result = Some(allocation.1.clone());
                    }
                }
                scope.allocations.remove(to_remove.expect(&format!(
                    "Missing allocation {:?} (was it already consumed?)",
                    location
                )));
                return result.unwrap();
            }
        }
        panic!("Missing scope for {:?}", location);
    }

    pub fn add_instantiation(
        &mut self,
        schedule_node: NodeId,
        spec_remotes: Vec<Location>,
        typ: expir::Type,
        context: &StaticContext,
    ) {
        let scope = self.get_latest_scope_mut();
        for spec_remote in &spec_remotes {
            scope.add_instantiation(
                spec_remote.clone(),
                typ.clone(),
                schedule_node.clone(),
                context,
            );
        }
    }

    pub fn expect_location(&self) -> Location {
        Location {
            funclet: self.get_latest_scope().funclet,
            node: self.get_latest_scope().node.unwrap(),
        }
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
        self.get_latest_scope().funclet
    }

    pub fn get_current_funclet<'a>(&self, context: &'a StaticContext) -> &'a expir::Funclet {
        &context.get_funclet(&self.get_latest_scope().funclet)
    }

    pub fn get_current_node_id(&self) -> Option<NodeId> {
        self.get_latest_scope().node
    }

    pub fn get_current_node<'a>(&self, context: &'a StaticContext) -> &'a Hole<expir::Node> {
        let scope = self.get_latest_scope();
        get_expect_box(
            &context.get_funclet(&scope.funclet).nodes,
            scope.node.unwrap(),
        )
    }

    pub fn get_funclet_spec<'a>(
        &self,
        funclet: FuncletId,
        spec_kind: &expir::FuncletKind,
        context: &'a StaticContext,
    ) -> &'a expir::FuncletSpec {
        match &context.get_funclet(&funclet).spec_binding {
            expir::FuncletSpecBinding::ScheduleExplicit {
                value,
                spatial,
                timeline,
            } => match spec_kind {
                ir::FuncletKind::Value => value,
                ir::FuncletKind::Timeline => timeline,
                ir::FuncletKind::Spatial => spatial,
                error_kind => {
                    unreachable!("Invalid kind for spec node lookup: {:?}", error_kind)
                }
            },
            _ => unreachable!(
                "{} is not a scheduling funclet",
                context.debug_info.funclet(&funclet)
            ),
        }
    }

    pub fn get_spec_node<'a>(
        &self,
        funclet: FuncletId,
        node_id: NodeId,
        spec_kind: &expir::FuncletKind,
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
            .get_funclet(&self.get_latest_scope().funclet)
            .tail_edge
    }

    pub fn is_end_of_funclet<'a>(&self, context: &'a StaticContext) -> bool {
        let scope = self.get_latest_scope();
        scope.node.unwrap() >= context.get_funclet(&scope.funclet).nodes.len()
    }

    pub fn next_node(&mut self) {
        self.get_latest_scope_mut().next_node();
    }

    pub fn get_node_instantiation(&self, node_id: NodeId, context: &StaticContext) -> Location {
        let scope = self.get_latest_scope();
        scope
            .node_type_information
            .get(&node_id)
            .expect(&format!(
                "Missing instantiation for node {}",
                context.debug_info.node(&scope.funclet, node_id)
            ))
            .0
            .clone()
    }

    // Returns true if two types are "close enough" to equal
    // TODO: refine this condition
    fn compare_types(&self, type1: &expir::Type, type2: &expir::Type) -> bool {
        match (type1, type2) {
            (
                ir::Type::NativeValue {
                    storage_type: storage_type1,
                },
                ir::Type::NativeValue {
                    storage_type: storage_type2,
                },
            ) => true,
            (
                ir::Type::Ref {
                    storage_type: storage_type1,
                    storage_place: storage_place1,
                    buffer_flags: buffer_flags1,
                },
                ir::Type::Ref {
                    storage_type: storage_type2,
                    storage_place: storage_place2,
                    buffer_flags: buffer_flags2,
                },
            ) => true,
            (
                ir::Type::Fence {
                    queue_place: queue_place1,
                },
                ir::Type::Fence {
                    queue_place: queue_place2,
                },
            ) => true,
            (
                ir::Type::Buffer {
                    storage_place: storage_place1,
                    static_layout_opt: static_layout_opt1,
                    flags: flags1,
                },
                ir::Type::Buffer {
                    storage_place: storage_place2,
                    static_layout_opt: static_layout_opt2,
                    flags: flags2,
                },
            ) => true,
            (
                ir::Type::Encoder {
                    queue_place: queue_place1,
                },
                ir::Type::Encoder {
                    queue_place: queue_place2,
                },
            ) => true,
            (ir::Type::Event, ir::Type::Event) => true,
            (ir::Type::BufferSpace, ir::Type::BufferSpace) => true,
            _ => false,
        }
    }

    // Returns an instantiation if one is available in any scope (most to leexpir recent)
    // if there is no instantiation already made for the given funclet/node
    //   pops the best available instantiation
    //   panics in this case if there is none that can fulfill the requirements
    pub fn find_instantiation(
        &self,
        target_location: &Location,
        target_type: &expir::Type,
        context: &StaticContext,
    ) -> Location {
        for scope in self.scopes.iter().rev() {
            match scope.instantiations.get(&target_location) {
                None => {}
                Some(instantiations) => {
                    for node in instantiations {
                        let node_info = scope.node_type_information.get(&node).unwrap();
                        if self.compare_types(&node_info.1, target_type) {
                            return Location {
                                funclet: scope.funclet.clone(),
                                node: node.clone(),
                            };
                        }
                    }
                }
            }
        }
        todo!()
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
