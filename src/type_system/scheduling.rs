use crate::ir;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::default::Default;
use super::spec_checker::*;
use super::error::Error;

#[derive(Debug)]
struct Slot {
    storage_type: ir::ffi::TypeId,
    //queue_stage: ir::ResourceQueueStage,
    queue_place: ir::Place,
}

#[derive(Debug)]
struct Encoder {
    queue_place: ir::Place,
}

#[derive(Debug)]
struct Fence {
    queue_place: ir::Place,
}

#[derive(Debug)]
enum JoinKind {
    Default,
    Inline,
    Serialized,
}

#[derive(Debug)]
struct JoinPoint {
    input_types: Box<[ir::TypeId]>,
    join_kind: JoinKind,
}

#[derive(Debug)]
struct Buffer {
    storage_place: ir::Place,
    static_layout_opt: Option<ir::StaticBufferLayout>,
}

#[derive(Debug)]
enum NodeType {
    Slot(Slot),
    Encoder(Encoder),
    Fence(Fence),
    JoinPoint,
    Buffer(Buffer),
}

impl NodeType {
    fn storage_type(&self) -> Option<ir::ffi::TypeId> {
        match self {
            NodeType::Slot(slot) => Some(slot.storage_type),
            NodeType::Encoder(_) => None,
            NodeType::Fence(_) => None,
            NodeType::JoinPoint => None,
            NodeType::Buffer(_) => None,
        }
    }
}

fn check_slot_type(program: &ir::Program, type_id: ir::TypeId, node_type: &NodeType) {
    match &program.types[type_id] {
        ir::Type::Slot {
            storage_type: storage_type_2,
            //queue_stage: queue_stage_2,
            queue_place: queue_place_2,
        } => {
            if let NodeType::Slot(Slot {
                storage_type,
                //queue_stage,
                queue_place,
            }) = node_type
            {
                assert_eq!(*queue_place_2, *queue_place);
                //assert_eq!(*queue_stage_2, *queue_stage);
                assert_eq!(*storage_type, *storage_type_2);
            } else {
                panic!("type id is a slot type, but node is not a slot");
            }
        }
        ir::Type::Fence {
            queue_place: queue_place_2,
        } => {
            if let NodeType::Fence(Fence { queue_place }) = node_type {
                assert_eq!(*queue_place_2, *queue_place);
            } else {
                panic!("type id is a fence type, but node is not a fence");
            }
        }
        _ => panic!("Not a slot type"),
    }
}

fn advance_forward_value_copy<'program>(value_spec_checker : &mut FuncletSpecChecker<'program>, input_impl_node_id : ir::NodeId, output_impl_node_id : ir::NodeId) -> Result<(), Error> {
    let scalar = & value_spec_checker.scalar_nodes[& input_impl_node_id];
    assert!(scalar.flow.is_readable());
    assert!(value_spec_checker.can_drop_node(output_impl_node_id));
    value_spec_checker.update_scalar_node(output_impl_node_id, scalar.quot, ir::Flow::Have);
    return Ok(());
}

fn advance_forward_value_do<'program>(value_spec_checker : &mut FuncletSpecChecker<'program>, spec_node_id : ir::NodeId, input_impl_node_ids : &[ir::NodeId], output_impl_node_ids : &[ir::NodeId]) -> Result<(), Error> {
    // Can only advance if output flow is either have or none

    let encoded_node = & value_spec_checker.spec_funclet.nodes[spec_node_id];
    match encoded_node {
        ir::Node::Constant { .. } => {
            // Inputs
            assert_eq!(input_impl_node_ids.len(), 0);
            // Outputs
            assert_eq!(output_impl_node_ids.len(), 1);
            assert!(value_spec_checker.can_drop_node(output_impl_node_ids[0]));
            value_spec_checker.update_scalar_node(output_impl_node_ids[0], ir::Quotient::Node{node_id: spec_node_id}, ir::Flow::Have);
        }
        ir::Node::Select {
            condition,
            true_case,
            false_case,
        } => {
            // Inputs
            assert_eq!(input_impl_node_ids.len(), 3);
            for (input_index, input_spec_node_id) in [*condition, *true_case, *false_case].iter().enumerate()
            {
                let scalar = & value_spec_checker.scalar_nodes[& input_impl_node_ids[input_index]];
                assert_eq!(scalar.quot, ir::Quotient::Node{node_id: *input_spec_node_id});
                assert!(scalar.flow.is_readable());
            }
            // Outputs
            assert_eq!(output_impl_node_ids.len(), 1);
            assert!(value_spec_checker.can_drop_node(output_impl_node_ids[0]));
            value_spec_checker.update_scalar_node(output_impl_node_ids[0], ir::Quotient::Node{node_id: spec_node_id}, ir::Flow::Have);
        }
        ir::Node::CallFunctionClass {
            function_id,
            arguments,
        } => {
            // Inputs
            assert_eq!(input_impl_node_ids.len(), arguments.len());
            for (input_index, input_spec_node_id) in arguments.iter().enumerate()
            {
                let scalar = & value_spec_checker.scalar_nodes[& input_impl_node_ids[input_index]];
                
                assert_eq!(scalar.quot, ir::Quotient::Node{node_id: *input_spec_node_id});
                assert!(scalar.flow.is_readable());
            }
            // Outputs
            for (output_index, output_impl_node_id) in output_impl_node_ids.iter().enumerate()
            {
                // To do: Check that spec node is really an extractresult
                assert!(value_spec_checker.can_drop_node(*output_impl_node_id));
                value_spec_checker.update_scalar_node(*output_impl_node_id, ir::Quotient::Node{node_id: spec_node_id + 1 + output_index}, ir::Flow::Have);
            }
        }
        _ => panic!("Unsupported node: {:?}", encoded_node)
    }

    return Ok(());
}

