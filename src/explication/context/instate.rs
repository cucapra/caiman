use super::*;
use crate::assembly::explication::util::*;

impl InState {
    pub fn new(funclet: FuncletId) -> InState {
        let scopes = vec![ScheduleScopeData::new(funclet)];
        InState {
            schedule_explication_data: HashMap::new(),
            scopes,
        }
    }

    // most of these functions are mutable
    // note that we are expected to clone the instate for each recursion
    // this way we avoid _problems_

    pub fn enter_funclet(&mut self, funclet: FuncletId) {
        let instantiations = self.scopes.last().cloned().map(|le| le.instantiations).unwrap_or(HashMap::default());
        let allocations = self.scopes.last().cloned().map(|le| le.allocations).unwrap_or(HashMap::default());
        self.scopes.push(ScheduleScopeData::new(funclet));
    }
    pub fn exit_funclet(&mut self) -> bool {
        // returns if we have popped the last element of the scope
        match self.scopes.pop() {
            None => panic!("Cannot leave a scope when there is no scope to leave"),
            Some(_) => {}
        }
        self.scopes.len() == 0
    }
    pub fn add_instantiation(
        &mut self,
        schedule_node: NodeId,
        spec_remotes: Vec<RemoteNodeId>,
        place: ir::Place,
    ) {
        let scope = self.get_latest_scope();
        for spec_remote in &spec_remotes {
            scope.add_instantiation(
                schedule_node.clone(),
                Location {
                    funclet: spec_remote.funclet.as_ref().unwrap().clone(),
                    node: spec_remote.node.as_ref().unwrap().clone(),
                },
                place
            );
        }
        let name = scope.funclet.clone();
        let explication_data = self.schedule_explication_data.get_mut(&name).unwrap();
        // let instantiated = InstantiatedNodes::new(&explication_data.type_instantiations, spec_remotes);
        // explication_data
        //     .type_instantiations
        //     .insert(schedule_node, instantiated);
    }

    pub fn get_latest_scope(&self) -> &ScheduleScopeData {
        self.scopes.last().unwrap()
    }

    pub fn get_latest_scope_mut(&mut self) -> &mut ScheduleScopeData {
        &mut self.scopes.last().unwrap()
    }

    pub fn add_available_operation(&mut self, schedule_node: NodeId, operation: OpCode) {
        self.get_latest_scope_mut().add_operation(schedule_node, operation)
    }

    pub fn add_explication_hole(&mut self, node: NodeId) {
        self.get_latest_scope_mut().add_explication_hole(node)
    }

    // Returns an instantiation if one is available in any scope (most to least recent)
    // if there is no instantiation already made for the given funclet/node
    //   pops the best available instantiation
    //   panics in this case if there is none that can fulfill the requirements
    pub fn get_instantiation(
        &mut self,
        target_location: &Location,
        target_place: &ir::Place,
    ) -> Location {
        for scope in self.scopes.iter().rev() {
            match scope.instantiations.get(&target_location) {
                None => {}
                Some(instantiations) => {
                    for (inst_place, node) in instantiations {
                        if inst_place == target_place {
                            return Location {
                                funclet: scope.name.clone(),
                                node: node.clone(),
                            }
                        }
                    }
                }
            }
        };
        let nodes = vec![
            &ast::Node::AllocTemporary {
                place: Some(target_place.clone()),
                storage_type: None,
            }
        ];
        self.pop_best_operation(&nodes)
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
    pub fn pop_best_operation(&mut self, nodes: &Vec<&ast::Node>) -> usize {
        struct HeuristicResults {
            pub opcode: OpCode,
            pub scope_index: usize,
            pub operation_index: usize,
            pub heuristic_value: usize,
        }
        let mut best_found: Option<HeuristicResults> = None;
        // enumerate before reversing for later access
        for (scope_index, scope) in self.scopes.iter().enumerate().rev() {
            for node in nodes {
                let opcode = OpCode::new(node);
                match scope.available_operations.get(&opcode) {
                    None => {}
                    Some(operations) => {
                        // 0 --> index, 1 --> value
                        for (operation_index, comp_node) in operations.iter().enumerate() {
                            best_found =
                                match compare_ops(node, self.get_node(&scope.name, comp_node)) {
                                    None => best_found,
                                    // this is the "magic heuristic"
                                    Some(heuristic_value) => {
                                        let new_found = Some(HeuristicResults {
                                            opcode: opcode.clone(),
                                            scope_index,
                                            operation_index,
                                            heuristic_value,
                                        });
                                        match best_found {
                                            None => new_found,
                                            Some(old_result) => {
                                                if heuristic_value > old_result.heuristic_value {
                                                    new_found
                                                } else {
                                                    Some(old_result)
                                                }
                                            }
                                        }
                                    }
                                }
                        }
                    }
                }
            }
        }
        match best_found {
            None => {}
            Some(result) => {
                let scope = &mut self.scopes[result.scope_index];
                return Location {
                    funclet: scope.name.clone(),
                    node: scope
                        .available_operations
                        .get_mut(&result.opcode)
                        .unwrap()
                        .remove(result.operation_index),
                };
            }
        }
        for scope in self.scopes.iter().rev() {
            match &scope.explication_hole {
                None => {}
                Some(hole) => {
                    return Location {
                        funclet: scope.name.clone(),
                        node: hole.clone(),
                    };
                }
            }
        }
        panic!("No resource found for any of {:?}", nodes);
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
            (None, None) => Some($n),
            (Some(_), None) => None,
            (None, Some(_)) => Some($n),
            (Some(s1), Some(s2)) => match_op_args!(@ $nested (s1, s2, $n))
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
            fn compare_ops(req_node: &ast::Node, target_node: &ast::Node) -> Option<usize> {
                match (req_node, target_node) {
                    $((ast::Node::$name { $($arg : [<$arg _one>],)* },
                    ast::Node::$name { $($arg : [<$arg _two>],)* }) => {
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