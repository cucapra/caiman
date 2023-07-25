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
    pub fn allocate(
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
            .insert(
                AllocationInfo {
                    schedule_funclet,
                    place,
                },
                schedule_node,
            );
    }
}

impl ScheduleFuncletData {
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
