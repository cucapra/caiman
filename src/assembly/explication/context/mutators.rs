use super::*;
use crate::assembly::explication::util::*;
use paste::paste;
use std::collections::hash_map::Entry;
use std::fmt::Debug;

impl<'context> Context<'context> {
    pub fn update_node(&mut self, node: NodeId) {
        self.location.node = Some(node);
    }

    pub fn enter_funclet(&mut self, funclet: FuncletId) {
        // updates the location and the scope vec
        self.location.funclet = Some(funclet.clone());
        let scope = ScheduleScopeData {
            name: funclet,
            instantiations: Default::default(),
            available_operations: Default::default(),
            explication_hole: None,
        };
        self.scopes.push(scope);
    }

    pub fn end_current_funclet(&mut self) -> bool {
        // returns if we have popped the last element of the scope
        match self.scopes.pop() {
            None => panic!("Cannot leave a scope when there is no scope to leave"),
            Some(funclet) => {
                self.location.funclet = Some(funclet.name.clone());
            }
        }
        self.scopes.len() == 0
    }

    pub fn add_instantiation(
        &mut self,
        schedule_node: NodeId,
        spec_funclet: FuncletId,
        spec_node: NodeId,
        place: ir::Place,
        is_value: bool,
    ) {
        self.get_latest_scope().add_instantiation(
            schedule_node,
            ScheduledInstantiationInfo {
                funclet: spec_funclet,
                node: spec_node,
                place,
                is_value,
            },
        )
    }

    pub fn add_available_operation(&mut self, schedule_node: NodeId, operation: OpCode) {
        self.get_latest_scope()
            .add_operation(schedule_node, operation)
    }

    pub fn add_explication_hole(&mut self, node: NodeId) {
        self.get_latest_scope().add_explication_hole(node)
    }

    pub fn pop_best_match(&mut self, node: &ast::Node) -> RemoteNodeId {
        let opcode = OpCode::new(node);
        for scope in self.scopes.iter_mut().rev() {
            // the premise here is to look less-to-more specific (as given by infos order)
            // then if nothing is found, return an explication hole
            // finally, if that doesn't work, go up the stack
            match scope.available_operations.get_mut(&opcode) {
                None => {}
                Some(operations) => {
                    // 0 --> index, 1 --> value
                    let mut best_found = None;
                    for (index, comp_node) in operations.iter().enumerate() {
                        best_found = match compare_ops(node, self.get_node(&scope.name, comp_node))
                        {
                            None => best_found,
                            // this is the "magic heuristic"
                            Some(new_value) => match best_found {
                                None => Some((index, new_value)),
                                Some((_, old_value)) => {
                                    if new_value > old_value {
                                        Some((index, new_value))
                                    } else {
                                        best_found
                                    }
                                }
                            },
                        }
                    }
                    match best_found {
                        None => {}
                        Some((index, value)) => {
                            return RemoteNodeId {
                                funclet: Some(scope.name.clone()),
                                node: Some(operations.remove(index))
                            };
                        }
                    }
                }
            }
            match &scope.explication_hole {
                None => {}
                Some(node) => {
                    return RemoteNodeId {
                        funclet: Some(scope.name.clone()),
                        node: Some(node.clone())
                    };
                }
            }
        }
        panic!("No available resource for resolving {:?} found", node);
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
