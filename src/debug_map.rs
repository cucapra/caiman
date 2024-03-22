use std::collections::HashMap;
use serde_derive::{Deserialize, Serialize};

fn unknown(index: &usize) -> String {
    format!("_UNKNOWN_{}", index)
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct DebugMap {
    // Maps from program indices to original strings in the assembly AST
    // The intention here is purely to recover error messages
    // This structure is decoupled from any one IR/AST
    pub type_map: HashMap<usize, String>,
    pub ffi_type_map: HashMap<usize, String>,
    pub function_class_map: HashMap<usize, String>,
    pub external_function_map: HashMap<usize, String>,
    pub funclet_map: HashMap<usize, FuncletDebugMap>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct FuncletDebugMap {
    // Debug information about a funclet
    pub name: String,
    pub node_map: HashMap<usize, String>,
}

impl DebugMap {
    pub fn get_type(&self, index: &usize) -> String {
        self.type_map.get(index).unwrap_or(&unknown(index)).clone()
    }
    pub fn get_ffi_type(&self, index: &usize) -> String {
        self.ffi_type_map.get(index).unwrap_or(&unknown(index)).clone()
    }
    pub fn get_function_class(&self, index: &usize) -> String {
        self.function_class_map.get(index).unwrap_or(&unknown(index)).clone()
    }
    pub fn get_external_function(&self, index: &usize) -> String {
        self.external_function_map.get(index).unwrap_or(&unknown(index)).clone()
    }
    pub fn get_funclet(&self, index: &usize) -> String {
        self.funclet_map
            .get(index)
            .map(|f| f.name.clone())
            .unwrap_or(unknown(index))
    }
    pub fn get_node(&self, funclet_index: &usize, node_index: &usize) -> String {
        match self.funclet_map.get(funclet_index) {
            None => format!("({}, {})", unknown(funclet_index), unknown(node_index)),
            Some(f) => f
                .node_map
                .get(node_index)
                .unwrap_or(&format!("({}, {})", &f.name, unknown(node_index)))
                .clone(),
        }
    }
}
