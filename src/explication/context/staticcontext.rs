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
        let mut result = StaticContext {
            program,
            debug_info,
            spec_explication_data: HashMap::new(),
        };
        result.initialize_declarations();
        result
    }

    // setup

    fn initialize_declarations(&mut self) {
        let mut result = HashMap::new();
        for (index, funclet) in self.program.funclets.iter() {
            match &funclet.kind {
                ir::FuncletKind::Value | ir::FuncletKind::Spatial | ir::FuncletKind::Timeline => {
                    result = self.initialize_spec_funclet_info(result, index, funclet);
                }
                _ => {}
            }
        }
        self.spec_explication_data = result;
    }

    fn initialize_spec_funclet_info(
        &self,
        mut result: HashMap<FuncletId, SpecFuncletData>,
        funclet_id: usize,
        funclet: &expir::Funclet,
    ) -> HashMap<FuncletId, SpecFuncletData> {
        let mut node_dependencies = HashMap::new();
        for (i, node) in funclet.nodes.as_ref().iter().enumerate() {
            match &node {
                Hole::Empty => {}
                Hole::Filled(node) => {
                    node_dependencies.insert(i, self.identify_node_deps(funclet_id, node));
                }
            }
        }
        let tail_dependencies = match &funclet.tail_edge {
            Hole::Empty => panic!(
                "Missing tail edge for spec funclet {}",
                self.debug_info.funclet(&funclet_id)
            ),
            Hole::Filled(t) => self.identify_tailedge_deps(funclet_id, t),
        };

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
            },
        );
        result
    }

    fn identify_node_deps(&self, funclet_id: usize, node: &expir::Node) -> Vec<NodeId> {
        let error = format!(
            "Invalid use of a hole hole in spec funclet {} in node {}",
            &self.debug_info.funclet(&funclet_id),
            &self.debug_info.node_expir(funclet_id, node)
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
                panic!(
                    "Unsupported node in spec funclet {}",
                    self.debug_info.node_expir(funclet_id, node)
                );
            }
        };
        dependencies
    }

    // helper methods for reading information
    fn identify_tailedge_deps(&self, funclet_id: usize, edge: &TailEdge) -> Vec<NodeId> {
        let error = format!(
            "Invalid hole in {:?}, cannot have an explication hole in a spec funclet {}",
            &edge,
            self.debug_info.funclet(&funclet_id)
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
        &self,
        funclet: &expir::Funclet,
        nodeid: NodeId,
        node_dependencies: &HashMap<NodeId, Vec<NodeId>>,
        type_map: &mut HashMap<NodeId, Vec<TypeId>>,
    ) -> Vec<expir::TypeId> {
        if let Some(result) = type_map.get(&nodeid) {
            result.clone()
        } else {
            let result = self.deduce_type(funclet, nodeid, node_dependencies, type_map);
            type_map.insert(nodeid.clone(), result.iter().cloned().collect());
            result
        }
    }

    fn deduce_type(
        &self,
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
                vec![self
                    .get_type_info(
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

    pub fn get_node(&self, location: Location) -> &expir::Node {
        self.program()
            .funclets
            .get(location.funclet_id)
            .expect(&format!(
                "Invalid funclet index {} corresponding with funclet {}",
                location.funclet_id,
                self.debug_info.funclet(&location.funclet_id)
            ))
            .nodes
            .get(location.node_id().unwrap())
            .expect(&format!(
                "Invalid node index {} for funclet {}",
                location.node_id().unwrap(),
                self.debug_info.funclet(&location.funclet_id)
            ))
            .as_ref()
            .opt()
            .expect(&format!(
                "Spec funclet {} cannot have hole",
                self.debug_info.funclet(&location.funclet_id)
            ))
    }

    pub fn get_type(&self, type_id: &TypeId) -> &expir::Type {
        self.program().types.get(*type_id).expect(&format!(
            "Invalid funclet index {} for type {:?} corresponding with type {}",
            type_id,
            &self.program().types,
            self.debug_info.typ(type_id)
        ))
    }
}
