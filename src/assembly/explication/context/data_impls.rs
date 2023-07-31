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

    pub fn add_operation(&mut self, node: NodeId, operation: OpCode) {
        let result = self
            .available_operations
            .entry(operation)
            .or_insert_with(|| Vec::new())
            .push(node);
    }

    pub fn add_explication_hole(&mut self, node: NodeId) {
        self.explication_hole = Some(node);
    }
}

macro_rules! op_code_initialization {
    ($($_lang:ident $name:ident ($($_arg:ident : $_arg_type:tt,)*) -> $_output:ident;)*) => {
        impl OpCode {
            pub fn new(node: &ast::Node) -> OpCode {
                match node {
                    $(ast::Node::$name { .. } => OpCode::$name,)*
                }
            }
        }
    };
}

with_operations!(op_code_initialization);

impl MetaData {
    pub fn new() -> MetaData {
        MetaData { name_index: 0 }
    }
    pub fn next_name(&mut self) -> String {
        self.name_index += 1;
        format!("~{}", self.name_index)
    }
}