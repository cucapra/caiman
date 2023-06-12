use super::*;

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
    pub fn new() -> MetaData {
        MetaData { name_index: 0 }
    }
    pub fn next_name(&mut self) -> String {
        self.name_index += 1;
        format!("~{}", self.name_index)
    }
}
