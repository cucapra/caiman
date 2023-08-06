use super::*;
use crate::assembly::explication::util::*;

impl LocalTypeDeclaration {
    pub fn new(info: &ast::LocalTypeInfo) -> LocalTypeDeclaration {
        let (place, ffi) = match info {
            ast::LocalTypeInfo::NativeValue { storage_type } => {
                (None, Some(unwrap_ffi_type(storage_type.clone())))
            }
            ast::LocalTypeInfo::Ref {
                storage_type,
                storage_place,
            } => (
                Some(storage_place.clone()),
                Some(unwrap_ffi_type(storage_type.clone())),
            ),
            ast::LocalTypeInfo::Fence { queue_place } => (Some(queue_place.clone()), None),
            ast::LocalTypeInfo::Buffer {
                storage_place,
                static_layout_opt,
            } => (Some(storage_place.clone()), None),
            ast::LocalTypeInfo::Encoder { queue_place } => (Some(queue_place.clone()), None),
            ast::LocalTypeInfo::Event => (None, None),
            ast::LocalTypeInfo::BufferSpace => (None, None),
        };
        LocalTypeDeclaration { place, ffi }
    }
}

impl ScheduleScopeData {
    pub fn add_instantiation(&mut self, schedule_node: CommandId, info: ScheduledInstantiationInfo) {
        let error_string = format!("Multiple instantiations of {:?} not supported", &info);
        self.instantiations
            .entry(info)
            .or_insert(Vec::new())
            .push(schedule_node);
    }

    pub fn add_operation(&mut self, node: CommandId, operation: OpCode) {
        let result = self
            .available_operations
            .entry(operation)
            .or_insert_with(|| Vec::new())
            .push(node);
    }

    pub fn add_explication_hole(&mut self, node: CommandId) {
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