fn advance_forward_timeline<'program>(timeline_spec_checker : &mut FuncletSpecChecker<'program>, spec_node_id : ir::NodeId, input_impl_node_ids : &[ir::NodeId], output_impl_node_ids : &[ir::NodeId]) -> Result<(), Error> {
    let encoded_node = & timeline_spec_checker.spec_funclet.nodes[spec_node_id];
    match encoded_node {
        ir::Node::EncodingEvent { local_past } => {
            assert_eq!(output_impl_node_ids.len(), 1);
            /*match timeline_spec_checker.current_implicit_tag {
                ir::Tag::Node{node_id : local_past_node_id} => {
                    assert_eq!(local_past_node_id, *local_past);
                }
                _ => panic!("Tag must be Node")
            }*/
            let from_tag = ir::Tag{quot: ir::Quotient::Node{node_id: *local_past}, flow: ir::Flow::Have};
            timeline_spec_checker.check_implicit_tag(from_tag)?;
            timeline_spec_checker.transition_state_subset_forwards(input_impl_node_ids, *local_past, spec_node_id + 2)?;
            timeline_spec_checker.transition_state_forwards(*local_past, spec_node_id + 1)?;
            timeline_spec_checker.update_scalar_node(output_impl_node_ids[0], ir::Quotient::Node{node_id: spec_node_id + 2}, ir::Flow::Have);
        }
        ir::Node::SubmissionEvent { local_past } => {
            assert_eq!(input_impl_node_ids.len(), 1);
            assert_eq!(output_impl_node_ids.len(), 1);

            let from_tag = ir::Tag{quot: ir::Quotient::Node{node_id: *local_past}, flow: ir::Flow::Have};
            // Use encoder
            timeline_spec_checker.check_node_tag(input_impl_node_ids[0], from_tag)?;
            timeline_spec_checker.update_scalar_node(input_impl_node_ids[0], ir::Quotient::Node{node_id: *local_past}, ir::Flow::None);

            //timeline_spec_checker.check_implicit_tag(from_tag)?;
            timeline_spec_checker.transition_state_forwards(*local_past, spec_node_id)?;
            timeline_spec_checker.update_scalar_node(output_impl_node_ids[0], ir::Quotient::Node{node_id: spec_node_id}, ir::Flow::Have);
        }
        ir::Node::SynchronizationEvent { local_past, remote_local_past } => {
            assert_eq!(input_impl_node_ids.len(), 1);
            assert_eq!(output_impl_node_ids.len(), 0);

            let from_tag = ir::Tag{quot: ir::Quotient::Node{node_id: *local_past}, flow: ir::Flow::Have};
            let remote_from_tag = ir::Tag{quot: ir::Quotient::Node{node_id: *remote_local_past}, flow: ir::Flow::Have};
            timeline_spec_checker.check_implicit_tag(from_tag)?;

            // Use fence
            timeline_spec_checker.check_node_tag(input_impl_node_ids[0], remote_from_tag)?;
            timeline_spec_checker.update_scalar_node(input_impl_node_ids[0], ir::Quotient::Node{node_id: *remote_local_past}, ir::Flow::None);

            timeline_spec_checker.transition_state_forwards(*local_past, spec_node_id)?;
            //timeline_spec_checker.transition_state_forwards(remote_from_tag, spec_node_id + 2);
            //timeline_spec_checker.transition_state_forwards(from_tag, spec_node_id + 1);
        }
        _ => panic!("Unsupported node: {:?}", encoded_node)
    }

    return Ok(());
}

#[derive(Debug)]
pub struct FuncletChecker<'program> {
    program: &'program ir::Program,
    value_funclet_id: ir::FuncletId,
    value_funclet: &'program ir::Funclet,
    scheduling_funclet: &'program ir::Funclet,
    value_spec: &'program ir::FuncletSpec,
    spatial_spec: &'program ir::FuncletSpec,
    timeline_spec: &'program ir::FuncletSpec,
    value_spec_checker_opt : Option<FuncletSpecChecker<'program>>,
    timeline_spec_checker_opt : Option<FuncletSpecChecker<'program>>,
    spatial_spec_checker_opt : Option<FuncletSpecChecker<'program>>,
    node_join_points: HashMap<ir::NodeId, JoinPoint>,
    node_types: HashMap<ir::NodeId, NodeType>,
    current_node_id: ir::NodeId,
}

impl<'program> FuncletChecker<'program> {
    pub fn new(program: &'program ir::Program, scheduling_funclet: &'program ir::Funclet) -> Self {
        assert_eq!(scheduling_funclet.kind, ir::FuncletKind::ScheduleExplicit);
        let value_spec = scheduling_funclet.spec_binding.get_value_spec();
        let spatial_spec = scheduling_funclet.spec_binding.get_spatial_spec();
        let timeline_spec = scheduling_funclet.spec_binding.get_timeline_spec();
        let value_funclet = &program.funclets[value_spec.funclet_id_opt.unwrap()];
        assert_eq!(value_funclet.kind, ir::FuncletKind::Value);
        let mut state = Self {
            program,
            value_funclet_id: value_spec.funclet_id_opt.unwrap(),
            value_funclet,
            scheduling_funclet,
            value_spec,
            spatial_spec,
            timeline_spec,
            value_spec_checker_opt: Some(FuncletSpecChecker::new(program, value_funclet, value_spec)),
            timeline_spec_checker_opt: Some(FuncletSpecChecker::new(program, &program.funclets[timeline_spec.funclet_id_opt.unwrap()], timeline_spec)),
            spatial_spec_checker_opt: Some(FuncletSpecChecker::new(program, &program.funclets[spatial_spec.funclet_id_opt.unwrap()], spatial_spec)),
            node_join_points: HashMap::new(),
            node_types: HashMap::new(),
            current_node_id: 0,
        };
        state.initialize();
        state
    }

