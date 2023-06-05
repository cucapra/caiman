use crate::assembly::ast;
use crate::assembly::ast::Hole;
use crate::assembly::ast::{
    ExternalFunctionId, FFIType, FuncletId, FunctionClassId, NodeId, RemoteNodeId, StorageTypeId,
    TypeId,
};
use crate::ir;
use debug_ignore::DebugIgnore;
use std::collections::{HashMap, HashSet};
use crate::assembly::table::Table;

pub struct Context {
    pub location: LocationNames,

    // information found about a given value funclet
    value_explication_data: HashMap<FuncletId, ValueFuncletData>,
    // information found about a given schedule funclet
    schedule_explication_data: HashMap<FuncletId, ScheduleFuncletData>,

    meta_data: MetaData,
}

#[derive(Debug)]
pub struct LocationNames {
    // a bit confusing, but unwrapping holes is annoying
    pub funclet_name: FuncletId,
    pub node_name: NodeId,
}

#[derive(Debug)]
pub struct ValueFuncletConnections {
    // stores connections of what refers to this value funclet
    pub schedule_funclets: Vec<FuncletId>,
    pub timeline_funclets: Vec<FuncletId>,
    pub spatial_funclets: Vec<FuncletId>,
}

#[derive(Debug)]
struct ValueExplicationInformation {
    // explication locations are in the scheduling world
    // maps from this node to the places it's been allocated on
    scheduled_allocations: HashMap<FuncletId, NodeId>,

    // indicates whether this operation has been written yet
    // used primarily to add operations when needed
    written: bool,
}

#[derive(Debug)]
struct ValueFuncletData {
    pub connections: ValueFuncletConnections,

    // information about allocated value elements
    explication_information: HashMap<NodeId, ValueExplicationInformation>,

    // map from call index to output name for each call
    call_outputs: HashMap<NodeId, HashMap<usize, NodeId>>,
}

#[derive(Debug)]
struct ScheduleFuncletData {
    // associated value funclet
    value_funclet: FuncletId,
    // map from the variables available to which node they are allocated
    allocations: HashMap<NodeId, NodeId>,
    // list of explication holes found, by index
    explication_holes: Vec<usize>,
}

#[derive(Debug)]
struct MetaData {
    name_index: usize,
}

impl LocationNames {
    pub fn new() -> LocationNames {
        LocationNames {
            funclet_name: FuncletId("".to_string()),
            node_name: NodeId("".to_string()),
        }
    }
}

impl ValueFuncletConnections {
    pub fn new() -> ValueFuncletConnections {
        ValueFuncletConnections {
            schedule_funclets: Vec::new(),
            timeline_funclets: Vec::new(),
            spatial_funclets: Vec::new(),
        }
    }
}

impl ValueExplicationInformation {
    pub fn new() -> ValueExplicationInformation {
        ValueExplicationInformation {
            scheduled_allocations: HashMap::new(),
            written: false,
        }
    }
}

impl ValueFuncletData {
    pub fn new() -> ValueFuncletData {
        ValueFuncletData {
            connections: ValueFuncletConnections::new(),
            explication_information: HashMap::new(),
            call_outputs: HashMap::new(),
        }
    }
    pub fn allocate(
        &mut self,
        value_node: NodeId,
        schedule_funclet: FuncletId,
        schedule_node: NodeId,
    ) {
        self.explication_information
            .entry(value_node)
            .or_insert(ValueExplicationInformation::new())
            .scheduled_allocations
            .insert(schedule_funclet, schedule_node);
    }
}

impl ScheduleFuncletData {
    pub fn new(value_funclet: FuncletId) -> ScheduleFuncletData {
        ScheduleFuncletData {
            value_funclet,
            allocations: HashMap::new(),
            explication_holes: Vec::new(),
        }
    }
    pub fn allocate(&mut self, schedule_node: NodeId, value_node: NodeId) {
        self.allocations.insert(schedule_node, value_node);
    }
}

impl MetaData {
    pub fn new(name_index: usize) -> MetaData {
        MetaData { name_index }
    }
    pub fn next_name(&mut self) -> String {
        self.name_index += 1;
        format!("~{}", self.name_index)
    }
}

impl Context {
    pub fn new(program : &ast::Program, name_index : usize) -> Context {
        Context {
            location: LocationNames::new(),
            value_explication_data: HashMap::new(),
            schedule_explication_data: HashMap::new(),
            meta_data: MetaData::new(name_index),
        }
    }

    pub fn setup_schedule_data(&mut self, value_funclet: FuncletId) {
        self.value_explication_data
            .get_mut(&value_funclet)
            .unwrap()
            .connections
            .schedule_funclets
            .push(self.location.funclet_name.clone());
        self.schedule_explication_data.insert(
            self.location.funclet_name.clone(),
            ScheduleFuncletData::new(value_funclet),
        );
    }

    // get allocations of the associated value node
    pub fn get_schedule_allocations(
        &self,
        funclet: &FuncletId,
        node: &NodeId,
    ) -> Option<&HashMap<FuncletId, NodeId>> {
        self.value_explication_data.get(funclet).and_then(|f| {
            f.explication_information
                .get(node)
                .map(|n| &n.scheduled_allocations)
        })
    }

    pub fn get_current_schedule_allocation(&self, node: &NodeId) -> Option<&NodeId> {
        self.get_current_value_funclet().as_ref().and_then(|vf| {
            self.get_schedule_allocations(&vf, node)
                .unwrap()
                .get(&self.location.funclet_name)
        })
    }

    // get what the associated schedule node is allocating
    pub fn get_value_allocation(&self, funclet: &FuncletId, node: &NodeId) -> Option<&ast::NodeId> {
        self.schedule_explication_data
            .get(funclet)
            .and_then(|f| f.allocations.get(node))
    }

    fn allocation_name(&mut self) -> String {
        self.meta_data.name_index += 1;
        format!("${}", self.meta_data.name_index)
    }

    pub fn add_allocation(&mut self, value_node: NodeId, schedule_node: NodeId) {
        let schedule_funclet = self.location.funclet_name.clone();
        let value_funclet = &self
            .schedule_explication_data
            .get(&schedule_funclet)
            .unwrap()
            .value_funclet;

        // unwrap explicitly cause we assume funclet data are setup earlier
        self.value_explication_data
            .get_mut(value_funclet)
            .unwrap()
            .allocate(
                value_node.clone(),
                schedule_funclet.clone(),
                schedule_node.clone(),
            );

        self.schedule_explication_data
            .get_mut(&schedule_funclet)
            .unwrap()
            .allocate(schedule_node, value_node);
    }

    pub fn get_current_value_funclet(&self) -> Option<&FuncletId> {
        self.schedule_explication_data
            .get(&self.location.funclet_name)
            .map(|f| &f.value_funclet)
    }

    pub fn next_name(&mut self) -> String {
        self.meta_data.next_name()
    }
}