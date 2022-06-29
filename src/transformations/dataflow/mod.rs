use crate::ir;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct NodeIndex(usize);

// Pulls in Operation enum & inherent methods
include!(concat!(env!("OUT_DIR"), "/generated/dataflow_base.rs"));

mod from_ir;
mod to_ir;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to translate IR to dataflow graph")]
    FromIr(#[from] from_ir::Error),
}
enum Node {
    Reference(NodeIndex),
    Operation(Operation),
}

pub struct Graph {
    nodes: Vec<Node>,
}
impl Graph {
    fn new(ir_nodes: Vec<ir::Node>) -> Result<Self, Error> {
        todo!()
    }
    fn into_ir(self) -> Result<Vec<ir::Node>, Error> {
        todo!()
    }
}
