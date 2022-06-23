#![warn(warnings)]
use crate::ir;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {}

type Result<T> = std::result::Result<T, Error>;

/// Transformations which operate on individual funclets in isolation.
mod funclet {
    use super::*;
    use std::collections::HashMap;

    /// Prunes all unused nodes from `funclet`.
    pub fn prune_unused_nodes(funclet: &mut ir::Funclet) -> Result<()> {
        /// Mark the node given by `id`, and all its value dependencies, as used by appending
        /// their node ids to `used_nodes`.
        fn mark_used(funclet: &ir::Funclet, id: ir::NodeId, used_ids: &mut Vec<ir::NodeId>) {
            used_ids.push(id);
            (&funclet.nodes[id])
                .for_each_referenced_node(|referenced| mark_used(funclet, referenced, used_ids))
        }
        // IDs of all used nodes.
        let mut used_ids: Vec<ir::NodeId> = Vec::with_capacity(funclet.nodes.len());
        funclet
            .tail_edge
            .for_each_referenced_node(|referenced| mark_used(funclet, referenced, &mut used_ids));
        used_ids.sort();
        used_ids.dedup();

        // Map from old node ID to new node ID. Has no entry for unused nodes.
        let inverse: HashMap<_, _> = used_ids.iter().enumerate().map(|(x, &y)| (y, x)).collect();

        let new_nodes: Vec<ir::Node> = used_ids
            .into_iter()
            .map(|id| funclet.nodes[id].map_referenced_nodes(|referenced| inverse[&referenced]))
            .collect();

        funclet.nodes = new_nodes.into_boxed_slice();
        funclet.tail_edge = funclet
            .tail_edge
            .map_referenced_nodes(|referenced| inverse[&referenced]);

        Ok(())
    }
}

/// Applies a set of optimizing transformations to the IR. These transformations only operate
/// on value operations; after the value/scheduling language split, this will become specific
/// to the value language.
pub fn apply(program: &mut ir::Program) -> Result<()> {
    for (_, f) in program.funclets.iter_mut() {
        funclet::prune_unused_nodes(f)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
