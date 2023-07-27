use super::*;

impl AlloctionHoleInfo {}

impl ScheduleScopeData {
    pub fn add_instantiation(&mut self, schedule_node: NodeId, info: ScheduledInstantiationInfo) {
        let error_string = format!("Multiple instantiations of {:?} not supported", &info);
        let result = self.instantiations.insert(info, schedule_node);
        match result {
            None => {}
            Some(dup) => panic!(error_string),
        }
    }

    pub fn add_allocation(&mut self, node: NodeId, info: AlloctionHoleInfo) {
        let result = self
            .available_allocations
            .entry(info)
            .or_insert_with(|| Vec::new())
            .push(node);
    }

    pub fn add_operation(&mut self, node: NodeId, info: OperationInfo) {
        let result = self
            .available_operations
            .entry(info)
            .or_insert_with(|| Vec::new())
            .push(node);
    }

    pub fn add_explication_hole(&mut self, node: NodeId) {
        self.explication_hole = Some(node);
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