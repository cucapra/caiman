use super::*;
use crate::explication::util;
use crate::explication::util::{Location, LocationTriple};
use itertools::Itertools;
use paste::paste;

impl InState {
    // setup stuff

    pub fn new_operation(funclet_id: FuncletId, context: &StaticContext) -> InState {
        let scopes = vec![ScheduleScopeData::new_operation(funclet_id)];
        InState { scopes }
    }

    pub fn new_storage(funclet_id: FuncletId, context: &StaticContext) -> InState {
        let scopes = vec![ScheduleScopeData::new_storage(funclet_id)];
        InState { scopes }
    }

    // most of these functions are mutable
    // note that we are expected to clone the instate for each recursion
    // this way we avoid _problems_

    pub fn enter_funclet(&mut self, funclet_id: FuncletId, context: &StaticContext) {
        let new_scope = match self.get_latest_scope().pass_information {
            PassInformation::Operation(_) => ScheduleScopeData::new_operation(funclet_id),
            PassInformation::Storage(_) => ScheduleScopeData::new_storage(funclet_id),
        };
        self.scopes.push(new_scope);
    }
    pub fn exit_funclet(&mut self) -> bool {
        // returns if we have popped the lexpir element of the scope
        match self.scopes.pop() {
            None => panic!("Cannot leave a scope when there is no scope to leave"),
            Some(_) => {}
        }
        self.scopes.len() == 0
    }

    // general utility

    pub fn hole_error(&self, context: &StaticContext) -> String {
        format!(
            "TODO Hole in funclet {} for node {}",
            context
                .debug_info
                .funclet(&self.get_latest_scope().funclet_id),
            context.debug_info.node_expir(
                self.get_current_funclet_id(),
                self.get_current_node(context)
                    .as_ref()
                    .opt()
                    .expect("Unreachable")
            )
        )
    }

    pub fn node_error(&self, node_id: NodeId, context: &StaticContext) -> String {
        format!(
            "for node {}",
            context
                .debug_info
                .node(&self.get_current_funclet_id(), node_id)
        )
    }

    pub fn current_node_error(&self, context: &StaticContext) -> String {
        self.node_error(self.get_current_node_id().unwrap(), context)
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

    // given a typeid corresponding to a native type
    // returns the storage_type of that native type
    pub fn expect_native_storage_type(
        &self,
        type_id: &expir::TypeId,
        context: &StaticContext,
    ) -> ffi::TypeId {
        match context.get_type(type_id) {
            ir::Type::NativeValue { storage_type } => storage_type.clone(),
            typ => panic!(
                "Expected a native type, got {:?} while checking {}",
                typ,
                context.debug_info.node(
                    &self.get_current_funclet_id(),
                    self.get_current_node_id().unwrap()
                )
            ),
        }
    }

    pub fn is_end_of_funclet<'a>(&self, context: &'a StaticContext) -> bool {
        let scope = self.get_latest_scope();
        scope.node_id.unwrap() >= context.get_funclet(&scope.funclet_id).nodes.len()
    }

    pub fn next_node(&mut self) {
        self.get_latest_scope_mut().next_node();
    }

    // operation stuff

    pub fn add_value_operation(&mut self, operation: Location, context: &StaticContext) {
        self.get_latest_scope_mut()
            .add_value_operation(operation, context);
    }

    pub fn has_value_operation(&self, operation: &Location, context: &StaticContext) {
        self.get_latest_scope()
            .has_value_operation(operation, context);
    }

    pub fn add_timeline_operation(&mut self, operation: Location, context: &StaticContext) {
        self.get_latest_scope_mut()
            .add_timeline_operation(operation, context);
    }

    pub fn has_timeline_operation(&self, operation: &Location, context: &StaticContext) {
        self.get_latest_scope()
            .has_timeline_operation(operation, context);
    }

    // returns all operations who's depedencies have already been satisfied
    pub fn find_satisfied_operations(
        &self,
        value_funclet_id: &FuncletId,
        context: &StaticContext,
    ) -> Vec<expir::Quotient> {
        let value_funclet = context.get_funclet(value_funclet_id);
        let mut result = Vec::new();
        let scope = self.get_latest_scope();
        for node_id in 0..value_funclet.nodes.len() {
            if !scope.has_value_operation(
                &Location::new(value_funclet_id.clone(), node_id.clone()),
                context,
            ) {
                if context
                    .get_node_dependencies(value_funclet_id, &node_id)
                    .iter()
                    .map(|dependency| {
                        scope.has_value_operation(
                            &Location::new(value_funclet_id.clone(), dependency.clone()),
                            context,
                        )
                    })
                    .all(|x| x)
                {
                    result.push(expir::Quotient::Node { node_id });
                }
            }
        }
        result
    }

    // storage stuff

    pub fn add_storage_node(
        &mut self,
        schedule_node: NodeId,
        typ: Hole<expir::Type>,
        context: &StaticContext,
    ) {
        self.get_latest_scope_mut()
            .add_storage_node(schedule_node, typ, context);
    }

