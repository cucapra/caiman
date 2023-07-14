use super::*;

impl NodeExplicationInformation {
    pub fn new() -> NodeExplicationInformation {
        NodeExplicationInformation {
            scheduled_allocations: Default::default(),
            scheduled_operations: Default::default(),
            operative_locations: Default::default(),
        }
    }
    fn allocate(&mut self, schedule_funclet: FuncletId, schedule_node: NodeId, place: ir::Place) {
        self.operative_locations.insert(schedule_funclet.clone());
        let result = self.scheduled_allocations.insert(
            AllocationInfo {
                schedule_funclet,
                place,
            },
            schedule_node,
        );
        assert!(result.is_none());
    }
    fn schedule_operation(&mut self, schedule_funclet: FuncletId, schedule_node: NodeId) {
        self.operative_locations.insert(schedule_funclet.clone());
        self.scheduled_operations
            .entry(schedule_funclet)
            .or_insert(Vec::new())
            .push(schedule_node);
    }
}

impl SpecFuncletData {
    pub fn new() -> SpecFuncletData {
        SpecFuncletData {
            connections: Vec::new(),
            explication_information: HashMap::new(),
            call_outputs: HashMap::new(),
        }
    }
    fn allocate(
        &mut self,
        value_node: NodeId,
        schedule_funclet: FuncletId,
        schedule_node: NodeId,
        place: ir::Place,
    ) {
        self.explication_information
            .entry(value_node)
            .or_insert(NodeExplicationInformation::new())
            .scheduled_allocations
            .insert(schedule_funclet, schedule_node);
    }
}

impl ScheduleFuncletData {
    pub fn new(
        value_funclet: FuncletId,
        timeline_funclet: FuncletId,
        spatial_funclet: FuncletId,
    ) -> ScheduleFuncletData {
        ScheduleFuncletData {
            value_funclet,
            timeline_funclet,
            spatial_funclet,
            allocations: HashMap::new(),
            explication_holes: Vec::new(),
        }
    }
    fn allocate(&mut self, node: NodeId, location: RemoteNodeId) {
        let result = self.allocations.insert(node, location);
        assert!(result.is_none());
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
