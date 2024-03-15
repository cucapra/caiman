use self::expir::FuncletKind;

use super::*;
use crate::ir;

// These are all getters designed to work with "original" program, before mutations touch things
// Specifically we want things like lists of funclet names up-front or node names up-front

impl StaticContext {
    pub fn new(program: expir::Program) -> StaticContext {
        let mut context = StaticContext {
            program,
            spec_explication_data: HashMap::new(),
        };
        context.initialize_declarations();
        context
    }

    fn initialize_declarations(&mut self) {
        for (index, funclet) in self.program.funclets.iter() {
            match &funclet.kind {
                FuncletKind::Value | FuncletKind::Unknown => {}
                _ => {
                    self.initialize_spec_funclet_info(index, funclet);
                }
            }
        }
    }

    fn initialize_spec_funclet_info(&mut self, index: usize, funclet: &expir::Funclet) {
        let mut node_dependencies = HashMap::new();
        let mut tail_dependencies = Vec::new();
        for (index, node) in funclet.nodes.as_ref().iter().enumerate() {
            match &node {
                None => {}
                Some(node) => {
                    node_dependencies.insert(index, identify_node_deps(node));
                }
            }
        }
        match &funclet.tail_edge {
            None => {}
            Some(t) => {
                identify_tailedge_deps(t);
            }
        }

        // second loop for type information
        let mut deduced_types = HashMap::new();
        match &funclet.kind {
            ir::FuncletKind::Value => {
                for node in funclet.nodes.as_ref() {
                    match node {
                        None => {}
                        // TODO: ???
                        Some(n) => {}
                    }
                }
            }
            _ => {}
        }

        self.spec_explication_data.insert(
            index,
            SpecFuncletData {
                node_dependencies,
                tail_dependencies,
                deduced_types,
                connections: vec![],
            },
        );
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
                None => {}
                Some(name) => {
                    index_map.insert(name.clone(), index);
                }
            }
        }
        let spec_data = self.get_spec_data(funclet);
        for node in self.get_funclet(funclet).nodes.iter() {
            match node {
                Some(expir::Node::ExtractResult { node_id, index }) => {
                    // lots of potentially panicking unwraps here
                    // none of these should be `None` at this point
                    let dependency = spec_data
                        .node_dependencies
                        .get(node_id.as_ref().unwrap())
                        .unwrap()
                        .first()
                        .unwrap();
                    // every extraction of one function should match to that function
                    result = assign_or_compare(result, dependency);
                }
                _ => panic!("Attempted to treat {:?} as an extract operation", node),
            }
        }
        result
    }

    fn get_spec_data(&self, funclet: &FuncletId) -> &SpecFuncletData {
        self.spec_explication_data
            .get(funclet)
            .expect(&format!("Unknown specification function {:?}", funclet))
    }

    fn get_funclet(&self, funclet: &FuncletId) -> &expir::Funclet {
        self.program.funclets.get(*funclet).expect(&format!(
            "Invalid funclet index {} for funclets {:?}",
            funclet, &self.program.funclets
        ))
    }
}

fn identify_node_deps(node: &expir::Node) -> Vec<NodeId> {
    let error = format!(
        "Invalid hole in {:?}, cannot have an explication hole in a spec funclet",
        &node
    );
    let dependencies = match node {
        expir::Node::Phi { index } => {
            vec![]
        }
        expir::Node::ExtractResult { node_id, index } => {
            vec![node_id.expect(&error)]
        }
        expir::Node::Constant { value, type_id } => vec![],
        expir::Node::CallFunctionClass {
            function_id,
            arguments,
        } => arguments
            .as_ref()
            .expect(&error)
            .iter()
            .map(|n| n.expect(&error))
            .collect(),
        expir::Node::Select {
            condition,
            true_case,
            false_case,
        } => {
            vec![
                condition.expect(&error),
                true_case.expect(&error),
                false_case.expect(&error),
            ]
        }
        expir::Node::EncodingEvent {
            local_past,
            remote_local_pasts,
        } => vec![local_past.expect(&error)]
            .into_iter()
            .chain(
                remote_local_pasts
                    .as_ref()
                    .expect(&error)
                    .iter()
                    .map(|n| n.expect(&error)),
            )
            .collect(),
        expir::Node::SubmissionEvent { local_past } => {
            vec![local_past.expect(&error)]
        }
        expir::Node::SynchronizationEvent {
            local_past,
            remote_local_past,
        } => {
            vec![local_past.expect(&error), remote_local_past.expect(&error)]
        }
        expir::Node::SeparatedBufferSpaces { count, space } => {
            vec![space.expect(&error)]
        }
        _ => {
            unreachable!("Unsupported named specification node type {:?}", &node);
        }
    };
    dependencies
}

// helper methods for reading information
fn identify_tailedge_deps(edge: &expir::TailEdge) -> Vec<NodeId> {
    let error = format!(
        "Invalid hole in {:?}, cannot have an explication hole in a spec funclet",
        &edge
    );
    let dependencies = match edge {
        expir::TailEdge::DebugHole { inputs } => inputs.iter().map(|n| n.clone()).collect(),
        expir::TailEdge::Return { return_values } => return_values
            .as_ref()
            .expect(&error)
            .iter()
            .map(|n| n.expect(&error))
            .collect(),
        expir::TailEdge::Jump { join, arguments } => vec![join.expect(&error)]
            .into_iter()
            .chain(
                arguments
                    .as_ref()
                    .expect(&error)
                    .iter()
                    .map(|n| n.expect(&error)),
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
    nodeid: &NodeId,
    node_dependencies: &HashMap<NodeId, Vec<NodeId>>,
    type_map: &mut HashMap<NodeId, Vec<TypeId>>,
) -> Vec<expir::TypeId> {
    if let Some(result) = type_map.get(nodeid) {
        result.clone()
    } else {
        let result = deduce_type(funclet, nodeid, node_dependencies, type_map);
        type_map.insert(nodeid.clone(), result.iter().cloned().collect());
        result
    }
}

fn deduce_type(
    funclet: &expir::Funclet,
    check_id: &NodeId,
    node_dependencies: &HashMap<NodeId, Vec<NodeId>>,
    type_map: &mut HashMap<NodeId, Vec<TypeId>>,
) -> Vec<TypeId> {
    let names = node_dependencies
        .get(check_id)
        .expect(&format!("Unknown spec node dependency {:?}", check_id));
    let node = funclet.nodes.get(*check_id).expect(&format!(
        "Invalid index {:?} looking up node in funclet {:?}",
        check_id, &funclet
    ));
    let error = format!(
        "Invalid hole in {:?}, cannot have an explication hole in a spec funclet",
        &node
    );
    let typ = match node.expect(&error) {
        expir::Node::Phi { index } => {
            vec![get_expect_box(&funclet.input_types, index.expect(&error)).clone()]
        }
        expir::Node::ExtractResult { node_id, index } => {
            let index = index.expect(&error);
            vec![get_type_info(
                funclet,
                node_id.as_ref().expect(&error),
                node_dependencies,
                type_map,
            )
            .get(index)
            .expect(&format!(
                "Not enough arguments to extract index {} from {:?}",
                index, node_id
            )).clone()]
        }
        expir::Node::Constant { value, type_id } => { Vec::new() }
        expir::Node::CallFunctionClass {
            function_id,
            arguments,
        } => { Vec::new() }
        expir::Node::Select {
            condition,
            true_case,
            false_case,
        } => { Vec::new() }
        _ => unreachable!("Not a value node {:?}", node),
    };
    typ
}
