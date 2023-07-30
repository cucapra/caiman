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
            available_allocations: Default::default(),
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

    pub fn add_available_allocation(
        &mut self,
        node: NodeId,
        ffi_type: Hole<FFIType>,
        place: Hole<ir::Place>,
    ) {
        self.get_latest_scope()
            .add_allocation(node, AlloctionHoleInfo { ffi_type, place })
    }

    pub fn add_available_operation(&mut self, schedule_node: NodeId, operation: OpCode) {
        self.get_latest_scope()
            .add_operation(schedule_node, operation)
    }

    pub fn add_explication_hole(&mut self, node: NodeId) {
        self.get_latest_scope().add_explication_hole(node)
    }

    fn pop_scoped<T, U>(&mut self, infos: Vec<T>, map: U) -> NodeId
    where
        T: std::hash::Hash + PartialEq + Eq + Debug,
        U: Fn(&mut ScheduleScopeData) -> &mut HashMap<T, Vec<NodeId>>,
    {
        for scope in self.scopes.iter_mut().rev() {
            // the premise here is to look less-to-more specific (as given by infos order)
            // then if nothing is found, return an explication hole
            // finally, if that doesn't work, go up the stack
            let data = map(scope);
            for info in &infos {
                match data.get_mut(info) {
                    None => {}
                    Some(mut v) => {
                        if v.len() > 0 {
                            return v.pop().unwrap();
                        }
                    }
                }
            }
            match &scope.explication_hole {
                None => {}
                Some(node) => {
                    return node.clone();
                }
            }
        }
        panic!("No available resource for {:?} found", infos.first());
    }
}

// extremely boring search algorithms for each operation
// since operations have a bunch of fields
// the most direct way to find the first available operation is just to search
//   for any that are either none or match the fields provided
// vectors are essentially unrolled as "arbitrary length fields"

macro_rules! match_op_args {
    ($arg1:ident $arg2:ident [$arg_type:ident]) => {
        match_op_args!(@ $arg1 $arg2 true)
    };
    ($arg1:ident $arg2:ident $arg_type:ident) => {
        match_op_args!(@ $arg1 $arg2 false)
    };
    (@ $arg1:ident $arg2:ident $nested:tt) => {
        match ($arg1, $arg2) {
            // matching each arrangement
            // we want to check for more specific matches
            (None, None) => { true },
            (Some(_), None) => { false }
            (None, Some(_)) => { true }
            (Some(s1), Some(s2)) => { match_op_args!(@ $nested (s1, s2)) }
        }
    };
    (@ false ($left:ident, $right:ident)) => {
        $left == $right
    };
    (@ true ($left:ident, $right:ident)) => {
        $left.iter().zip($right.iter())
            .fold(true, |res, (val_one, val_two)| res &&
                match_op_args!(@ val_one val_two false)
        )
    }
}

macro_rules! operation_iniatializations {
    ($($_lang:ident $name:ident ($($arg:ident : $arg_type:tt,)*) -> $_output:ident;)*) => {
        paste! {
            fn compare_ops(req_node: &ast::Node, target_node: &ast::Node) -> bool {
                match (req_node, target_node) {
                    $((ast::Node::$name { $($arg : [<$arg _one>],)* },
                    ast::Node::$name { $($arg : [<$arg _two>],)* }) => {
                        let mut valid_node = true;
                        $(
                            valid_node = valid_node && match_op_args!([<$arg _one>] [<$arg _two>] $arg_type);
                        )*
                        valid_node
                    })*
                    _ => { unreachable!("Attempting to compare two nodes of different opcodes {:?} and {:?}", &req_node, &target_node)}
                }
            }
        }
    };
}

with_operations!(operation_iniatializations);