    fn initialize(&mut self) {
        for (index, input_type_id) in self.scheduling_funclet.input_types.iter().enumerate() {
            let is_valid = match &self.scheduling_funclet.nodes[index] {
                //ir::Node::None => true,
                ir::Node::Phi { .. } => true,
                _ => false,
            };
            assert!(is_valid);

            let node_type = match &self.program.types[*input_type_id] {
                ir::Type::Slot {
                    storage_type,
                    //queue_stage,
                    queue_place,
                } => NodeType::Slot(Slot {
                    storage_type: *storage_type,
                    //queue_stage: *queue_stage,
                    queue_place: *queue_place,
                }),
                ir::Type::Fence { queue_place } => NodeType::Fence(Fence {
                    queue_place: *queue_place,
                }),
                ir::Type::Buffer {
                    storage_place,
                    static_layout_opt,
                } => NodeType::Buffer(Buffer {
                    storage_place: *storage_place,
                    static_layout_opt: *static_layout_opt,
                }),
                _ => panic!("Not a legal argument type for a scheduling funclet"),
            };

            self.node_types.insert(index, node_type);
        }

        /*for (output_index, output_type) in self.scheduling_funclet.output_types.iter().enumerate() {
            let (value_tag, timeline_tag, spatial_tag) = (
                self.value_spec.output_tags[output_index],
                self.timeline_spec.output_tags[output_index],
                self.spatial_spec.output_tags[output_index],
            );

            match &self.program.types[*output_type] {
                ir::Type::Slot { queue_place, .. } => {
                    // Local is the only place where data can be passed out of the function directly by value
                    if spatial_tag == ir::SpatialTag::None {
                        assert_eq!(*queue_place, ir::Place::Local);
                    }
                }
                ir::Type::Fence { queue_place } => {
                    panic!("Returning fences is currently unimplemented")
                }
                ir::Type::Buffer {
                    storage_place,
                    static_layout_opt,
                } => {
                    assert!(spatial_tag != ir::SpatialTag::None);
                }
                _ => (),
            }
        }*/
    }

