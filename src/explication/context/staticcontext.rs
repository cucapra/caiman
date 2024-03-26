use self::{
    expir::{FuncletKind, TailEdge},
    DebugInfo,
};

use super::*;
use crate::ir;

// These are all getters designed to work with "original" program, before mutations touch things
// Specifically we want things like lists of funclet names up-front or node names up-front

impl<'context> StaticContext<'context> {
    pub fn new(
        program: &'context expir::Program,
        debug_info: &'context DebugInfo,
    ) -> StaticContext<'context> {
        let spec_explication_data = initialize_declarations(&program, debug_info);
        StaticContext {
            program,
            debug_info,
            spec_explication_data,
        }
    }

    // for quick and dirty things
    pub fn program(&self) -> &expir::Program {
        &self.program
    }

    pub fn get_matching_operation(
        &self,
        funclet: &FuncletId,
        returns: Vec<Hole<&NodeId>>,
    ) -> Option<&NodeId> {
        let mut result = None;
        let mut index_map = HashMap::new();
        for (index, ret) in returns.into_iter().enumerate() {
            match ret {
                Hole::Empty => {}
                Hole::Filled(name) => {
                    index_map.insert(name.clone(), index);
                }
            }
        }
        let debug_funclet = self.debug_info.funclet(funclet);
        let spec_data = self.get_spec_data(funclet);
        for (index, node) in self.get_funclet(funclet).nodes.iter().enumerate() {
            match node {
                Hole::Filled(expir::Node::ExtractResult { node_id, index }) => {
                    // lots of potentially panicking unwraps here
                    // none of these should be `None` at this point
                    let dependency = spec_data
                        .node_dependencies
                        .get(node_id.as_ref().opt().unwrap())
                        .unwrap()
                        .first()
                        .unwrap();
                    // every extraction of one function should match to that function
                    result = assign_or_compare(result, dependency);
                }
                _ => {
                    panic!(
                        "Attempted to treat {} as an extract operation",
                        self.debug_info.node(funclet, index)
                    )
                }
            }
        }
        result
    }

    fn get_spec_data(&self, funclet: &FuncletId) -> &SpecFuncletData {
        self.spec_explication_data.get(&funclet).expect(&format!(
            "Unknown specification function {:?}",
            self.debug_info.funclet(funclet)
        ))
    }

    pub fn get_funclet(&self, funclet: &FuncletId) -> &expir::Funclet {
        self.program().funclets.get(*funclet).expect(&format!(
            "Invalid funclet index {} for funclets {:?} corresponding with funclet {}",
            funclet,
            &self.program().funclets,
            self.debug_info.funclet(funclet)
        ))
    }
}

fn initialize_declarations(
    program: &expir::Program,
    debug_info: &DebugInfo,
) -> HashMap<FuncletId, SpecFuncletData> {
    let mut result = HashMap::new();
    for (index, funclet) in program.funclets.iter() {
        match &funclet.kind {
            ir::FuncletKind::Value | ir::FuncletKind::Spatial | ir::FuncletKind::Timeline => {
                result = initialize_spec_funclet_info(result, debug_info, index, funclet);
            }
            _ => {}
        }
    }
    result
}

fn initialize_spec_funclet_info(
    mut result: HashMap<FuncletId, SpecFuncletData>,
    debug_info: &DebugInfo,
    funclet_id: usize,
    funclet: &expir::Funclet,
) -> HashMap<FuncletId, SpecFuncletData> {
    let mut node_dependencies = HashMap::new();
    let mut tail_dependencies = Vec::new();
    for (i, node) in funclet.nodes.as_ref().iter().enumerate() {
        match &node {
            Hole::Empty => {}
            Hole::Filled(node) => {
                node_dependencies.insert(
                    i,
                    identify_node_deps(&debug_info.node(&funclet_id, i), node),
                );
            }
        }
    }
    match &funclet.tail_edge {
        Hole::Empty => {}
        Hole::Filled(t) => {
            identify_tailedge_deps(&debug_info.funclet(&funclet_id), t);
        }
    }

    // second loop for type information
    let mut deduced_types = HashMap::new();
    match &funclet.kind {
        ir::FuncletKind::Value => {
            for node in funclet.nodes.as_ref() {
                match node {
                    Hole::Empty => {}
                    // TODO: ???
                    Hole::Filled(n) => {}
                }
            }
        }
        _ => {}
    }

    result.insert(
        funclet_id,
        SpecFuncletData {
            node_dependencies,
            tail_dependencies,
            deduced_types,
            connections: vec![],
        },
    );
    result
}

