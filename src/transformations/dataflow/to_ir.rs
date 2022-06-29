use super::{NodeIndex, Operation};
use crate::convert::{ConversionContext, Convert};
use crate::ir;
use std::collections::HashMap;
use std::convert::Infallible;

/// Context used for dataflow-to-IR conversions.
pub struct Context<'a> {
    /// Map from a node's index in the dataflow graph to its IR id
    node_map: &'a HashMap<NodeIndex, ir::NodeId>,
}
impl<'a> Context<'a> {
    pub fn new(node_map: &'a HashMap<NodeIndex, ir::NodeId>) -> Self {
        Self { node_map }
    }
    pub fn id_for_index(&self, index: NodeIndex) -> Result<ir::NodeId, Infallible> {
        // This is *not* an infallible operation. However, if it fails, the bug is in
        // the topological sort/IR emission, which is an implementation bug of this module.
        // For this reason I don't think it's worth it to return a meaningful error, but
        // this might change in the future...
        Ok(self.node_map[&index])
    }
}
impl<'a> ConversionContext for Context<'a> {
    type Error = Infallible;
}

impl<'a> Convert<ir::NodeId, Context<'a>> for NodeIndex {
    fn convert(self, context: &Context) -> Result<ir::NodeId, Infallible> {
        context.id_for_index(self)
    }
}
impl<'a> Convert<Box<[ir::NodeId]>, Context<'a>> for Box<[NodeIndex]> {
    fn convert(self, context: &Context) -> Result<Box<[ir::NodeId]>, Infallible> {
        self.iter().map(|&id| id.convert(context)).collect()
    }
}

include!(concat!(env!("OUT_DIR"), "/generated/dataflow_to_ir.rs"));