    fn get_funclet_value_spec<'funclet>(
        &self,
        funclet: &'funclet ir::Funclet,
    ) -> &'funclet ir::FuncletSpec {
        if let ir::FuncletSpecBinding::ScheduleExplicit {
            value,
            spatial,
            timeline,
        } = &funclet.spec_binding
        {
            value
        } else {
            panic!("Does not have a ScheduleExplicit spec binding")
        }
    }

    fn get_funclet_timeline_spec<'funclet>(
        &self,
        funclet: &'funclet ir::Funclet,
    ) -> &'funclet ir::FuncletSpec {
        if let ir::FuncletSpecBinding::ScheduleExplicit {
            value,
            spatial,
            timeline,
        } = &funclet.spec_binding
        {
            timeline
        } else {
            panic!("Does not have a ScheduleExplicit spec binding")
        }
    }

    fn get_funclet_spatial_spec<'funclet>(
        &self,
        funclet: &'funclet ir::Funclet,
    ) -> &'funclet ir::FuncletSpec {
        if let ir::FuncletSpecBinding::ScheduleExplicit {
            value,
            spatial,
            timeline,
        } = &funclet.spec_binding
        {
            spatial
        } else {
            panic!("Does not have a ScheduleExplicit spec binding")
        }
    }

    fn can_drop_node(&self, node_id : ir::NodeId) -> bool {
        self.value_spec_checker_opt.as_ref().unwrap().can_drop_node(node_id) || 
        self.timeline_spec_checker_opt.as_ref().unwrap().can_drop_node(node_id) ||
        self.spatial_spec_checker_opt.as_ref().unwrap().can_drop_node(node_id)
    }

    fn is_neutral_node(&self, node_id : ir::NodeId) -> bool {
        self.value_spec_checker_opt.as_ref().unwrap().is_neutral_node(node_id) || 
        self.timeline_spec_checker_opt.as_ref().unwrap().is_neutral_node(node_id) ||
        self.spatial_spec_checker_opt.as_ref().unwrap().is_neutral_node(node_id)
    }

    fn drop_node(&mut self, node_id : ir::NodeId) {
        self.value_spec_checker_opt.as_mut().unwrap().drop_node(node_id);
        self.timeline_spec_checker_opt.as_mut().unwrap().drop_node(node_id);
        self.spatial_spec_checker_opt.as_mut().unwrap().drop_node(node_id);
    }

    fn contextualize_error(&self, error : Error) -> Error {
        match error {
            Error::Unknown{message} => Error::Unknown{message},
            Error::Generic{message} => Error::Generic{message: format!("Node #{} ({:?}):\n\t{}\n--- Funclet ---\n{:?}\n", self.current_node_id, & self.scheduling_funclet.nodes[self.current_node_id], message, self.scheduling_funclet)}
        }
    }

    /*fn report_if_error<T>(&self, result : Result<T, Error>) -> Result<T, Error> {
        Ok(result?)
    }*/

    pub fn check_next_node(&mut self, current_node_id: ir::NodeId) -> Result<(), Error> {
        assert_eq!(self.current_node_id, current_node_id);
        let current_node = &self.scheduling_funclet.nodes[current_node_id];
        match current_node {
            ir::Node::None => (),
            ir::Node::Phi { .. } => (),
            ir::Node::ExtractResult { node_id, index } => (),
            ir::Node::AllocTemporary {
                place,
                storage_type,
                operation,
            } => {
                self.value_spec_checker_opt.as_mut().unwrap().update_scalar_node(current_node_id, ir::Quotient::Node{node_id: operation.node_id}, ir::Flow::Have);
                self.timeline_spec_checker_opt.as_mut().unwrap().update_scalar_node(current_node_id, ir::Quotient::None, ir::Flow::Have);
                self.spatial_spec_checker_opt.as_mut().unwrap().update_scalar_node(current_node_id, ir::Quotient::None, ir::Flow::Have);
                self.node_types.insert(
                    current_node_id,
                    NodeType::Slot(Slot {
                        storage_type: *storage_type,
                        //queue_stage: ir::ResourceQueueStage::Bound,
                        queue_place: *place,
                    }),
                );
            }
            ir::Node::Drop {
                node: dropped_node_id,
            } => {
                if let Some(node_type) = self.node_types.remove(dropped_node_id) {
                    assert!(self.can_drop_node(*dropped_node_id));
                } else {
                    panic!("No node at #{}", dropped_node_id)
                }
                self.drop_node(*dropped_node_id);
            }
            ir::Node::LocalDoBuiltin {
                operation,
                inputs,
                outputs,
            } => {
                assert_eq!(
                    self.value_spec.funclet_id_opt.unwrap(),
                    operation.funclet_id
                );

                advance_forward_value_do(self.value_spec_checker_opt.as_mut().unwrap(), operation.node_id, inputs, outputs).map_err(|e| self.contextualize_error(e))?;

                // To do: Check timeline and spatial
            }
            ir::Node::LocalDoExternal {
                operation,
                external_function_id,
                inputs,
                outputs,
            } => {
                assert_eq!(
                    self.value_spec.funclet_id_opt.unwrap(),
                    operation.funclet_id
                );

                advance_forward_value_do(self.value_spec_checker_opt.as_mut().unwrap(), operation.node_id, inputs, outputs).map_err(|e| self.contextualize_error(e))?;
                // To do: Check timeline and spatial

                /*let encoded_funclet = &self.program.funclets[operation.funclet_id];
                let encoded_node = &encoded_funclet.nodes[operation.node_id];

                match encoded_node {
                    ir::Node::CallValueFunction {
                        function_id,
                        arguments,
                    } => {
                        assert!(self.program.function_classes[*function_id].external_function_ids.contains(external_function_id));
                        let function = &self.program.native_interface.external_functions
                            [external_function_id.0];
                        let cpu_operation = function.get_cpu_pure_operation().unwrap();

                        assert_eq!(inputs.len(), arguments.len());
                        assert_eq!(inputs.len(), cpu_operation.input_types.len());
                        assert_eq!(outputs.len(), cpu_operation.output_types.len());

                        for (input_index, input_node_id) in arguments.iter().enumerate() {
                            assert_eq!(
                                self.node_types[&inputs[input_index]]
                                    .storage_type()
                                    .unwrap(),
                                cpu_operation.input_types[input_index]
                            );

                            let value_tag = self.value_spec_checker_opt.as_mut().unwrap().scalar_nodes[&inputs[input_index]].tag;
                            let funclet_id = self.value_funclet_id;
                            check_value_tag_compatibility_interior(
                                &self.program,
                                Some(funclet_id),
                                value_tag,
                                ir::ValueTag::Node {
                                    node_id: *input_node_id,
                                },
                            );
                        }

                        for (index, output_type_id) in cpu_operation.output_types.iter().enumerate()
                        {
                            /*self.transition_slot(
                                outputs[index],
                                ir::Place::Local,
                                &[(ir::ResourceQueueStage::Bound, ir::ResourceQueueStage::Ready)],
                            );*/
                        }
                    }
                    _ => panic!("Node is not supported with LocalDoExternal: {:?}", encoded_node)
                }

                self.check_do_output(operation, encoded_funclet, encoded_node, outputs);*/
            }
            ir::Node::EncodeDoExternal {
                operation,
                external_function_id,
                inputs,
                outputs,
                encoder,
            } => {
                assert_eq!(
                    self.value_spec.funclet_id_opt.unwrap(),
                    operation.funclet_id
                );

                //assert_eq!(*place, ir::Place::Gpu);
                
                advance_forward_value_do(self.value_spec_checker_opt.as_mut().unwrap(), operation.node_id, inputs, outputs).map_err(|e| self.contextualize_error(e))?;
                // To do: Check timeline and spatial

                /*let encoded_funclet = &self.program.funclets[operation.funclet_id];
                let encoded_node = &encoded_funclet.nodes[operation.node_id];

                match encoded_node {
                    ir::Node::CallValueFunction {
                        function_id,
                        arguments,
                        //dimensions,
                    } => {
                        assert_eq!(*place, ir::Place::Gpu);
                        assert!(self.program.function_classes[*function_id].external_function_ids.contains(external_function_id));

                        let function = &self.program.native_interface.external_functions
                            [external_function_id.0];
                        let kernel = function.get_gpu_kernel().unwrap();

                        assert_eq!(inputs.len(), arguments.len());
                        assert_eq!(outputs.len(), kernel.output_types.len());

                        for (input_index, input_node_id) in
                            arguments.iter().enumerate()
                        {
                            let value_tag = self.value_spec_checker_opt.as_mut().unwrap().scalar_nodes[&inputs[input_index]].tag;
                            let funclet_id = self.value_funclet_id;
                            check_value_tag_compatibility_interior(
                                &self.program,
                                Some(funclet_id),
                                value_tag,
                                ir::ValueTag::Node {
                                    node_id: *input_node_id,
                                },
                            );
                        }

                        ir::validation::validate_gpu_kernel_bindings(
                            kernel,
                            &inputs[kernel.dimensionality..],
                            outputs,
                        );

                        let mut forwarding_input_scheduling_node_ids = HashSet::<ir::NodeId>::new();
                        let mut forwarded_output_scheduling_node_ids = HashSet::<ir::NodeId>::new();
                        for (input_index, _) in arguments[kernel.dimensionality..].iter().enumerate() {
                            assert_eq!(
                                self.node_types[&inputs[kernel.dimensionality + input_index]]
                                    .storage_type()
                                    .unwrap(),
                                kernel.input_types[input_index]
                            );

                            if let Some(forwarded_output_index) =
                                kernel.output_of_forwarding_input(input_index)
                            {
                                /*let transitions = [
                                    (
                                        ir::ResourceQueueStage::Encoded,
                                        ir::ResourceQueueStage::Encoded,
                                    ),
                                    (
                                        ir::ResourceQueueStage::Submitted,
                                        ir::ResourceQueueStage::Encoded,
                                    ),
                                    (
                                        ir::ResourceQueueStage::Ready,
                                        ir::ResourceQueueStage::Encoded,
                                    ),
                                ];
                                self.forward_slot(
                                    inputs[input_index],
                                    outputs[forwarded_output_index],
                                    *place,
                                    &transitions,
                                );*/

                                forwarding_input_scheduling_node_ids.insert(inputs[input_index]);
                                forwarded_output_scheduling_node_ids
                                    .insert(outputs[forwarded_output_index]);
                            }
                        }

                        //output_of_forwarding_input

                        // To do: Input checks
                        for input_scheduling_node_id in inputs[kernel.dimensionality..].iter() {
                            let is_forwarding = forwarding_input_scheduling_node_ids
                                .contains(input_scheduling_node_id);
                            if !is_forwarding {
                                /*let transitions = [
                                    (
                                        ir::ResourceQueueStage::Encoded,
                                        ir::ResourceQueueStage::Encoded,
                                    ),
                                    (
                                        ir::ResourceQueueStage::Submitted,
                                        ir::ResourceQueueStage::Encoded,
                                    ),
                                    (
                                        ir::ResourceQueueStage::Ready,
                                        ir::ResourceQueueStage::Encoded,
                                    ),
                                ];
                                self.transition_slot(
                                    *input_scheduling_node_id,
                                    *place,
                                    &transitions,
                                );*/
                            }
                        }

                        for (index, output_type_id) in kernel.output_types.iter().enumerate() {
                            assert_eq!(
                                self.node_types[&outputs[index]].storage_type().unwrap(),
                                kernel.output_types[index]
                            );

                            let is_forwarded =
                                forwarded_output_scheduling_node_ids.contains(&outputs[index]);
                            if !is_forwarded {
                                /*self.transition_slot(
                                    outputs[index],
                                    *place,
                                    &[(
                                        ir::ResourceQueueStage::Bound,
                                        ir::ResourceQueueStage::Encoded,
                                    )],
                                );*/
                            }
                        }
                    }
                    _ => panic!("Cannot encode {:?}", encoded_node),
                }

                self.check_do_output(operation, encoded_funclet, encoded_node, outputs);*/
            }
            ir::Node::LocalCopy {
                input,
                output,
            } => {
                advance_forward_value_copy(self.value_spec_checker_opt.as_mut().unwrap(), *input, *output).map_err(|e| self.contextualize_error(e))?;
            }
            ir::Node::EncodeCopy {
                input,
                output,
                encoder,
            } => {
                advance_forward_value_copy(self.value_spec_checker_opt.as_mut().unwrap(), *input, *output).map_err(|e| self.contextualize_error(e))?;
            }
            ir::Node::BeginEncoding { place, event, encoded } => {
                assert_eq!(
                    self.timeline_spec.funclet_id_opt.unwrap(),
                    event.funclet_id
                );

                let timeline_spec_checker = self.timeline_spec_checker_opt.as_mut().unwrap();
                let ir::Node::EncodingEvent{..} = & timeline_spec_checker.spec_funclet.nodes[event.node_id] else { panic!("Must be an encoding event") };
                advance_forward_timeline(timeline_spec_checker, event.node_id, &[], &[current_node_id]).map_err(|e| self.contextualize_error(e))?;

                self.node_types.insert(
                    current_node_id,
                    NodeType::Encoder(Encoder {
                        queue_place: *place,
                    }),
                );
            }
            ir::Node::Submit { event, encoder } => {
                assert_eq!(
                    self.timeline_spec.funclet_id_opt.unwrap(),
                    event.funclet_id
                );

                let Some(NodeType::Encoder(Encoder { queue_place })) = self.node_types.remove(encoder) else {
                    panic!("Not an encoder");
                };

                let timeline_spec_checker = self.timeline_spec_checker_opt.as_mut().unwrap();
                let ir::Node::SubmissionEvent{..} = & timeline_spec_checker.spec_funclet.nodes[event.node_id] else { panic!("Must be a submission event") };
                advance_forward_timeline(timeline_spec_checker, event.node_id, &[*encoder], &[current_node_id]).map_err(|e| self.contextualize_error(e))?;
                //self.value_spec_checker_opt.as_mut().unwrap().update_scalar_node(current_node_id, ir::Tag::None, ir::Flow::Have);
                //let implicit_tag = timeline_spec_checker.current_implicit_tag;
                //self.timeline_spec_checker_opt.as_mut().unwrap().update_scalar_node(current_node_id, implicit_tag, ir::Flow::Have);
                //self.spatial_spec_checker_opt.as_mut().unwrap().update_scalar_node(current_node_id, ir::Tag::None, ir::Flow::Have);

                self.node_types.insert(
                    current_node_id,
                    NodeType::Fence(Fence {
                        queue_place,
                    }),
                );
            }
            ir::Node::SyncFence {
                fence,
                event,
            } => {
                assert_eq!(
                    self.timeline_spec.funclet_id_opt.unwrap(),
                    event.funclet_id
                );

                let timeline_spec_checker = self.timeline_spec_checker_opt.as_mut().unwrap();
                let ir::Node::SynchronizationEvent{..} = & timeline_spec_checker.spec_funclet.nodes[event.node_id] else { panic!("Must be an synchronization event") };
                advance_forward_timeline(timeline_spec_checker, event.node_id, &[*fence], &[]).map_err(|e| self.contextualize_error(e))?;

                let fenced_place = if let Some(NodeType::Fence(Fence { queue_place })) =
                    &self.node_types.remove(fence)
                {
                    *queue_place
                } else {
                    panic!("Not a fence");
                };

                assert_eq!(fenced_place, ir::Place::Gpu);
            }
            /*ir::Node::StaticAllocFromStaticBuffer {
                buffer: buffer_node_id,
                place,
                storage_type,
                operation,
            } => {
                // Temporary restriction
                match *place {
                    ir::Place::Local => panic!("Unimplemented allocating locally"),
                    _ => {}
                }
                let buffer_spatial_tag = self.spatial_spec_checker_opt.as_mut().unwrap().scalar_nodes[buffer_node_id].tag;
                assert_ne!(buffer_spatial_tag, ir::SpatialTag::None);

                if let Some(NodeType::Buffer(Buffer {
                    storage_place,
                    static_layout_opt: Some(static_layout),
                })) = self.node_types.get_mut(buffer_node_id)
                {
                    // We might eventually separate storage places and queue places
                    assert_eq!(*storage_place, *place);
                    // To do check alignment compatibility
                    let storage_size = self
                        .program
                        .native_interface
                        .calculate_type_byte_size(*storage_type);
                    let alignment_bits = self
                        .program
                        .native_interface
                        .calculate_type_alignment_bits(*storage_type);
                    let starting_alignment_offset = 1usize << static_layout.alignment_bits;
                    let additional_alignment_offset =
                        if alignment_bits > static_layout.alignment_bits {
                            let alignment_offset = 1usize << alignment_bits;
                            alignment_offset - starting_alignment_offset
                        } else {
                            0usize
                        };
                    let total_byte_size = storage_size + additional_alignment_offset;

                    assert!(static_layout.byte_size >= total_byte_size);
                    static_layout.byte_size -= total_byte_size;
                    static_layout.alignment_bits =
                        (total_byte_size + starting_alignment_offset).trailing_zeros() as usize;

                    self.value_spec_checker_opt.as_mut().unwrap().update_scalar_node(current_node_id, ir::ValueTag::Node { node_id: operation.node_id }, ir::Flow::Have);
                    self.timeline_spec_checker_opt.as_mut().unwrap().update_scalar_node(current_node_id, ir::TimelineTag::None, ir::Flow::Have);
                    self.spatial_spec_checker_opt.as_mut().unwrap().update_scalar_node(current_node_id, buffer_spatial_tag, ir::Flow::Have);
                    self.node_types.insert(
                        current_node_id,
                        NodeType::Slot(Slot {
                            storage_type: *storage_type,
                            //queue_stage: ir::ResourceQueueStage::Bound,
                            queue_place: *place,
                        }),
                    );
                } else {
                    panic!("No static buffer at node #{}", buffer_node_id)
                }
            }*/
            ir::Node::DefaultJoin => {
                let mut input_types = Vec::<ir::TypeId>::new();
                for (index, type_id) in self.scheduling_funclet.output_types.iter().enumerate() {
                    input_types.push(*type_id);
                }
                //self.join_node_value_tags.insert(current_node_id, value_tags.into_boxed_slice());
                let join_point = JoinPoint {
                    join_kind: JoinKind::Default,
                    input_types: input_types.into_boxed_slice(),
                };
                self.value_spec_checker_opt.as_mut().unwrap().initialize_default_join(current_node_id);
                self.timeline_spec_checker_opt.as_mut().unwrap().initialize_default_join(current_node_id);
                self.spatial_spec_checker_opt.as_mut().unwrap().initialize_default_join(current_node_id);
                self.node_join_points.insert(current_node_id, join_point);
                self.node_types.insert(current_node_id, NodeType::JoinPoint);
            }
            ir::Node::InlineJoin {
                funclet: join_funclet_id,
                captures,
                continuation: continuation_join_node_id,
            } => self.handle_join(
                *join_funclet_id,
                captures,
                *continuation_join_node_id,
                JoinKind::Inline,
            ).map_err(|e| self.contextualize_error(e))?,
            ir::Node::SerializedJoin {
                funclet: join_funclet_id,
                captures,
                continuation: continuation_join_node_id,
            } => self.handle_join(
                *join_funclet_id,
                captures,
                *continuation_join_node_id,
                JoinKind::Serialized,
            ).map_err(|e| self.contextualize_error(e))?,
            _ => panic!("Unimplemented"),
        }

        self.current_node_id += 1;
        return Ok(());
    }

    fn handle_join(
        &mut self,
        join_funclet_id: ir::FuncletId,
        captures: &[ir::NodeId],
        continuation_join_node_id: ir::NodeId,
        join_kind: JoinKind,
    ) -> Result<(), Error> {
        let join_funclet = &self.program.funclets[join_funclet_id];
        let join_funclet_value_spec = &self.get_funclet_value_spec(join_funclet);
        let join_funclet_timeline_spec = &self.get_funclet_timeline_spec(join_funclet);
        let join_funclet_spatial_spec = &self.get_funclet_spatial_spec(join_funclet);
        let continuation_join_point = &self.node_join_points[&continuation_join_node_id];

        if let Some(NodeType::JoinPoint) = self.node_types.remove(&continuation_join_node_id) {
            // Nothing, for now...
        } else {
            panic!("Node at #{} is not a join point", continuation_join_node_id)
        }

        for (capture_index, capture_node_id) in captures.iter().enumerate() {
            let node_type = self.node_types.remove(capture_node_id).unwrap();
            check_slot_type(
                &self.program,
                join_funclet.input_types[capture_index],
                &node_type,
            );
        }

        let mut remaining_input_types = Vec::<ir::TypeId>::new();
        for input_index in captures.len()..join_funclet.input_types.len() {
            remaining_input_types.push(join_funclet.input_types[input_index]);
        }

        let continuation_join_input_types = &continuation_join_point.input_types;

        for (join_output_index, join_output_type) in join_funclet.output_types.iter().enumerate() {
            assert_eq!(
                *join_output_type,
                continuation_join_input_types[join_output_index]
            );
        }

        let join_point = JoinPoint {
            join_kind,
            input_types: remaining_input_types.into_boxed_slice(),
        };
        self.value_spec_checker_opt.as_mut().unwrap().join(self.current_node_id, captures, join_funclet_value_spec, continuation_join_node_id)?;
        self.timeline_spec_checker_opt.as_mut().unwrap().join(self.current_node_id, captures, join_funclet_timeline_spec, continuation_join_node_id)?;
        self.spatial_spec_checker_opt.as_mut().unwrap().join(self.current_node_id, captures, join_funclet_spatial_spec, continuation_join_node_id)?;
        self.node_join_points
            .insert(self.current_node_id, join_point);
        self.node_types
            .insert(self.current_node_id, NodeType::JoinPoint);
        
        return Ok(());
    }

    
    pub fn check_tail_edge(&mut self) -> Result<(), Error> {
        assert_eq!(self.current_node_id, self.scheduling_funclet.nodes.len());
        match &self.scheduling_funclet.tail_edge {
            ir::TailEdge::Return { return_values } => {
                for (return_index, return_node_id) in return_values.iter().enumerate() {
                    let Some(node_type) = self.node_types.remove(return_node_id) else {
                        return Err(Error::Generic{message: format!("Returning nonexistent node #{}. Was it already used?", return_node_id)});
                    };
                    check_slot_type(
                        &self.program,
                        self.scheduling_funclet.output_types[return_index],
                        &node_type,
                    );
                }

                self.value_spec_checker_opt.as_mut().unwrap().check_return(return_values).map_err(|e| self.contextualize_error(e))?;
                self.timeline_spec_checker_opt.as_mut().unwrap().check_return(return_values).map_err(|e| self.contextualize_error(e))?;
                self.spatial_spec_checker_opt.as_mut().unwrap().check_return(return_values).map_err(|e| self.contextualize_error(e))?;
            }
            /*ir::TailEdge::Yield {
                external_function_id,
                yielded_nodes: yielded_node_ids,
                next_funclet,
                continuation_join: continuation_join_node_id,
                arguments: argument_node_ids,
            } => {
                // To do: Need pipeline to check yield point types
                let continuation_join_point = &self.node_join_points[continuation_join_node_id];

                if let Some(NodeType::JoinPoint) = self.node_types.remove(continuation_join_node_id)
                {
                    // Nothing, for now...
                } else {
                    panic!("Node at #{} is not a join point", continuation_join_node_id)
                }

                for node_id in yielded_node_ids.iter() {
                    self.node_types.remove(node_id);
                }

                for argument_node_id in argument_node_ids.iter() {
                    self.node_types.remove(argument_node_id);
                }

                /*assert_eq!(true_funclet.output_types[output_index], continuation_join_point.input_types[output_index]);
                assert_eq!(false_funclet.output_types[output_index], continuation_join_point.input_types[output_index]);
                for (return_index, return_node_id) in return_values.iter().enumerate()
                {
                    check_slot_type(& self.program, true_funclet.input_types[argument_index], & node_type);
                }*/
            }*/
            ir::TailEdge::Jump { join, arguments } => {
                let join_point = &self.node_join_points[join];

                if let Some(NodeType::JoinPoint) = self.node_types.remove(join) {
                    // Nothing, for now...
                } else {
                    panic!("Node at #{} is not a join point", join)
                }

                for (argument_index, argument_node_id) in arguments.iter().enumerate() {
                    let node_type = self.node_types.remove(argument_node_id).unwrap();
                    check_slot_type(
                        &self.program,
                        join_point.input_types[argument_index],
                        &node_type,
                    );
                }

                self.value_spec_checker_opt.as_mut().unwrap().check_jump(*join, arguments).map_err(|e| self.contextualize_error(e))?;
                self.timeline_spec_checker_opt.as_mut().unwrap().check_jump(*join, arguments).map_err(|e| self.contextualize_error(e))?;
                self.spatial_spec_checker_opt.as_mut().unwrap().check_jump(*join, arguments).map_err(|e| self.contextualize_error(e))?;
            }
            ir::TailEdge::ScheduleCall {
                value_operation: value_operation_ref,
                callee_funclet_id: callee_scheduling_funclet_id_ref,
                callee_arguments,
                continuation_join: continuation_join_node_id,
            } => {
                let value_operation = *value_operation_ref;
                let callee_scheduling_funclet_id = *callee_scheduling_funclet_id_ref;
                let continuation_join_point = &self.node_join_points[continuation_join_node_id];

                if let Some(NodeType::JoinPoint) = self.node_types.remove(continuation_join_node_id)
                {
                    // Nothing, for now...
                } else {
                    panic!("Node at #{} is not a join point", continuation_join_node_id)
                }

                assert_eq!(value_operation.funclet_id, self.value_funclet_id);

                let callee_funclet = &self.program.funclets[callee_scheduling_funclet_id];
                assert_eq!(callee_funclet.kind, ir::FuncletKind::ScheduleExplicit);

                let value_spec = self.get_funclet_value_spec(callee_funclet);
				let callee_value_funclet_id = value_spec.funclet_id_opt.unwrap();
                assert_eq!(value_operation.funclet_id, callee_value_funclet_id);
				let callee_value_funclet = & self.program.funclets[callee_value_funclet_id];
				assert_eq!(callee_value_funclet.kind, ir::FuncletKind::Value);
                let timeline_spec = self.get_funclet_timeline_spec(callee_funclet);
                let spatial_spec = self.get_funclet_spatial_spec(callee_funclet);
                if let ir::Node::CallFunctionClass{function_id, arguments} = &callee_value_funclet.nodes[value_operation.node_id] {
                    self.value_spec_checker_opt.as_mut().unwrap().check_vertical_call(*continuation_join_node_id, callee_arguments, value_spec, arguments, value_operation.node_id).map_err(|e| self.contextualize_error(e))?;
                }
                else {
                    panic!("Not a call")
                };
                
                self.timeline_spec_checker_opt.as_mut().unwrap().check_interior_call(*continuation_join_node_id, callee_arguments, timeline_spec).map_err(|e| self.contextualize_error(e))?;
                self.spatial_spec_checker_opt.as_mut().unwrap().check_interior_call(*continuation_join_node_id, callee_arguments, spatial_spec).map_err(|e| self.contextualize_error(e))?;

                // Step 1: Check current -> callee edge
                for (argument_index, argument_node_id) in callee_arguments.iter().enumerate() {
                    let node_type = self.node_types.remove(argument_node_id).unwrap();
                    check_slot_type(
                        &self.program,
                        callee_funclet.input_types[argument_index],
                        &node_type,
                    );
                }

                // Step 2: Check callee -> continuation edge
                for (callee_output_index, callee_output_type) in
                    callee_funclet.output_types.iter().enumerate()
                {
                    assert_eq!(
                        *callee_output_type,
                        continuation_join_point.input_types[callee_output_index]
                    );
                }
            }
            ir::TailEdge::ScheduleSelect {
                value_operation,
                condition: condition_slot_node_id,
                callee_funclet_ids,
                callee_arguments,
                continuation_join: continuation_join_node_id,
            } => {
                assert_eq!(value_operation.funclet_id, self.value_funclet_id);
                let continuation_join_point = &self.node_join_points[continuation_join_node_id];

                if let Some(NodeType::JoinPoint) = self.node_types.remove(continuation_join_node_id)
                {
                    // Nothing, for now...
                } else {
                    panic!("Node at #{} is not a join point", continuation_join_node_id)
                }

                assert_eq!(callee_funclet_ids.len(), 2);
                let true_funclet_id = callee_funclet_ids[0];
                let false_funclet_id = callee_funclet_ids[1];
                let true_funclet = &self.program.funclets[true_funclet_id];
                let false_funclet = &self.program.funclets[false_funclet_id];
                let true_funclet_value_spec = &self.get_funclet_value_spec(true_funclet);
                let true_funclet_timeline_spec = &self.get_funclet_timeline_spec(true_funclet);
                let false_funclet_value_spec = &self.get_funclet_value_spec(false_funclet);
                let false_funclet_timeline_spec = &self.get_funclet_timeline_spec(false_funclet);

                let current_value_funclet = &self.program.funclets[value_operation.funclet_id];
                assert_eq!(current_value_funclet.kind, ir::FuncletKind::Value);

                let condition_value_tag = self.value_spec_checker_opt.as_mut().unwrap().scalar_nodes[condition_slot_node_id];

                assert_eq!(
                    value_operation.funclet_id,
                    true_funclet_value_spec.funclet_id_opt.unwrap()
                );
                assert_eq!(
                    value_operation.funclet_id,
                    true_funclet_value_spec.funclet_id_opt.unwrap()
                );

                assert_eq!(callee_arguments.len(), true_funclet.input_types.len());
                assert_eq!(callee_arguments.len(), false_funclet.input_types.len());

                if let ir::Node::Select{condition, true_case, false_case} = &current_value_funclet.nodes[value_operation.node_id] {
                    let cast_to_tag = ir::Quotient::Node{node_id: value_operation.node_id};
                    self.value_spec_checker_opt.as_mut().unwrap().check_choice(*continuation_join_node_id, callee_arguments, &[&[(*true_case, value_operation.node_id)], &[(*false_case, value_operation.node_id)]], &[true_funclet_value_spec, false_funclet_value_spec]).map_err(|e| self.contextualize_error(e))?;
                }
                else {
                    panic!("Not a select")
                };
                self.timeline_spec_checker_opt.as_mut().unwrap().check_choice(*continuation_join_node_id, callee_arguments, &[&[], &[]], &[true_funclet_value_spec, false_funclet_value_spec]).map_err(|e| self.contextualize_error(e))?;
                self.spatial_spec_checker_opt.as_mut().unwrap().check_choice(*continuation_join_node_id, callee_arguments, &[&[], &[]], &[true_funclet_value_spec, false_funclet_value_spec]).map_err(|e| self.contextualize_error(e))?;

                for (argument_index, argument_node_id) in callee_arguments.iter().enumerate() {
                    let node_type = self.node_types.remove(argument_node_id).unwrap();
                    check_slot_type(
                        &self.program,
                        true_funclet.input_types[argument_index],
                        &node_type,
                    );
                    check_slot_type(
                        &self.program,
                        false_funclet.input_types[argument_index],
                        &node_type,
                    );
                }

                assert_eq!(
                    true_funclet.output_types.len(),
                    continuation_join_point.input_types.len()
                );
                assert_eq!(
                    false_funclet.output_types.len(),
                    continuation_join_point.input_types.len()
                );
                for (output_index, _) in true_funclet.output_types.iter().enumerate() {
                    assert_eq!(
                        true_funclet.output_types[output_index],
                        continuation_join_point.input_types[output_index]
                    );
                    assert_eq!(
                        false_funclet.output_types[output_index],
                        continuation_join_point.input_types[output_index]
                    );
                }
            }
            _ => panic!("Unimplemented"),
        }

        // Enforce use of all nodes
        for (node_id, node_type) in self.node_types.iter() {
            assert!(self.can_drop_node(*node_id) || self.is_neutral_node(*node_id));
            //self.drop_node(*dropped_node_id)
        }

        return Ok(());
    }
}
