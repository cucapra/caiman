use super::*;
use crate::explication::util::*;
use std::collections::hash_map::Entry;
use std::fmt::Debug;

impl FuncletOutState {
    pub fn new() -> FuncletOutState {
        FuncletOutState {
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