    pub fn set_storage_type(
        &mut self,
        schedule_node: NodeId,
        typ: expir::Type,
        context: &StaticContext,
    ) {
        self.get_latest_scope_mut()
            .set_storage_type(schedule_node, typ, context);
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

    pub fn get_node_information(
        &self,
        node_id: &NodeId,
        context: &StaticContext,
    ) -> &StorageNodeInformation {
        self.get_latest_scope()
            .get_node_information(node_id, context)
    }

    pub fn get_nodes_with_timeline_status(
        &self,
        timeline_status: &expir::Quotient,
        context: &StaticContext,
    ) -> Vec<NodeId> {
        self.get_latest_scope()
            .as_storage()
            .storage_node_information
            .iter()
            .filter(|(_, info)| {
                info.instantiation
                    .as_ref()
                    .map(|inst| {
                        inst.timeline
                            .as_ref()
                            .map(|loc| loc.quot == *timeline_status)
                            .unwrap_or(false)
                    })
                    .unwrap_or(false)
            })
            .map(|v| v.0.clone())
            .collect_vec()
    }

    pub fn get_instantiations(
        &self,
        location: &Location,
        context: &StaticContext,
    ) -> HashSet<NodeId> {
        // Gets every instantiation of the location explicitly
        // Useful for update-related stuff

        self.get_latest_scope()
            .as_storage()
            .instantiations
            .iter()
            .filter(|(loc, _)| **loc == *location)
            .fold(HashSet::new(), |acc, (_, hs)| {
                acc.intersection(hs).cloned().collect()
            })
    }

    // Returns an ordered list of all storage nodes in any scope (most to least recent)
    // The order is as follows (tiebreaks are in node index order)
    //   1. fully realized storage nodes without an instantiation
    //   2. unrealized storage nodes (those still pending explication)
    //   3. fully realized storage nodes with an existing instantiation
    //   4. recurse from 1 up the stack
    pub fn find_all_storage_nodes(
        &self,
        target_type: &expir::Type,
        context: &StaticContext,
    ) -> Vec<Location> {
        let mut result = Vec::new();

        // keep track of each "kind" of result
        let mut empty_nodes = Vec::new();
        let mut unrealized_nodes = Vec::new();
        let mut filled_nodes = Vec::new();

        for scope in self.scopes.iter().rev() {
            // sort the results so we go top to bottom of the funclet
            for node_id in scope.storage_of_type(target_type, context).iter().sorted() {
                // this is ok because we can just use the phi associated with an input
                let location = Location::new(scope.funclet_id.clone(), node_id.clone());
                let info = scope.get_node_information(node_id, context);
                match info.instantiation {
                    Some(_) => match info.typ {
                        Hole::Empty => unrealized_nodes.push(location),
                        Hole::Filled(_) => filled_nodes.push(location),
                    },
                    None => empty_nodes.push(location),
                }
            }
            result.append(&mut empty_nodes);
            result.append(&mut unrealized_nodes);
            result.append(&mut filled_nodes);
        }
        result
    }

    // Returns an ordered list of storage nodes in any scope (most to least recent)
    // The order is as follows:
    //   1. fully realized storage nodes with a matching instantiation
    //   2. unrealized storage nodes (those still pending explication)
    //   3. recurse from 1 up the stack
    pub fn find_matching_instantiations(
        &self,
        mut target_location_triple: LocationTriple,
        target_type: &expir::Type,
        context: &StaticContext,
    ) -> Vec<Location> {
        target_location_triple = target_location_triple.into_node_id(context);
        self.instantiation_search(Some(target_location_triple), target_type, context)
    }

    // Does the same as the above, but with no "matching instantiaion" requirement
    pub fn find_all_instantiations(
        &self,
        target_type: &expir::Type,
        context: &StaticContext,
    ) -> Vec<Location> {
        self.instantiation_search(None, target_type, context)
    }

    fn instantiation_search(
        &self,
        target_location_triple: Option<LocationTriple>,
        target_type: &expir::Type,
        context: &StaticContext,
    ) -> Vec<Location> {
        let mut end_result = Vec::new();
        let mut result = Vec::new();
        for scope in self.scopes.iter().rev() {
            // sort the results so we go top to bottom of the funclet
            let nodes = match target_location_triple {
                None => scope.all_instantiations(context),
                Some(ref triple) => scope.match_triple(&triple, context),
            };
            for node in nodes.iter().sorted() {
                let node_info = scope
                    .as_storage()
                    .storage_node_information
                    .get(&node)
                    .unwrap();
                let location = Location::new(scope.funclet_id.clone(), node.clone());
                match &node_info.typ {
                    Hole::Empty => end_result.push(location),
                    Hole::Filled(typ) => {
                        if is_of_type(&typ, target_type) {
                            result.push(location);
                        }
                    }
                }
            }
            result.append(&mut end_result);
        }
        result
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
