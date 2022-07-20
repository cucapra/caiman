use crate::arena::Arena;
use crate::ir;
use std::{
    collections::hash_map::{Entry, HashMap},
    convert::{TryFrom, TryInto},
};

macro_rules! make_id {
    ($name:ident, $repr:ty) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        struct $name($repr);
        impl $name {
            const INVALID: Self = Self(<$repr>::MAX);
        }
    };
}
make_id!(NodeId, usize);

#[derive(Clone)]
struct Node {
    funclet_id: ir::FuncletId,
    outgoing: Vec<NodeId>,
}

pub struct FuncletCfg {
    nodes: Vec<Node>,
    lookup: HashMap<ir::FuncletId, NodeId>,
    head: ir::FuncletId,
}
impl FuncletCfg {
    pub fn new(head: ir::FuncletId, funclets: &Arena<ir::Funclet>) -> Self {
        let mut nodes = Vec::new();
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
                        outgoing: Vec::new(),
                    });
                    let tail = &funclets[&funclet_id].tail_edge;
                    tail.for_each_funclet(|id| stack.push((id, node_id)));
                    node_id
                }
            };
            if incoming_node_id != NodeId::INVALID {
                nodes[incoming_node_id.0].outgoing.push(node_id)
            }
        }
        Self {
            nodes,
            lookup,
            head,
        }
    }

    pub fn tarjan_scc(&self) -> Vec<Vec<NodeId>> {
        make_id!(ComponentId, usize);

        #[derive(Clone, Copy)]
        struct NodeInfo {
            index: ComponentId,
            low_link: ComponentId,
            on_stack: bool,
        }
        fn strong_connect(
            node_id: NodeId,
            nodes: &[Node],
            next_index: &mut ComponentId,
            node_info: &mut [NodeInfo],
            node_stack: &mut Vec<NodeId>,
            component_stack: &mut Vec<Vec<NodeId>>,
        ) {
            node_stack.push(node_id);
            node_info[node_id.0] = NodeInfo {
                index: *next_index,
                low_link: *next_index,
                on_stack: true,
            };
            *next_index = ComponentId(next_index.0 + 1);

            // for each successor...
            for succ in nodes[node_id.0].outgoing.iter() {
                if node_info[succ.0].index == ComponentId::INVALID {
                    strong_connect(
                        *succ,
                        nodes,
                        next_index,
                        node_info,
                        node_stack,
                        component_stack,
                    );
                    node_info[node_id.0].low_link = std::cmp::min(
                        node_info[node_id.0].low_link, //
                        node_info[succ.0].low_link,
                    );
                } else if node_info[succ.0].on_stack {
                    node_info[node_id.0].low_link = std::cmp::min(
                        node_info[node_id.0].low_link, //
                        node_info[succ.0].index,
                    );
                }
            }

            if node_info[node_id.0].low_link == node_info[node_id.0].index {
                let mut component = Vec::new();
                loop {
                    let included = node_stack.pop().unwrap();
                    node_info[included.0].on_stack = false;
                    component.push(included);
                    if included == node_id {
                        break;
                    }
                }
                component_stack.push(component);
            }
        }

        let mut next_index = ComponentId(0);
        let mut node_stack = Vec::new();
        let mut node_info = vec![
            NodeInfo {
                index: ComponentId::INVALID,
                low_link: ComponentId::INVALID,
                on_stack: false
            };
            self.nodes.len()
        ];
        let mut component_stack = Vec::new();
        for node_id in 0..self.nodes.len() {
            if node_info[node_id].index == ComponentId::INVALID {
                strong_connect(
                    NodeId(node_id),
                    &self.nodes,
                    &mut next_index,
                    &mut node_info,
                    &mut node_stack,
                    &mut component_stack,
                );
            }
        }
        component_stack
    }
    pub fn make_reducible(&mut self) {
        // We transform the (potentially irreducible) CFG into a reducible CFG by using
        // node splitting. This can cause a potentially exponential blowup in program size,
        // but it's probably not worth worrying about: https://doi.org/10.1002/spe.1059

        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn assert_cfg(tails: &[ir::TailEdge], sccs: &[&[usize]]) {
        let pipeline = ir::Pipeline {
            name: "test pipeline".to_string(),
            entry_funclet: 0,
            kind: ir::PipelineKind::Function,
        };
        // NOTE: correctness here depends on empty arenas adding nodes with sequential ids
        let mut funclets = Arena::new();
        for tail_edge in tails.iter() {
            funclets.create(ir::Funclet {
                tail_edge: tail_edge.clone(),
                // everything below this point is probably incorrect
                kind: ir::FuncletKind::MixedExplicit,
                input_types: Box::new([]),
                output_types: Box::new([]),
                nodes: Box::new([]),
                input_resource_states: Default::default(),
                output_resource_states: Default::default(),
                local_meta_variables: Default::default(),
            });
        }
        let graph = FuncletCfg::new(pipeline.entry_funclet, &funclets);
        let mut actual = graph.tarjan_scc();
        for scc in actual.iter_mut() {
            scc.sort_unstable();
        }
        let mut expected = Vec::new();
        for scc in sccs {
            let mut scc: Vec<_> = scc.iter().map(|&id| NodeId(id)).collect();
            scc.sort_unstable();
            expected.push(scc);
        }
        assert_eq!(actual, expected);
    }
    macro_rules! ret {
        () => {
            ir::TailEdge::Return {
                return_values: Box::new([]),
            }
        };
    }
    macro_rules! jmp {
        ($i:expr) => {
            ir::TailEdge::Jump(ir::Jump {
                target: $i,
                args: Box::new([]),
            })
        };
    }
    macro_rules! sel {
        ($($i:expr),* $(,)?) => {
            ir::TailEdge::Switch {
                key: 0,
                cases: Box::new([
                    $(ir::Jump {target: $i, args: Box::new([])}),*
                ])
            }
        }
    }

    #[test]
    fn basic() {
        assert_cfg(&[ret!()], &[&[0]]);
        assert_cfg(&[jmp!(1), jmp!(2), ret!()], &[&[2], &[1], &[0]])
    }
    #[test]
    fn branches() {
        assert_cfg(
            &[sel![1, 2], sel![2, 3], ret!(), ret!()],
            &[&[3], &[2], &[1], &[0]],
        );
    }
    #[test]
    fn loops() {
        assert_cfg(&[jmp!(0)], &[&[0]]);
        assert_cfg(&[jmp!(1), jmp!(0)], &[&[0, 1]]);
        assert_cfg(&[sel!(1, 2), jmp!(3), jmp!(3), jmp!(0)], &[&[0, 1, 2, 3]]);
        assert_cfg(
            &[jmp!(1), sel!(2, 3), jmp!(4), jmp!(4), sel!(1, 5), ret!()],
            &[&[5], &[1, 2, 3, 4], &[0]],
        )
    }
}
