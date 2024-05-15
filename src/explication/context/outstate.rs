use super::*;
use crate::explication::util::*;
use std::collections::hash_map::Entry;
use std::fmt::Debug;

impl OperationOutState {
    pub fn new() -> OperationOutState {
        OperationOutState {
            nodes: VecDeque::new(),
            tail_edge: None,
        }
    }

    pub fn add_hole(&mut self) {
        self.nodes.push_front(Hole::Empty);
    }

    pub fn add_node(&mut self, node: expir::Node) {
        self.nodes.push_front(Hole::Filled(node));
    }

    pub fn drain_nodes(&mut self) -> Vec<Hole<expir::Node>> {
        self.nodes.drain(..).collect()
    }

    pub fn set_tail_edge(&mut self, tail_edge: expir::TailEdge) {
        assert!(self.tail_edge.is_none());
        self.tail_edge = Some(tail_edge)
    }

    pub fn has_tail_edge(&self) -> bool {
        self.tail_edge.is_some()
    }

    pub fn take_tail_edge(&mut self) -> Option<expir::TailEdge> {
        self.tail_edge.take()
    }
}

impl StorageOutState {
    pub fn new() -> StorageOutState {
        StorageOutState {
            to_fill: HashMap::new(),
            nodes: VecDeque::new(),
            tail_edge: None,
        }
    }

    pub fn add_node(&mut self, node: ir::Node) {
        self.nodes.push_front(node);
    }

    pub fn drain_nodes(&mut self) -> Vec<ir::Node> {
        self.nodes.drain(..).collect()
    }

    pub fn take_to_fill(&mut self, node_id: &NodeId) -> Option<ir::Node> {
        self.to_fill.remove_entry(node_id).map(|(_, n)| n)
    }

    pub fn add_to_fill(&mut self, node_id: NodeId, node: ir::Node) {
        let check = self.to_fill.insert(node_id, node);
        assert!(check.is_none());
    }

    pub fn is_to_fill_empty(&self) -> bool {
        self.to_fill.is_empty()
    }

    pub fn set_tail_edge(&mut self, tail_edge: ir::TailEdge) {
        assert!(self.tail_edge.is_none());
        self.tail_edge = Some(tail_edge)
    }

    pub fn has_tail_edge(&self) -> bool {
        self.tail_edge.is_some()
    }

    pub fn expect_tail_edge(&mut self) -> ir::TailEdge {
        self.tail_edge.as_ref().expect("No tail edge found").clone()
    }
}