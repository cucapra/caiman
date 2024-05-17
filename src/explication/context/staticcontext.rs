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
        let mut deduced_types = HashMap::new();
        for node_id in 0..funclet.nodes.len() {
            node_dependencies.insert(node_id, self.identify_node_deps(funclet_id, node_id));
            self.deduce_type(funclet_id, node_id, &node_dependencies, &mut deduced_types);
        }

        let tail_dependencies = match &funclet.tail_edge {
            Hole::Empty => panic!(
                "Missing tail edge for spec funclet {}",
                self.debug_info.funclet(&funclet_id)
            ),
            Hole::Filled(t) => self.identify_tailedge_deps(funclet_id, t),
        };

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

    fn identify_node_deps(&self, funclet_id: FuncletId, node_id: NodeId) -> Vec<NodeId> {
        let error = format!(
            "Invalid use of a hole in node {}",
            &self.debug_info.node(&funclet_id, node_id)
        );
        let dependencies = match self.get_node(Location::new(funclet_id, node_id)) {
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
            node => {
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

    // helper function that deduces the type for check_id
    // handles the errors assuming that the given node must have one output type
    fn expect_one_output_type(
        &self,
        funclet_id: FuncletId,
        check_id: NodeId,
        node_dependencies: &HashMap<NodeId, Vec<NodeId>>,
        type_map: &mut HashMap<NodeId, SpecNodeTypeInformation>,
    ) -> usize {
        let output_types = &self
            .deduce_type(funclet_id, check_id, node_dependencies, type_map)
            .output_types;
        assert!(
            output_types.len() == 1,
            "Expected node with exactly one return type, got {:?}: in spec node {}",
            output_types,
            self.debug_info.node(&funclet_id, check_id)
        );
        output_types.first().unwrap().clone()
    }

    fn deduce_type(
        &'context self,
        funclet_id: FuncletId,
        node_id: NodeId,
        node_dependencies: &HashMap<NodeId, Vec<NodeId>>,
        deduced_types: &'context mut HashMap<NodeId, SpecNodeTypeInformation>,
    ) -> &'context SpecNodeTypeInformation {
        if deduced_types.contains_key(&node_id) {
            return &deduced_types.get(&node_id).unwrap();
        };
        let funclet = self.get_funclet(&funclet_id);
        let node = self.get_node(Location::new(funclet_id, node_id));
        let hole_error = format!(
            "Invalid hole in {:?}, cannot have an explication hole in a spec funclet",
            self.debug_info.node(&funclet_id, node_id)
        );
        let error_text = format!(
            ": in spec node {}",
            self.debug_info.node(&funclet_id, node_id)
        );
        let error = |text: &str| format!("{}: {}", text, error_text);
        let result = match node {
            expir::Node::Phi { index } => {
                let input_types = vec![];
                let output_types = vec![get_expect_box(
                    &funclet.input_types,
                    index.as_ref().opt().expect(&hole_error).clone(),
                )
                .clone()];
                SpecNodeTypeInformation {
                    input_types,
                    output_types,
                }
            }
            expir::Node::ExtractResult { node_id, index } => {
                let input_types = vec![];
                let output_types = vec![self
                    .deduce_type(
                        funclet_id,
                        *node_id.as_ref().opt().expect(&hole_error),
                        node_dependencies,
                        deduced_types,
                    )
                    .output_types
                    .get(index.as_ref().opt().expect(&hole_error).clone())
                    .expect(&format!(
                        "Not enough arguments to extract index {} from {:?}",
                        index, node_id
                    ))
                    .clone()];
                SpecNodeTypeInformation {
                    input_types,
                    output_types,
                }
            }
            expir::Node::Constant { value, type_id } => {
                let input_types = vec![];
                let output_types = vec![type_id.as_ref().opt().expect(&hole_error).clone()];
                SpecNodeTypeInformation {
                    input_types,
                    output_types,
                }
            }
            expir::Node::CallFunctionClass {
                function_id,
                arguments,
            } => {
                let function_class = self
                    .program
                    .function_classes
                    .get(function_id.as_ref().opt().expect(&hole_error).clone())
                    .expect(&error(&format!("Unknown function class {}", &function_id)));

                let input_types = function_class.input_types.iter().cloned().collect();
                let output_types = function_class.output_types.iter().cloned().collect();
                SpecNodeTypeInformation {
                    input_types,
                    output_types,
                }
            }
            expir::Node::Select {
                condition,
                true_case,
                false_case,
            } => {
                let condition_type = self.expect_one_output_type(
                    funclet_id,
                    condition.as_ref().opt().expect(&hole_error).clone(),
                    node_dependencies,
                    deduced_types,
                );
                let true_type = self.expect_one_output_type(
                    funclet_id,
                    true_case.as_ref().opt().expect(&hole_error).clone(),
                    node_dependencies,
                    deduced_types,
                );
                let false_type = self.expect_one_output_type(
                    funclet_id,
                    false_case.as_ref().opt().expect(&hole_error).clone(),
                    node_dependencies,
                    deduced_types,
                );
                assert!(
                    true_type == false_type,
                    error("Expected matching condition types")
                );
                let input_types = vec![
                    condition_type.clone(),
                    true_type.clone(),
                    false_type.clone(),
                ];
                let output_types = vec![true_type.clone()];
                SpecNodeTypeInformation {
                    input_types,
                    output_types,
                }
            }
            expir::Node::EncodingEvent {
                local_past,
                remote_local_pasts,
            } => {
                let local_past_type = self.expect_one_output_type(
                    funclet_id,
                    local_past.as_ref().opt().expect(&hole_error).clone(),
                    node_dependencies,
                    deduced_types,
                );
                let input_types = vec![];
                let output_types = vec![local_past_type.clone(), local_past_type.clone()];
                SpecNodeTypeInformation {
                    input_types,
                    output_types,
                }
            }
            expir::Node::SubmissionEvent { local_past } => {
                let local_past_type = self.expect_one_output_type(
                    funclet_id,
                    local_past.as_ref().opt().expect(&hole_error).clone(),
                    node_dependencies,
                    deduced_types,
                );
                let input_types = vec![];
                let output_types = vec![local_past_type.clone()];
                SpecNodeTypeInformation {
                    input_types,
                    output_types,
                }
            }
            expir::Node::SynchronizationEvent {
                local_past,
                remote_local_past,
            } => {
                let local_past_type = self.expect_one_output_type(
                    funclet_id,
                    local_past.as_ref().opt().expect(&hole_error).clone(),
                    node_dependencies,
                    deduced_types,
                );
                let input_types = vec![];
                let output_types = vec![local_past_type.clone()];
                SpecNodeTypeInformation {
                    input_types,
                    output_types,
                }
            }
            expir::Node::SeparatedBufferSpaces { count, space } => {
                let space_type = self.expect_one_output_type(
                    funclet_id,
                    space.as_ref().opt().expect(&hole_error).clone(),
                    node_dependencies,
                    deduced_types,
                );
                let input_types = vec![];
                let output_types = vec![space_type]
                    .iter()
                    .cycle()
                    .take(count.as_ref().opt().expect(&hole_error).clone() + 1)
                    .cloned()
                    .collect();
                SpecNodeTypeInformation {
                    input_types,
                    output_types,
                }
            }
            _ => unreachable!("Not a spec node {:?}", node),
        };
        deduced_types.insert(node_id, result);
        &deduced_types.get(&node_id).unwrap()
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

    pub fn get_funclet(&self, funclet_id: &FuncletId) -> &expir::Funclet {
        self.program().funclets.get(*funclet_id).expect(&format!(
            "Invalid funclet index {} for funclets {:?} corresponding with funclet {}",
            funclet_id,
            &self.program().funclets,
            self.debug_info.funclet(funclet_id)
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

    pub fn get_node_dependencies(&self, funclet_id: &FuncletId, node_id: &NodeId) -> &Vec<NodeId> {
        &self
            .get_spec_data(funclet_id)
            .node_dependencies
            .get(node_id)
            .expect(&format!(
                "No dependency information found for node {}",
                self.debug_info.node(funclet_id, node_id.clone())
            ))
    }

    pub fn get_tail_edge_dependencies(&self, funclet_id: &FuncletId) -> &Vec<NodeId> {
        &self.get_spec_data(funclet_id).tail_dependencies
    }

    pub fn get_node_type_information(
        &self,
        funclet_id: &FuncletId,
        node_id: &NodeId,
    ) -> &SpecNodeTypeInformation {
        &self
            .get_spec_data(funclet_id)
            .deduced_types
            .get(node_id)
            .expect(&format!(
                "No deduced types found for node {}",
                self.debug_info.node(funclet_id, node_id.clone())
            ))
    }
}
