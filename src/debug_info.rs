use crate::ir;
use itertools::Itertools;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

fn unknown(index: &usize) -> String {
    format!("__UNNAMED: {}", index)
}

fn unknown_quot(quot: &ir::Quotient) -> String {
    format!("__UNNAMED: {:?}", quot)
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct DebugInfo {
    // Maps from program indices to original strings in the assembly AST
    // The intention here is purely to recover error messages
    // This structure is decoupled from any one IR/AST
    pub type_map: HashMap<usize, String>,
    pub function_class_map: HashMap<usize, String>,
    pub external_function_map: HashMap<usize, String>,
    pub funclet_map: HashMap<usize, FuncletDebugMap>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct FuncletDebugMap {
    // Debug information about a funclet
    pub name: String,
    // we need to use the quotient to differentiate which thing to index into
    pub node_map: HashMap<ir::Quotient, String>,
}

impl DebugInfo {
    pub fn typ(&self, index: &usize) -> String {
        self.type_map.get(index).unwrap_or(&unknown(index)).clone()
    }
    pub fn function_class(&self, index: &usize) -> String {
        self.function_class_map
            .get(index)
            .unwrap_or(&unknown(index))
            .clone()
    }
    pub fn external_function(&self, index: &usize) -> String {
        self.external_function_map
            .get(index)
            .unwrap_or(&unknown(index))
            .clone()
    }
    pub fn funclet(&self, index: &usize) -> String {
        self.funclet_map
            .get(index)
            .map(|f| f.name.clone())
            .unwrap_or(unknown(index))
    }
    pub fn quot(&self, funclet_index: &usize, quotient: &ir::Quotient) -> String {
        match self.funclet_map.get(funclet_index) {
            None => format!("{}.{}", unknown(funclet_index), unknown_quot(quotient)),
            Some(f) => format!("{}.{}", &f.name, f.node_map
                .get(quotient)
                .unwrap_or(&unknown_quot(quotient))
                .clone()),
        }
    }
    pub fn node(&self, funclet_index: &usize, node_index: usize) -> String {
        self.quot(
            funclet_index,
            &ir::Quotient::Node {
                node_id: node_index,
            },
        )
    }
    pub fn tag(&self, funclet_index: &usize, tag: &ir::Tag) -> String {
        format!(
            "Tag: {}, {:?}",
            self.quot(funclet_index, &tag.quot),
            &tag.flow
        )
    }
    pub fn spec(&self, spec: &ir::FuncletSpec) -> String {
        match &spec.funclet_id_opt {
            None => "FuncletSpec { no_id }".to_string(),
            Some(funclet_index) => format!("FuncletSpec {{ Funclet {}, inputs: {:?}, outputs: {:?}, implicit_in: {}, implicit_out: {}}}",
                self.funclet(funclet_index),
                spec.input_tags.iter().map(|t| self.tag(&funclet_index, &t)).collect_vec(),
                spec.output_tags.iter().map(|t| self.tag(&funclet_index, &t)).collect_vec(),
                self.tag(funclet_index, &spec.implicit_in_tag),
                self.tag(funclet_index, &spec.implicit_out_tag))
        }
    }
}
