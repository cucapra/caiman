use super::{NodeIndex, Operation};
use crate::convert::{ConversionContext, Convert};
use crate::ir;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    /// This error is produced when an [`ir::Node`](crate::ir) depends on another node which:
    /// 1. occurs after the dependent node, or
    /// 2. doesn't exist at all
    #[error("ir::Node #{dependent} has invalid dependency on ir::Node #{dependency}")]
    InvalidDependency {
        /// The ID of the node which caused the failure.
        dependent: ir::NodeId,
        /// The ID of the dependency
        dependency: ir::NodeId,
    },
}

/// Context used for IR-to-dataflow conversions.
pub struct Context<'a> {
    /// Map from a node's IR id to its index in the dataflow graph. This is currently implemented
    /// as a slice indexed by `ir::NodeId`, but that's subject to change.
    node_map: &'a [NodeIndex],
    /// ID of the current node, used for error reporting
    cur_id: ir::NodeId,
}
impl<'a> Context<'a> {
    pub fn new(node_map: &'a [NodeIndex], cur_id: ir::NodeId) -> Self {
        Self { node_map, cur_id }
    }
    /// Retrieves the node index associated with the IR node given by `id`.
    pub fn index_for_id(&self, id: ir::NodeId) -> Result<NodeIndex, Error> {
        // TODO: this only works as long as ir::NodeId is just a usize, if that changes
        // just make node_map into &'a HashMap
        match self.node_map.get(id) {
            Some(&index) => Ok(index),
            None => Err(Error::InvalidDependency {
                dependent: self.cur_id,
                dependency: id,
            }),
        }
    }
}
impl<'a> ConversionContext for Context<'a> {
    type Error = Error;
}

impl<'a> Convert<NodeIndex, Context<'a>> for ir::NodeId {
    fn convert(self, context: &Context) -> Result<NodeIndex, Error> {
        context.index_for_id(self)
    }
}
impl<'a> Convert<Box<[NodeIndex]>, Context<'a>> for Box<[ir::NodeId]> {
    fn convert(self, context: &Context) -> Result<Box<[NodeIndex]>, Error> {
        self.iter().map(|&id| id.convert(context)).collect()
    }
}

include!(concat!(env!("OUT_DIR"), "/generated/dataflow_from_ir.rs"));