fn identify_node_deps(debug_node: &str, node: &expir::Node) -> Vec<NodeId> {
    let error = format!(
        "Invalid hole in {:?}, cannot have an explication hole in a spec funclet",
        &node
    );
    let dependencies = match node {
        expir::Node::Phi { index } => {
            vec![]
        }
        expir::Node::ExtractResult { node_id, index } => {
            vec![node_id.clone().opt().expect(&error)]
        }
        expir::Node::Constant { value, type_id } => vec![],
        expir::Node::CallFunctionClass {
            function_id,
            arguments,
        } => arguments
            .as_ref()
            .opt()
            .expect(&error)
            .iter()
            .map(|n| n.clone().opt().expect(&error))
            .collect(),
        expir::Node::Select {
            condition,
            true_case,
            false_case,
        } => {
            vec![
                condition.clone().opt().expect(&error),
                true_case.clone().opt().expect(&error),
                false_case.clone().opt().expect(&error),
            ]
        }
        expir::Node::EncodingEvent {
            local_past,
            remote_local_pasts,
        } => vec![local_past.clone().opt().expect(&error)]
            .into_iter()
            .chain(
                remote_local_pasts
                    .clone()
                    .opt()
                    .as_ref()
                    .expect(&error)
                    .iter()
                    .map(|n| n.clone().opt().expect(&error)),
            )
            .collect(),
        expir::Node::SubmissionEvent { local_past } => {
            vec![local_past.clone().opt().expect(&error)]
        }
        expir::Node::SynchronizationEvent {
            local_past,
            remote_local_past,
        } => {
            vec![
                local_past.clone().opt().expect(&error),
                remote_local_past.clone().opt().expect(&error),
            ]
        }
        expir::Node::SeparatedBufferSpaces { count, space } => {
            vec![space.clone().opt().expect(&error)]
        }
        _ => {
            panic!("Unsupported node {} of type {:?}", debug_node, &node);
        }
    };
    dependencies
}

// helper methods for reading information
fn identify_tailedge_deps(debug_funclet: &str, edge: &TailEdge) -> Vec<NodeId> {
    let error = format!(
        "Invalid hole in {:?}, cannot have an explication hole in a spec funclet {}",
        &edge, debug_funclet
    );
    let dependencies = match edge {
        expir::TailEdge::DebugHole { inputs } => inputs.iter().map(|n| n.clone()).collect(),
        expir::TailEdge::Return { return_values } => return_values
            .clone()
            .opt()
            .as_ref()
            .expect(&error)
            .iter()
            .map(|n| n.clone().opt().expect(&error))
            .collect(),
        expir::TailEdge::Jump { join, arguments } => vec![join.clone().opt().expect(&error)]
            .into_iter()
            .chain(
                arguments
                    .clone()
                    .opt()
                    .as_ref()
                    .expect(&error)
                    .iter()
                    .map(|n| n.clone().opt().expect(&error)),
            )
            .collect(),
        _ => {
            unreachable!("Unsupported specification tailedge type {:?}", &edge);
        }
    };
    dependencies
}

fn get_type_info(
    funclet: &expir::Funclet,
    nodeid: NodeId,
    node_dependencies: &HashMap<NodeId, Vec<NodeId>>,
    type_map: &mut HashMap<NodeId, Vec<TypeId>>,
) -> Vec<expir::TypeId> {
    if let Some(result) = type_map.get(&nodeid) {
        result.clone()
    } else {
        let result = deduce_type(funclet, nodeid, node_dependencies, type_map);
        type_map.insert(nodeid.clone(), result.iter().cloned().collect());
        result
    }
}

fn deduce_type(
    funclet: &expir::Funclet,
    check_id: NodeId,
    node_dependencies: &HashMap<NodeId, Vec<NodeId>>,
    type_map: &mut HashMap<NodeId, Vec<TypeId>>,
) -> Vec<TypeId> {
    let names = node_dependencies
        .get(&check_id)
        .expect(&format!("Unknown spec node dependency {:?}", check_id));
    let node = get_expect_box(&funclet.nodes, check_id);
    let error = format!(
        "Invalid hole in {:?}, cannot have an explication hole in a spec funclet",
        &node
    );
    let typ = match node.clone().opt().expect(&error) {
        expir::Node::Phi { index } => {
            vec![get_expect_box(&funclet.input_types, index.opt().expect(&error)).clone()]
        }
        expir::Node::ExtractResult { node_id, index } => {
            let index = index.opt().expect(&error);
            vec![get_type_info(
                funclet,
                *node_id.as_ref().opt().expect(&error),
                node_dependencies,
                type_map,
            )
            .get(index)
            .expect(&format!(
                "Not enough arguments to extract index {} from {:?}",
                index, node_id
            ))
            .clone()]
        }
        expir::Node::Constant { value, type_id } => Vec::new(),
        expir::Node::CallFunctionClass {
            function_id,
            arguments,
        } => Vec::new(),
        expir::Node::Select {
            condition,
            true_case,
            false_case,
        } => Vec::new(),
        _ => unreachable!("Not a value node {:?}", node),
    };
    typ
}
