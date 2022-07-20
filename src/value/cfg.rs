use crate::arena::Arena;
use crate::ir;
use std::{
    collections::hash_map::{Entry, HashMap},
    convert::{TryFrom, TryInto},
};

macro_rules! make_id {
    ($name:ident, $repr:ty) => {
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        struct $name($repr);
        impl $name {
            const INVALID: Self = Self(<$repr>::MAX);
        }
    };
}
make_id!(NodeId, usize);
make_id!(EdgeId, usize);

#[derive(Clone, Copy)]
struct Node {
    funclet_id: ir::FuncletId,
    first_edge: EdgeId,
}

#[derive(Clone, Copy)]
struct Edge {
    destination: NodeId,
    next_edge: EdgeId,
}
pub struct PipelineCfg {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    lookup: HashMap<ir::FuncletId, NodeId>,
    head: ir::FuncletId,
}
impl PipelineCfg {
    fn new(head: ir::FuncletId, funclets: &Arena<ir::Funclet>) -> Self {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut lookup = HashMap::new();
        let mut stack = Vec::new();
        stack.push((head, NodeId::INVALID));
        while let Some((funclet_id, incoming_node_id)) = stack.pop() {
            let node_id = match lookup.entry(funclet_id) {
                Entry::Occupied(node_id) => *node_id.get(),
                Entry::Vacant(spot) => {
                    let node_id = NodeId(nodes.len());
                    spot.insert(node_id);
                    nodes.push(Node {
                        funclet_id,
                        first_edge: EdgeId::INVALID,
                    });
                    let tail = &funclets[&funclet_id].tail_edge;
                    tail.for_each_funclet(|id| stack.push((id, node_id)));
                    node_id
                }
            };
            if incoming_node_id != NodeId::INVALID {
                let existing_edge = nodes[incoming_node_id.0].first_edge;
                let new_edge_id = EdgeId(edges.len());
                edges.push(Edge {
                    destination: node_id,
                    next_edge: existing_edge,
                });
                nodes[incoming_node_id.0].first_edge = new_edge_id;
            }
        }
        Self {
            nodes,
            edges,
            lookup,
            head,
        }
    }
}
fn compute_apeg(pipeline: &ir::Pipeline, funclets: &Arena<ir::Funclet>) {}
