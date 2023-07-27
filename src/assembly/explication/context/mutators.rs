use super::*;
use crate::assembly::explication::util::*;
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
        self.get_latest_scope().add_instantiation(schedule_node, ScheduledInstantiationInfo {
            funclet: spec_funclet,
            node: spec_node,
            place,
            is_value,
        })
    }

    pub fn add_available_allocation(
        &mut self,
        node: NodeId,
        ffi_type: Hole<FFIType>,
        place: Hole<ir::Place>
    ) {
        self.get_latest_scope().add_allocation(node, AlloctionHoleInfo {
            ffi_type,
            place,
        })
    }

    pub fn add_available_read (
        &mut self,
        schedule_node: NodeId,
        spec_funclet: FuncletId,
        spec_node: NodeId,
    ) {
        self.get_latest_scope().add_operation(schedule_node, OperationInfo {
            node: spec_node,
            funclet: spec_funclet,
            operation: Opcode::Read,
        })
    }

    pub fn add_available_write (
        &mut self,
        schedule_node: NodeId,
        spec_funclet: FuncletId,
        spec_node: NodeId,
    ) {
        self.get_latest_scope().add_operation(schedule_node, OperationInfo {
            node: spec_node,
            funclet: spec_funclet,
            operation: Opcode::Write,
        })
    }

    pub fn add_available_copy (
        &mut self,
        schedule_node: NodeId,
        spec_funclet: FuncletId,
        spec_node: NodeId,
    ) {
        self.get_latest_scope().add_operation(schedule_node, OperationInfo {
            node: spec_node,
            funclet: spec_funclet,
            operation: Opcode::Copy,
        })
    }

    pub fn add_explication_hole(
        &mut self,
        node: NodeId,
    ) {
        self.get_latest_scope().add_explication_hole(node)
    }

    fn pop_scoped<T, U, V>(&mut self, info: T, map: U) -> V
    where
        T: std::hash::Hash + PartialEq + Eq + Debug,
        U: Fn(&mut ScheduleScopeData) -> &mut HashMap<T, Vec<V>>,
    {
        for scope in self.scopes.iter_mut().rev() {
            match map(scope).get_mut(&info) {
                None => {}
                Some(mut v) => {
                    if v.len() > 0 {
                        return v.pop().unwrap();
                    }
                }
            }
        }
        panic!("No available resource for {:?} found", info);
    }

    pub fn pop_available_allocation(
        &mut self,
        ffi_type: Hole<FFIType>,
        place: Hole<ir::Place>,
    ) -> NodeId {
        let info = AlloctionHoleInfo { ffi_type, place };
        self.pop_scoped(info, |mut s| &mut s.available_allocations)
    }

    pub fn pop_available_write(&mut self, funclet: FuncletId, node: NodeId) -> NodeId {
        let info = OperationInfo {
            funclet,
            node,
            operation: Opcode::Write,
        };
        self.pop_scoped(info, |mut s| &mut s.available_operations)
    }

    pub fn pop_available_read(&mut self, funclet: FuncletId, node: NodeId) -> NodeId {
        let info = OperationInfo {
            funclet,
            node,
            operation: Opcode::Read,
        };
        self.pop_scoped(info, |mut s| &mut s.available_operations)
    }

    pub fn pop_available_copy(&mut self, funclet: FuncletId, node: NodeId) -> NodeId {
        let info = OperationInfo {
            funclet,
            node,
            operation: Opcode::Copy,
        };
        self.pop_scoped(info, |mut s| &mut s.available_operations)
    }
}
