use super::error::{Error, ErrorContext};
use super::spec_checker::*;
use crate::ir;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::default::Default;

#[derive(Debug)]
struct LocalVar {
    storage_type: ir::ffi::TypeId,
}

#[derive(Debug)]
struct Slot {
    storage_type: ir::ffi::TypeId,
    queue_place: ir::Place,
    buffer_flags : ir::BufferFlags
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
    buffer_flags : ir::BufferFlags
}

impl Buffer {
    fn alloc_static(&mut self, native_interface : & ir::ffi::NativeInterface, error_context : & ErrorContext, storage_type : ir::StorageTypeId) -> Result<(), Error> {
        let Some(static_layout) = self.static_layout_opt.as_mut() else { panic!("{}", error_context) };
        /*// To do check alignment compatibility
        let storage_size = native_interface
            .calculate_type_byte_size(storage_type);
        let alignment_bits = native_interface
            .calculate_type_alignment_bits(storage_type);
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
            (total_byte_size + starting_alignment_offset).trailing_zeros() as usize;*/
        
        static_layout.alloc_static(native_interface, storage_type);
        
        return Ok(());
    }

    fn split_static(&mut self, native_interface : & ir::ffi::NativeInterface, error_context : & ErrorContext, size : usize) -> Result<ir::StaticBufferLayout, Error> {
        let Some(static_layout) = self.static_layout_opt.as_mut() else { panic!("{}", error_context) };

        /*let predecessor_static_layout = ir::StaticBufferLayout{byte_size : size, alignment_bits : static_layout.alignment_bits};

        assert!(static_layout.byte_size >= size);
        static_layout.byte_size -= size;
        let starting_alignment_offset = 1usize << static_layout.alignment_bits;
        static_layout.alignment_bits =
            (size + starting_alignment_offset).trailing_zeros() as usize;*/
        
        let predecessor_static_layout = static_layout.split_static(native_interface, size);

        return Ok(predecessor_static_layout);
    }

    fn merge_static_left(&mut self, native_interface : & ir::ffi::NativeInterface, error_context : & ErrorContext, predecessor_static_layout : ir::StaticBufferLayout) -> Result<(), Error> {
        let Some(static_layout) = self.static_layout_opt.as_mut() else { panic!("{}", error_context) };

        //static_layout.byte_size += predecessor_static_layout.byte_size;
        //static_layout.alignment_bits = predecessor_static_layout.alignment_bits;

        static_layout.merge_static_left(native_interface, predecessor_static_layout);
        
        return Ok(());
    }
}

#[derive(Debug)]
enum NodeType {
    LocalVar(LocalVar),
    Slot(Slot),
    Encoder(Encoder),
    Fence(Fence),
    JoinPoint,
    Buffer(Buffer),
}

impl NodeType {
    fn storage_type(&self) -> Option<ir::ffi::TypeId> {
        match self {
            NodeType::LocalVar(var) => Some(var.storage_type),
            NodeType::Slot(slot) => Some(slot.storage_type),
            NodeType::Encoder(_) => None,
            NodeType::Fence(_) => None,
            NodeType::JoinPoint => None,
            NodeType::Buffer(_) => None,
        }
    }
}

fn check_slot_type(program: &ir::Program, type_id: ir::TypeId, node_type: &NodeType, error_context : &ErrorContext) {
    match &program.types[type_id] {
        ir::Type::NativeValue {
            storage_type: storage_type_1,
        } => {
            if let NodeType::LocalVar(LocalVar {
                storage_type: storage_type_2,
            }) = node_type
            {
                assert_eq!(*storage_type_1, *storage_type_2, "\n{}", error_context);
            } else {
                panic!("type id is a native value type, but node is not a local variable\n{}", error_context);
            }
        }
        ir::Type::Ref {
            storage_type: storage_type_2,
            //queue_stage: queue_stage_2,
            storage_place: queue_place_2,
            buffer_flags: buffer_flags_2
        } => {
            if let NodeType::Slot(Slot {
                storage_type,
                //queue_stage,
                queue_place,
                buffer_flags
            }) = node_type
            {
                assert_eq!(*queue_place_2, *queue_place, "\n{}", error_context);
                //assert_eq!(*queue_stage_2, *queue_stage);
                assert_eq!(*storage_type, *storage_type_2, "\n{}", error_context);
                assert_eq!(*buffer_flags, *buffer_flags_2, "\n{}", error_context);
            } else {
                panic!("type id is a ref type, but node is not a ref\n{}", error_context);
            }
        }
        ir::Type::Fence {
            queue_place: queue_place_2,
        } => {
            if let NodeType::Fence(Fence { queue_place }) = node_type {
                assert_eq!(*queue_place_2, *queue_place, "\n{}", error_context);
            } else {
                panic!("type id is a fence type, but node is not a fence\n{}", error_context);
            }
        }
        _ => panic!("Unimplemented"),
    }
}

fn check_slot_storage_type(
    program: &ir::Program,
    storage_type_id: ir::ffi::TypeId,
    node_type: &NodeType,
) {
    // To do
}

fn advance_forward_value_copy<'program>(
    value_spec_checker: &mut FuncletSpecChecker<'program>,
    error_context: &ErrorContext,
    input_impl_node_id: ir::NodeId,
    output_impl_node_id: ir::NodeId,
) -> Result<(), Error> {
    let scalar = &value_spec_checker.scalar_nodes[&input_impl_node_id];
    assert!(scalar.flow.is_readable());
    //value_spec_checker.check_node_tag(output_impl_node_id, ir::Tag{quot: scalar.quot, flow: ir::Flow::None})?;
    assert!(value_spec_checker.can_drop_node(output_impl_node_id));
    value_spec_checker.update_scalar_node(output_impl_node_id, scalar.quot, ir::Flow::Have);
    return Ok(());
}

fn advance_forward_value_do<'program>(
    value_spec_checker: &mut FuncletSpecChecker<'program>,
    error_context: &ErrorContext,
    spec_node_id: ir::NodeId,
    input_impl_node_ids: &[ir::NodeId],
    output_impl_node_ids: &[ir::NodeId],
) -> Result<(), Error> {
    // Can only advance if output flow is either have or none

    let encoded_node = &value_spec_checker.spec_funclet.nodes[spec_node_id];
    match encoded_node {
        ir::Node::Constant { .. } => {
            // Inputs
            assert_eq!(input_impl_node_ids.len(), 0);
            // Outputs
            assert_eq!(output_impl_node_ids.len(), 1);
            //value_spec_checker.check_node_tag(output_impl_node_ids[0], ir::Tag{quot: ir::Quotient::Node{node_id: spec_node_id}, flow: ir::Flow::None})?;
            assert!(value_spec_checker.can_drop_node(output_impl_node_ids[0]));
            value_spec_checker.update_scalar_node(
                output_impl_node_ids[0],
                ir::Quotient::Node {
                    node_id: spec_node_id,
                },
                ir::Flow::Have,
            );
        }
        ir::Node::Select {
            condition,
            true_case,
            false_case,
        } => {
            // Inputs
            assert_eq!(input_impl_node_ids.len(), 3);
            for (input_index, input_spec_node_id) in
                [*condition, *true_case, *false_case].iter().enumerate()
            {
                let scalar = &value_spec_checker.scalar_nodes[&input_impl_node_ids[input_index]];
                assert_eq!(
                    scalar.quot,
                    ir::Quotient::Node {
                        node_id: *input_spec_node_id
                    }
                );
                assert!(scalar.flow.is_readable());
            }
            // Outputs
            assert_eq!(output_impl_node_ids.len(), 1);
            //value_spec_checker.check_node_tag(output_impl_node_ids[0], ir::Tag{quot: ir::Quotient::Node{node_id: spec_node_id}, flow: ir::Flow::None})?;
            assert!(value_spec_checker.can_drop_node(output_impl_node_ids[0]));
            value_spec_checker.update_scalar_node(
                output_impl_node_ids[0],
                ir::Quotient::Node {
                    node_id: spec_node_id,
                },
                ir::Flow::Have,
            );
        }
        ir::Node::CallFunctionClass {
            function_id,
            arguments,
        } => {
            // Inputs
            assert_eq!(input_impl_node_ids.len(), arguments.len());
            for (input_index, input_spec_node_id) in arguments.iter().enumerate() {
                let scalar = &value_spec_checker.scalar_nodes[&input_impl_node_ids[input_index]];

                assert_eq!(
                    scalar.quot,
                    ir::Quotient::Node {
                        node_id: *input_spec_node_id
                    }
                );
                assert!(scalar.flow.is_readable());
            }
            // Outputs
            for (output_index, output_impl_node_id) in output_impl_node_ids.iter().enumerate() {
                // To do: Check that spec node is really an extractresult
                assert!(value_spec_checker.can_drop_node(*output_impl_node_id));
                //value_spec_checker.check_node_tag(*output_impl_node_id, ir::Tag{quot: ir::Quotient::Node{node_id: spec_node_id}, flow: ir::Flow::Need})?;
                value_spec_checker.update_scalar_node(
                    *output_impl_node_id,
                    ir::Quotient::Node {
                        node_id: spec_node_id + 1 + output_index,
                    },
                    ir::Flow::Have,
                );
            }
        }
        _ => panic!("Unsupported node: {:?}", encoded_node),
    }

    return Ok(());
}

fn advance_forward_timeline<'program>(
    timeline_spec_checker: &mut FuncletSpecChecker<'program>,
    error_context: &ErrorContext,
    spec_node_id: ir::NodeId,
    input_impl_node_ids: &[ir::NodeId],
    output_impl_node_ids: &[ir::NodeId],
) -> Result<(), Error> {
    let encoded_node = &timeline_spec_checker.spec_funclet.nodes[spec_node_id];
    match encoded_node {
        ir::Node::EncodingEvent {
            local_past,
            remote_local_pasts,
        } => {
            assert_eq!(output_impl_node_ids.len(), 1);
            /*match timeline_spec_checker.current_implicit_tag {
                ir::Tag::Node{node_id : local_past_node_id} => {
                    assert_eq!(local_past_node_id, *local_past);
                }
                _ => panic!("Tag must be Node")
            }*/
            let from_tag = ir::Tag {
                quot: ir::Quotient::Node {
                    node_id: *local_past,
                },
                flow: ir::Flow::Have,
            };
            timeline_spec_checker.check_implicit_tag(error_context, from_tag)?;
            let encoded_impl_node_ids = &input_impl_node_ids[remote_local_pasts.len()..];
            let fence_impl_node_ids = &input_impl_node_ids[0..remote_local_pasts.len()];
            let new_encoded_state_spec_node_id = spec_node_id + 2;
            timeline_spec_checker.transition_state_subset_forwards(
                encoded_impl_node_ids,
                *local_past,
                new_encoded_state_spec_node_id,
            )?;
            for remote_local_past_spec_node_id in remote_local_pasts.iter() {
                timeline_spec_checker.transition_state_subset_forwards(
                    encoded_impl_node_ids,
                    *remote_local_past_spec_node_id,
                    new_encoded_state_spec_node_id,
                )?;
            }
            timeline_spec_checker.transition_state_forwards(*local_past, spec_node_id + 1)?;
            timeline_spec_checker.update_scalar_node(
                output_impl_node_ids[0],
                ir::Quotient::Node {
                    node_id: new_encoded_state_spec_node_id,
                },
                ir::Flow::Have,
            );
        }
        ir::Node::SubmissionEvent { local_past } => {
            assert_eq!(input_impl_node_ids.len(), 1);
            assert_eq!(output_impl_node_ids.len(), 1);

            let from_tag = ir::Tag {
                quot: ir::Quotient::Node {
                    node_id: *local_past,
                },
                flow: ir::Flow::Have,
            };
            // Use encoder
            timeline_spec_checker.check_node_tag(
                error_context,
                input_impl_node_ids[0],
                from_tag,
            )?;
            timeline_spec_checker.update_scalar_node(
                input_impl_node_ids[0],
                ir::Quotient::Node {
                    node_id: *local_past,
                },
                ir::Flow::None,
            );

            //timeline_spec_checker.check_implicit_tag(from_tag)?;
            timeline_spec_checker.transition_state_forwards(*local_past, spec_node_id)?;
            timeline_spec_checker.update_scalar_node(
                output_impl_node_ids[0],
                ir::Quotient::Node {
                    node_id: spec_node_id,
                },
                ir::Flow::Have,
            );
        }
        ir::Node::SynchronizationEvent {
            local_past,
            remote_local_past,
        } => {
            assert_eq!(input_impl_node_ids.len(), 1);
            assert_eq!(output_impl_node_ids.len(), 0);

            let from_tag = ir::Tag {
                quot: ir::Quotient::Node {
                    node_id: *local_past,
                },
                flow: ir::Flow::Have,
            };
            let remote_from_tag = ir::Tag {
                quot: ir::Quotient::Node {
                    node_id: *remote_local_past,
                },
                flow: ir::Flow::Have,
            };
            timeline_spec_checker.check_implicit_tag(error_context, from_tag)?;

            // Use fence
            timeline_spec_checker.check_node_tag(
                error_context,
                input_impl_node_ids[0],
                remote_from_tag,
            )?;
            timeline_spec_checker.update_scalar_node(
                input_impl_node_ids[0],
                ir::Quotient::Node {
                    node_id: *remote_local_past,
                },
                ir::Flow::None,
            );

            timeline_spec_checker.transition_state_forwards(*local_past, spec_node_id)?;
            timeline_spec_checker.transition_state_forwards(*remote_local_past, spec_node_id);
        }
        _ => panic!("Unsupported node: {:?}", encoded_node),
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
    value_spec_checker_opt: Option<FuncletSpecChecker<'program>>,
    timeline_spec_checker_opt: Option<FuncletSpecChecker<'program>>,
    spatial_spec_checker_opt: Option<FuncletSpecChecker<'program>>,
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
            value_spec_checker_opt: Some(FuncletSpecChecker::new(
                program,
                value_funclet,
                value_spec,
            )),
            timeline_spec_checker_opt: Some(FuncletSpecChecker::new(
                program,
                &program.funclets[timeline_spec.funclet_id_opt.unwrap()],
                timeline_spec,
            )),
            spatial_spec_checker_opt: Some(FuncletSpecChecker::new(
                program,
                &program.funclets[spatial_spec.funclet_id_opt.unwrap()],
                spatial_spec,
            )),
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
                ir::Type::NativeValue { storage_type } => NodeType::LocalVar(LocalVar {
                    storage_type: *storage_type,
                }),
                ir::Type::Ref {
                    storage_type,
                    //queue_stage,
                    storage_place,
                    buffer_flags
                } => NodeType::Slot(Slot {
                    storage_type: *storage_type,
                    //queue_stage: *queue_stage,
                    queue_place: *storage_place,
                    buffer_flags: *buffer_flags
                }),
                ir::Type::Fence { queue_place } => NodeType::Fence(Fence {
                    queue_place: *queue_place,
                }),
                ir::Type::Buffer {
                    storage_place,
                    static_layout_opt,
                    flags
                } => NodeType::Buffer(Buffer {
                    storage_place: *storage_place,
                    static_layout_opt: *static_layout_opt,
                    buffer_flags: *flags
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

    fn can_drop_node(&self, node_id: ir::NodeId) -> bool {
        self.value_spec_checker_opt
            .as_ref()
            .unwrap()
            .can_drop_node(node_id)
            || self
                .timeline_spec_checker_opt
                .as_ref()
                .unwrap()
                .can_drop_node(node_id)
            || self
                .spatial_spec_checker_opt
                .as_ref()
                .unwrap()
                .can_drop_node(node_id)
    }

    fn is_neutral_node(&self, node_id: ir::NodeId) -> bool {
        self.value_spec_checker_opt
            .as_ref()
            .unwrap()
            .is_neutral_node(node_id)
            || self
                .timeline_spec_checker_opt
                .as_ref()
                .unwrap()
                .is_neutral_node(node_id)
            || self
                .spatial_spec_checker_opt
                .as_ref()
                .unwrap()
                .is_neutral_node(node_id)
    }

    fn drop_node(&mut self, node_id: ir::NodeId) {
        self.value_spec_checker_opt
            .as_mut()
            .unwrap()
            .drop_node(node_id);
        self.timeline_spec_checker_opt
            .as_mut()
            .unwrap()
            .drop_node(node_id);
        self.spatial_spec_checker_opt
            .as_mut()
            .unwrap()
            .drop_node(node_id);
    }

    /*fn contextualize_error(&self, error : Error) -> Error {
        match error {
            Error::Unknown{message} => Error::Unknown{message},
            Error::Generic{message} => Error::Generic{message: format!("Node #{} ({:?}):\n\t{}\n--- Funclet ---\n{:?}\n", self.current_node_id, & self.scheduling_funclet.nodes[self.current_node_id], message, self.scheduling_funclet)}
        }
    }*/

    /*fn report_if_error<T>(&self, result : Result<T, Error>) -> Result<T, Error> {
        Ok(result?)
    }*/

    fn check_yield_for_spec(
        &self,
        error_context: &ErrorContext,
        external_function_id: ir::ffi::ExternalFunctionId,
        operation: ir::Quotient,
        funclet_spec: &ir::FuncletSpec,
        spec_checker_opt: Option<&FuncletSpecChecker>,
        yielded_impl_node_ids: &[ir::NodeId],
        continuation_impl_node_id: ir::NodeId,
    ) -> Result<(), Error> {
        if let ir::Quotient::None = operation {
            return Ok(());
        }

        let Some(spec_funclet_id) = funclet_spec.funclet_id_opt else { return Err(Error::Generic{message: String::from("Expected spec funclet id")}); };
        let spec_funclet = &self.program.funclets[spec_funclet_id];
        let ir::Quotient::Node{node_id: call_node_id} = operation else { return Err(Error::Generic{message: String::from("Expected operation to be a Node")}); };
        let ir::Node::CallFunctionClass{function_id, arguments} = & spec_funclet.nodes[call_node_id] else { return Err(Error::Generic{message: String::from("Expected call node")}); };
        if !self.program.function_classes[*function_id]
            .external_function_ids
            .contains(&external_function_id)
        {
            return Err(Error::Generic {
                message: format!(
                    "External function #{} does not implement function class #{}",
                    external_function_id, function_id
                ),
            });
        }

        let spec_checker = spec_checker_opt.unwrap();

        let ir::ffi::ExternalFunction::CpuEffectfulOperation(effectful_operation) = & self.program.native_interface.external_functions[external_function_id.0] else { panic!("") };
        assert_eq!(effectful_operation.input_types.len() + 1, arguments.len());
        assert_eq!(yielded_impl_node_ids.len() + 1, arguments.len());

        assert!(arguments.len() > 0);

        for index in 0..yielded_impl_node_ids.len() {
            // 0th argument is the implicit
            let argument_spec_node_id = arguments[index + 1];
            spec_checker.check_node_tag(
                error_context,
                yielded_impl_node_ids[index],
                ir::Tag {
                    quot: ir::Quotient::Node {
                        node_id: argument_spec_node_id,
                    },
                    flow: ir::Flow::Have,
                },
            )?;
        }
        spec_checker.check_implicit_tag(
            error_context,
            ir::Tag {
                quot: ir::Quotient::Node {
                    node_id: arguments[0],
                },
                flow: ir::Flow::Have,
            },
        )?;

        // Check continuation against outputs
        let continuation_join_point = &self.node_join_points[&continuation_impl_node_id];
        assert_eq!(
            continuation_join_point.input_types.len(),
            effectful_operation.output_types.len()
        );
        let mut continuation_input_tags = Vec::<ir::Tag>::new();
        for index in 0..continuation_join_point.input_types.len() {
            // 0th output is the implicit
            let extract_node_id = call_node_id + 2 + index;
            let ir::Node::ExtractResult{node_id, index: argument_index} = & spec_funclet.nodes[extract_node_id] else { return Err(Error::Generic{message: String::from("Expected extract result")}); };
            assert_eq!(call_node_id, *node_id);
            assert_eq!(*argument_index, index);
            continuation_input_tags.push(ir::Tag {
                quot: ir::Quotient::Node {
                    node_id: extract_node_id,
                },
                flow: ir::Flow::Have,
            });
        }

        // Need to figure out what the implicit tag should be
        spec_checker_opt.unwrap().check_join_tags(
            error_context,
            continuation_impl_node_id,
            continuation_input_tags.as_slice(),
            ir::Tag {
                quot: ir::Quotient::Node {
                    node_id: call_node_id + 1,
                },
                flow: ir::Flow::Have,
            },
        );

        return Ok(());
    }

    pub fn check_next_node(
        &mut self,
        error_context: &ErrorContext,
        current_node_id: ir::NodeId,
    ) -> Result<(), Error> {
        assert_eq!(self.current_node_id, current_node_id);
        let current_node = &self.scheduling_funclet.nodes[current_node_id];
        match current_node {
            ir::Node::None => (),
            ir::Node::Phi { .. } => (),
            ir::Node::ExtractResult { node_id, index } => (),
            ir::Node::AllocTemporary {
                place,
                storage_type,
                buffer_flags
            } => {
                // Has no value and can only be overwritten
                self.value_spec_checker_opt
                    .as_mut()
                    .unwrap()
                    .update_scalar_node(current_node_id, ir::Quotient::None, ir::Flow::None);
                // Can be used in encoding
                self.timeline_spec_checker_opt
                    .as_mut()
                    .unwrap()
                    .update_node_current_with_implicit(current_node_id);
                // Borrowed against the current space and needs to be released before returning
                let spatial_spec_checker = self.spatial_spec_checker_opt.as_mut().unwrap();
                assert_eq!(
                    spatial_spec_checker.current_implicit_tag.flow,
                    ir::Flow::Have
                );
                spatial_spec_checker.update_scalar_node(
                    current_node_id,
                    spatial_spec_checker.current_implicit_tag.quot,
                    ir::Flow::Met,
                );
                self.node_types.insert(
                    current_node_id,
                    NodeType::Slot(Slot {
                        storage_type: *storage_type,
                        queue_place: *place,
                        buffer_flags : *buffer_flags
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
                operation:
                    ir::Quotient::Node {
                        node_id: operation_node_id,
                    },
                inputs,
                outputs,
            } => {
                let encoded_node = &self
                    .value_spec_checker_opt
                    .as_ref()
                    .unwrap()
                    .spec_funclet
                    .nodes[*operation_node_id];
                let (output_count, storage_type) = match encoded_node {
                    ir::Node::Constant { type_id, .. } => {
                        let ir::Type::NativeValue{storage_type} = &self.program.types[*type_id] else { panic!("Must be native value") };
                        (1, *storage_type)
                    }
                    ir::Node::Select { true_case, .. } => {
                        (1, self.node_types[&inputs[1]].storage_type().unwrap())
                    }
                    _ => panic!("Unsupported with LocalDoBuiltin: {:?}", encoded_node),
                };

                advance_forward_value_do(
                    self.value_spec_checker_opt.as_mut().unwrap(),
                    error_context,
                    *operation_node_id,
                    inputs,
                    outputs,
                )?;

                for (input_index, input_impl_node_id) in inputs.iter().enumerate() {
                    self.timeline_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .check_node_is_readable_at_implicit(error_context, *input_impl_node_id)?;
                    self.spatial_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .check_node_is_readable_at_implicit(error_context, *input_impl_node_id)?;
                }

                for (output_index, output_impl_node_id) in outputs.iter().enumerate() {
                    self.timeline_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .check_node_is_readable_at_implicit(error_context, *output_impl_node_id)?;
                    self.spatial_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .check_node_is_readable_at_implicit(error_context, *output_impl_node_id)?;
                }
            }
            ir::Node::LocalDoExternal {
                operation:
                    ir::Quotient::Node {
                        node_id: operation_node_id,
                    },
                external_function_id,
                inputs,
                outputs,
            } => {
                let external_function =
                    &self.program.native_interface.external_functions[external_function_id.0];
                assert_eq!(
                    external_function.get_input_types().map(|x| x.len()),
                    Some(inputs.len())
                );

                advance_forward_value_do(
                    self.value_spec_checker_opt.as_mut().unwrap(),
                    error_context,
                    *operation_node_id,
                    inputs,
                    outputs,
                )?;

                for (input_index, input_impl_node_id) in inputs.iter().enumerate() {
                    /*let node_type : &NodeType = self.node_types.get(input_impl_node_id).unwrap();
                    match external_function {
                        ExternalFunction::CpuPureOperation{ .. } => {
                            match node_type {
                                NodeType::LocalVar{ .. } => (),
                                _ => panic!("Inputs to a CPU pure operation must be ", )
                            }
                        }
                    }*/
                    
                    self.timeline_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .check_node_is_readable_at_implicit(error_context, *input_impl_node_id)?;
                    self.spatial_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .check_node_is_readable_at_implicit(error_context, *input_impl_node_id)?;
                }

                //let output_types external_function.get_output_types().unwrap();
                for (output_index, output_impl_node_id) in outputs.iter().enumerate() {
                    self.timeline_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .check_node_is_readable_at_implicit(error_context, *output_impl_node_id)?;
                    self.spatial_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .check_node_is_readable_at_implicit(error_context, *output_impl_node_id)?;
                }

                let encoded_node = &self.value_funclet.nodes[*operation_node_id];

                match encoded_node {
                    ir::Node::CallFunctionClass {
                        function_id,
                        arguments,
                    } => {
                        assert!(self.program.function_classes[*function_id]
                            .external_function_ids
                            .contains(external_function_id));
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
                        }
                    }
                    _ => panic!(
                        "Node is not supported with LocalDoExternal: {:?}",
                        encoded_node
                    ),
                }
            }
            ir::Node::EncodeDoExternal {
                operation:
                    ir::Quotient::Node {
                        node_id: operation_node_id,
                    },
                external_function_id,
                inputs,
                outputs,
                encoder,
            } => {
                //assert_eq!(*place, ir::Place::Gpu);

                /*for (input_index, input_impl_node_id) in inputs.iter().enumerate() {
                    self.timeline_spec_checker_opt.as_mut().unwrap().check_node_is_readable_at_implicit(*input_impl_node_id)?;
                    self.spatial_spec_checker_opt.as_mut().unwrap().check_node_is_readable_at_implicit(*input_impl_node_id)?;
                }

                for (output_index, output_impl_node_id) in outputs.iter().enumerate() {
                    self.timeline_spec_checker_opt.as_mut().unwrap().check_node_is_current_with_implicit(*output_impl_node_id)?;
                    self.spatial_spec_checker_opt.as_mut().unwrap().check_node_is_current_with_implicit(*output_impl_node_id)?;
                    self.timeline_spec_checker_opt.as_mut().unwrap().update_node_current_with_implicit(*output_impl_node_id);
                    self.spatial_spec_checker_opt.as_mut().unwrap().update_node_current_with_implicit(*output_impl_node_id);
                }*/

                let encoded_node = &self.value_funclet.nodes[*operation_node_id];

                let value_spec_checker = self.value_spec_checker_opt.as_mut().unwrap();
                let timeline_spec_checker = self.timeline_spec_checker_opt.as_mut().unwrap();
                let spatial_spec_checker = self.spatial_spec_checker_opt.as_mut().unwrap();
                //let encoder_value_tag = value_spec_checker.scalar_nodes[encoder];
                let encoder_timeline_tag = timeline_spec_checker.scalar_nodes[encoder];
                let encoder_spatial_tag = spatial_spec_checker.scalar_nodes[encoder];

                let Some(NodeType::Encoder(Encoder { queue_place })) = self.node_types.get(encoder) else {
                    panic!("Node #{} is not an encoder\n{}", encoder, error_context);
                };

                match encoded_node {
                    ir::Node::CallFunctionClass {
                        function_id,
                        arguments,
                    } => {
                        assert!(self.program.function_classes[*function_id]
                            .external_function_ids
                            .contains(external_function_id));

                        let function = &self.program.native_interface.external_functions
                            [external_function_id.0];
                        let kernel = function.get_gpu_kernel().unwrap();

                        assert_eq!(inputs.len(), arguments.len());
                        assert_eq!(outputs.len(), kernel.output_types.len());

                        /*ir::validation::validate_gpu_kernel_bindings(
                            kernel,
                            &inputs[kernel.dimensionality..],
                            outputs,
                        );*/

                        for (input_index, input_impl_node_id) in
                            inputs[0..kernel.dimensionality].iter().enumerate()
                        {
                            let Some(NodeType::LocalVar{ .. }) = self.node_types.get(input_impl_node_id)
                            else {
                                panic!("Dimension arguments to a GPU compute dispatch must be NativeValue and not Refs\n{}", error_context);
                            };
                            value_spec_checker.check_node_is_readable_at_implicit(
                                error_context,
                                *input_impl_node_id,
                            )?;
                            timeline_spec_checker.check_node_is_readable_at_implicit(
                                error_context,
                                *input_impl_node_id,
                            )?;
                            spatial_spec_checker.check_node_is_readable_at_implicit(
                                error_context,
                                *input_impl_node_id,
                            )?;
                        }

                        let mut forwarding_input_scheduling_node_ids = HashSet::<ir::NodeId>::new();
                        let mut forwarded_output_scheduling_node_ids = HashSet::<ir::NodeId>::new();
                        for (input_index, input_impl_node_id) in
                            inputs[kernel.dimensionality..].iter().enumerate()
                        {
                            assert_eq!(
                                self.node_types[input_impl_node_id]
                                    .storage_type()
                                    .unwrap(),
                                kernel.input_types[input_index]
                            );

                            let NodeType::Slot(Slot{queue_place: ir::Place::Gpu, buffer_flags, ..}) = self.node_types.get(input_impl_node_id).as_ref().unwrap() else {
                                panic!("Non-dimensionality arguments to encode_do of a GPU kernel must be GPU refs (offending argument to encode_do: #{} which is node #{})\n{}", input_index, input_impl_node_id, error_context)
                            };

                            assert!(buffer_flags.storage || buffer_flags.uniform, "Argument #{} to encode_do is marked as neither a storage nor uniform buffer (maps to node #{})\n{}", input_index, input_impl_node_id, error_context);

                            //value_spec_checker.check_node_is_readable_at(*input_impl_node_id, encoder_value_tag)?;
                            timeline_spec_checker.check_node_is_readable_at(
                                error_context,
                                *input_impl_node_id,
                                encoder_timeline_tag,
                            )?;
                            spatial_spec_checker.check_node_is_readable_at(
                                error_context,
                                *input_impl_node_id,
                                encoder_spatial_tag,
                            )?;

                            if let Some(forwarded_output_index) =
                                kernel.output_of_forwarding_input(input_index)
                            {
                                // Must be the same location
                                assert_eq!(outputs[forwarded_output_index], *input_impl_node_id);
                                forwarding_input_scheduling_node_ids.insert(*input_impl_node_id);
                                forwarded_output_scheduling_node_ids
                                    .insert(outputs[forwarded_output_index]);
                            }
                        }

                        //output_of_forwarding_input

                        for (index, output_type_id) in kernel.output_types.iter().enumerate() {
                            assert_eq!(
                                self.node_types[&outputs[index]].storage_type().unwrap(),
                                kernel.output_types[index]
                            );

                            let output_impl_node_id = outputs[index];
                            timeline_spec_checker.check_node_is_readable_at(
                                error_context,
                                output_impl_node_id,
                                encoder_timeline_tag,
                            )?;
                            spatial_spec_checker.check_node_is_readable_at(
                                error_context,
                                output_impl_node_id,
                                encoder_spatial_tag,
                            )?;

                            let is_forwarded =
                                forwarded_output_scheduling_node_ids.contains(&outputs[index]);
                            if !is_forwarded {}
                        }
                    }
                    _ => panic!("Unsupported with EncodeDoExternal: {:?}", encoded_node),
                }

                advance_forward_value_do(
                    self.value_spec_checker_opt.as_mut().unwrap(),
                    error_context,
                    *operation_node_id,
                    inputs,
                    outputs,
                )?;
            }
            ir::Node::ReadRef {
                source,
                storage_type,
            } => {
                let NodeType::Slot(Slot{queue_place, buffer_flags, ..}) = &self.node_types[source] else { panic!("Must be a slot") };
                assert_eq!(*queue_place, ir::Place::Local);

                // Input
                self.timeline_spec_checker_opt
                    .as_mut()
                    .unwrap()
                    .check_node_is_readable_at_implicit(error_context, *source)?;
                self.spatial_spec_checker_opt
                    .as_mut()
                    .unwrap()
                    .check_node_is_readable_at_implicit(error_context, *source)?;

                // Output
                advance_forward_value_copy(
                    self.value_spec_checker_opt.as_mut().unwrap(),
                    error_context,
                    *source,
                    current_node_id,
                )?;
                self.timeline_spec_checker_opt
                    .as_mut()
                    .unwrap()
                    .update_node_current_with_implicit(current_node_id);
                self.spatial_spec_checker_opt
                    .as_mut()
                    .unwrap()
                    .update_node_current_with_implicit(current_node_id);

                self.node_types.insert(
                    current_node_id,
                    NodeType::LocalVar(LocalVar {
                        storage_type: *storage_type,
                    }),
                );
            }
            ir::Node::WriteRef {
                destination,
                storage_type,
                source,
            } => {
                let NodeType::LocalVar(LocalVar{..}) = &self.node_types[source] else { panic!("Must be a local var") };
                let NodeType::Slot(Slot{queue_place, buffer_flags, ..}) = &self.node_types[destination] else { panic!("Must be a slot") };
                assert_eq!(*queue_place, ir::Place::Local);

                // Input
                self.timeline_spec_checker_opt
                    .as_mut()
                    .unwrap()
                    .check_node_is_current_with_implicit(error_context, *source)?;
                self.spatial_spec_checker_opt
                    .as_mut()
                    .unwrap()
                    .check_node_is_current_with_implicit(error_context, *source)?;

                // Output
                advance_forward_value_copy(
                    self.value_spec_checker_opt.as_mut().unwrap(),
                    error_context,
                    *source,
                    *destination,
                )?;
                self.timeline_spec_checker_opt
                    .as_mut()
                    .unwrap()
                    .check_node_is_readable_at_implicit(error_context, *destination)?;
                self.spatial_spec_checker_opt
                    .as_mut()
                    .unwrap()
                    .check_node_is_readable_at_implicit(error_context, *destination)?;
            }
            ir::Node::LocalCopy { input, output } => {

                let Some(NodeType::Slot(Slot { buffer_flags: input_buffer_flags, .. })) = self.node_types.get(input) else {
                    panic!("Source of local_copy (node #{}) is not a ref \n{}", input, error_context);
                };
                let Some(NodeType::Slot(Slot { buffer_flags: output_buffer_flags, .. })) = self.node_types.get(output) else {
                    panic!("Destination of local_copy (node #{}) is not a ref \n{}", output, error_context);
                };
                assert!(input_buffer_flags.map_read, "Source of local_copy (node #{}) must be marked with map_read\n{}", input, error_context);
                assert!(output_buffer_flags.map_write, "Destination of local_copy (node #{}) must be marked with map_write\n{}", output, error_context);

                advance_forward_value_copy(
                    self.value_spec_checker_opt.as_mut().unwrap(),
                    error_context,
                    *input,
                    *output,
                )?;
                self.timeline_spec_checker_opt
                    .as_mut()
                    .unwrap()
                    .check_node_is_readable_at_implicit(error_context, *input)?;
                self.spatial_spec_checker_opt
                    .as_mut()
                    .unwrap()
                    .check_node_is_readable_at_implicit(error_context, *input)?;
                self.timeline_spec_checker_opt
                    .as_mut()
                    .unwrap()
                    .check_node_is_readable_at_implicit(error_context, *output)?;
                self.spatial_spec_checker_opt
                    .as_mut()
                    .unwrap()
                    .check_node_is_readable_at_implicit(error_context, *output)?;
            }
            ir::Node::EncodeCopy {
                input,
                output,
                encoder,
            } => {
                let Some(NodeType::Encoder(Encoder { queue_place })) = self.node_types.get(encoder) else {
                    panic!("Not an encoder\n{}", error_context);
                };
                let Some(NodeType::Slot(Slot { buffer_flags: input_buffer_flags, .. })) = self.node_types.get(input) else {
                    panic!("Source of encode_copy (node #{}) is not a ref \n{}", input, error_context);
                };
                let Some(NodeType::Slot(Slot { buffer_flags: output_buffer_flags, .. })) = self.node_types.get(output) else {
                    panic!("Destination of encode_copy (node #{}) is not a ref \n{}", output, error_context);
                };
                assert!(input_buffer_flags.copy_src, "Source of encode_copy (node #{}) must be marked with copy_src\n{}", input, error_context);
                assert!(output_buffer_flags.copy_dst, "Destination of encode_copy (node #{}) must be marked with copy_dst\n{}", output, error_context);
                
                advance_forward_value_copy(
                    self.value_spec_checker_opt.as_mut().unwrap(),
                    error_context,
                    *input,
                    *output,
                )?;
                //??
            }
            ir::Node::BeginEncoding {
                place,
                event:
                    ir::Quotient::Node {
                        node_id: event_node_id,
                    },
                encoded,
                fences,
            } => {
                let timeline_spec_checker = self.timeline_spec_checker_opt.as_mut().unwrap();
                let ir::Node::EncodingEvent{remote_local_pasts, ..} = & timeline_spec_checker.spec_funclet.nodes[*event_node_id] else { panic!("Must be an encoding event\n{}", error_context) };
                assert_eq!(remote_local_pasts.len(), fences.len());
                let mut input_impl_node_ids = Vec::<ir::NodeId>::new();
                input_impl_node_ids.extend_from_slice(fences);
                input_impl_node_ids.extend_from_slice(encoded);
                advance_forward_timeline(
                    timeline_spec_checker,
                    error_context,
                    *event_node_id,
                    input_impl_node_ids.as_slice(),
                    &[current_node_id],
                )?;
                //self.timeline_spec_checker_opt.as_mut().unwrap().update_node_current_with_implicit(current_node_id);
                self.value_spec_checker_opt
                    .as_mut()
                    .unwrap()
                    .update_scalar_node(current_node_id, ir::Quotient::None, ir::Flow::Have);
                self.spatial_spec_checker_opt
                    .as_mut()
                    .unwrap()
                    .update_node_current_with_implicit(current_node_id);

                self.node_types.insert(
                    current_node_id,
                    NodeType::Encoder(Encoder {
                        queue_place: *place,
                    }),
                );
            }
            ir::Node::Submit {
                event:
                    ir::Quotient::Node {
                        node_id: event_node_id,
                    },
                encoder,
            } => {
                let Some(NodeType::Encoder(Encoder { queue_place })) = self.node_types.remove(encoder) else {
                    panic!("Not an encoder\n{}", error_context);
                };

                let timeline_spec_checker = self.timeline_spec_checker_opt.as_mut().unwrap();
                let ir::Node::SubmissionEvent{..} = & timeline_spec_checker.spec_funclet.nodes[*event_node_id] else { panic!("Must be a submission event\n{}", error_context) };
                advance_forward_timeline(
                    timeline_spec_checker,
                    error_context,
                    *event_node_id,
                    &[*encoder],
                    &[current_node_id],
                )?;
                self.value_spec_checker_opt
                    .as_mut()
                    .unwrap()
                    .update_scalar_node(current_node_id, ir::Quotient::None, ir::Flow::Have);
                //let implicit_tag = timeline_spec_checker.current_implicit_tag;
                //self.timeline_spec_checker_opt.as_mut().unwrap().update_node_current_with_implicit(current_node_id);
                //self.spatial_spec_checker_opt.as_mut().unwrap().update_scalar_node(current_node_id, ir::Tag::None, ir::Flow::Have);
                self.spatial_spec_checker_opt
                    .as_mut()
                    .unwrap()
                    .update_node_current_with_implicit(current_node_id);

                //panic!("Node #{} After submit: {:?}\nSpec funclet: {:?}", current_node_id, self.timeline_spec_checker_opt.as_ref().unwrap().scalar_nodes, self.timeline_spec_checker_opt.as_ref().unwrap().spec_funclet);

                self.node_types
                    .insert(current_node_id, NodeType::Fence(Fence { queue_place }));
            }
            ir::Node::SyncFence {
                fence,
                event:
                    ir::Quotient::Node {
                        node_id: event_node_id,
                    },
            } => {
                let timeline_spec_checker = self.timeline_spec_checker_opt.as_mut().unwrap();
                let ir::Node::SynchronizationEvent{..} = & timeline_spec_checker.spec_funclet.nodes[*event_node_id] else {
                    panic!("Must be an synchronization event\n{}", error_context)
                };
                advance_forward_timeline(
                    timeline_spec_checker,
                    error_context,
                    *event_node_id,
                    &[*fence],
                    &[],
                )?;

                let fenced_place = if let Some(NodeType::Fence(Fence { queue_place })) =
                    &self.node_types.remove(fence)
                {
                    *queue_place
                } else {
                    panic!("Not a fence");
                };

                assert_eq!(fenced_place, ir::Place::Gpu);
            }
            ir::Node::StaticSplit {
                spatial_operation: ir::Quotient::Node{node_id: spatial_spec_node_id},
                node: buffer_impl_node_id,
                sizes,
                place,
            } => {
                // Temporary restriction
                match *place {
                    ir::Place::Local => panic!("Unimplemented"),
                    _ => {}
                }

                let spatial_spec_checker = self.spatial_spec_checker_opt.as_mut().unwrap();
                let ir::Node::SeparatedBufferSpaces{count: space_count, space: space_spec_node_id} = & spatial_spec_checker.spec_funclet.nodes[*spatial_spec_node_id] else {
                    panic!("Must be a separated space\n{}", error_context)
                };

                self.spatial_spec_checker_opt
                .as_mut()
                .unwrap()
                .check_node_tag(error_context, *buffer_impl_node_id, ir::Tag{quot: ir::Quotient::Node{node_id: *space_spec_node_id}, flow: ir::Flow::Have});

                let Some(NodeType::Buffer(buffer)) = self.node_types.get_mut(buffer_impl_node_id) else { panic!("{}", error_context); };
                assert_eq!(buffer.storage_place, *place);

                assert_eq!(sizes.len(), *space_count);

                let output_count = sizes.len() + 1;

                for (i, size) in sizes.iter().enumerate() {
                    let offset = 1 + i;
                    let output_impl_node_id = current_node_id + offset;

                    let Some(NodeType::Buffer(buffer)) = self.node_types.get_mut(buffer_impl_node_id) else { panic!("{}", error_context); };
                    let buffer_flags = buffer.buffer_flags;
                    let new_static_layout = buffer.split_static(&self.program.native_interface, error_context, *size).unwrap();

                    self.value_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .update_scalar_node(output_impl_node_id, ir::Quotient::None, ir::Flow::Have);
                    self.timeline_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .update_scalar_node(output_impl_node_id, ir::Quotient::None, ir::Flow::Have);
                    self.spatial_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .update_scalar_node(
                            output_impl_node_id,
                            ir::Quotient::Node{node_id: *spatial_spec_node_id + offset},
                            ir::Flow::Have,
                        );
                    self.node_types.insert(
                        output_impl_node_id,
                        NodeType::Buffer(Buffer {
                            storage_place : * place,
                            static_layout_opt : Some(new_static_layout),
                            buffer_flags
                        }),
                    );
                }

                {
                    let offset = 1 + sizes.len();

                    self.value_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .update_scalar_node(*buffer_impl_node_id, ir::Quotient::None, ir::Flow::Have);
                    self.timeline_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .update_scalar_node(*buffer_impl_node_id, ir::Quotient::None, ir::Flow::Have);
                    self.spatial_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .update_scalar_node(
                            *buffer_impl_node_id,
                            ir::Quotient::Node{node_id: *spatial_spec_node_id + offset},
                            ir::Flow::Have,
                        );
                }
            }
            ir::Node::StaticMerge {
                spatial_operation: ir::Quotient::Node{node_id: spatial_spec_node_id},
                nodes: impl_node_ids,
                place,
            } => {
                // Temporary restriction
                match *place {
                    ir::Place::Local => panic!("Unimplemented"),
                    _ => {}
                }

                assert!(impl_node_ids.len() > 0);

                let spatial_spec_checker = self.spatial_spec_checker_opt.as_mut().unwrap();
                let ir::Node::SeparatedBufferSpaces{count: space_count, space: space_spec_node_id} = & spatial_spec_checker.spec_funclet.nodes[*spatial_spec_node_id] else {
                    panic!("Must be a separated space\n{}", error_context)
                };

                let buffer_impl_node_id = impl_node_ids[impl_node_ids.len() - 1];
                self.spatial_spec_checker_opt
                .as_mut()
                .unwrap()
                .check_node_tag(error_context, buffer_impl_node_id, ir::Tag{quot: ir::Quotient::Node{node_id: *space_spec_node_id + impl_node_ids.len()}, flow: ir::Flow::Have});

                // Deallocate in reverse order
                for i in (0 .. (impl_node_ids.len() - 1)).rev() {
                    let offset = 1 + i;
                    let impl_node_id = impl_node_ids[i];

                    let Some(NodeType::Buffer(Buffer {
                        storage_place,
                        static_layout_opt : Some(predecessor_static_layout),
                        buffer_flags
                    })) = self.node_types.remove(& impl_node_id) else {
                        panic!("{}", error_context)
                    };

                    assert_eq!(storage_place, *place);

                    let Some(NodeType::Buffer(buffer)) = self.node_types.get_mut(& buffer_impl_node_id) else { panic!("{}", error_context); };
                    assert_eq!(buffer_flags, buffer.buffer_flags);
                    buffer.merge_static_left(&self.program.native_interface, error_context, predecessor_static_layout)?;

                    assert!(self.value_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .can_drop_node(impl_node_id));
                    self.timeline_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .check_node_tag(error_context, impl_node_id, ir::Tag{quot: ir::Quotient::None, flow: ir::Flow::Have});
                    self.spatial_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .check_node_tag(
                            error_context,
                            impl_node_id,
                            ir::Tag{quot: ir::Quotient::Node{node_id: *spatial_spec_node_id + offset}, flow: ir::Flow::Have}
                        );
                }

                {
                    self.value_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .update_scalar_node(buffer_impl_node_id, ir::Quotient::None, ir::Flow::Have);
                    self.timeline_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .update_scalar_node(buffer_impl_node_id, ir::Quotient::None, ir::Flow::Have);
                    self.spatial_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .update_scalar_node(
                            buffer_impl_node_id,
                            ir::Quotient::Node{node_id: *spatial_spec_node_id},
                            ir::Flow::Have,
                        );
                }
            }
            /*ir::Node::StaticAlloc {
                spatial_operation: ir::Quotient::Node{node_id: spatial_spec_node_id},
                node: buffer_impl_node_id,
                storage_types,
                place,
            } => {
                // Temporary restriction
                match *place {
                    ir::Place::Local => panic!("Unimplemented"),
                    _ => {}
                }

                let spatial_spec_checker = self.spatial_spec_checker_opt.as_mut().unwrap();
                let ir::Node::SeparatedBufferSpaces{count: space_count, space: space_spec_node_id} = & spatial_spec_checker.spec_funclet.nodes[*spatial_spec_node_id] else {
                    panic!("Must be a separated space\n{}", error_context)
                };

                self.spatial_spec_checker_opt
                .as_mut()
                .unwrap()
                .check_node_tag(error_context, *buffer_impl_node_id, ir::Tag{quot: ir::Quotient::Node{node_id: *space_spec_node_id}, flow: ir::Flow::Have});

                let Some(NodeType::Buffer(buffer)) = self.node_types.get_mut(buffer_impl_node_id) else { panic!("{}", error_context); };
                assert_eq!(buffer.storage_place, *place);

                assert_eq!(storage_types.len(), *space_count);

                let output_count = storage_types.len() + 1;

                for (i, storage_type) in storage_types.iter().enumerate() {
                    let offset = 1 + i;
                    let output_impl_node_id = current_node_id + offset;

                    let Some(NodeType::Buffer(buffer)) = self.node_types.get_mut(buffer_impl_node_id) else { panic!("{}", error_context); };
                    buffer.alloc_static(&self.program.native_interface, error_context, *storage_type)?;

                    self.value_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .update_scalar_node(output_impl_node_id, ir::Quotient::None, ir::Flow::Have);
                    self.timeline_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .update_scalar_node(output_impl_node_id, ir::Quotient::None, ir::Flow::Have);
                    self.spatial_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .update_scalar_node(
                            output_impl_node_id,
                            ir::Quotient::Node{node_id: *spatial_spec_node_id + offset},
                            ir::Flow::Have,
                        );
                    self.node_types.insert(
                        output_impl_node_id,
                        NodeType::Slot(Slot {
                            storage_type: *storage_type,
                            queue_place: *place,
                        }),
                    );
                }

                {
                    let offset = 1 + storage_types.len();

                    self.value_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .update_scalar_node(*buffer_impl_node_id, ir::Quotient::None, ir::Flow::Have);
                    self.timeline_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .update_scalar_node(*buffer_impl_node_id, ir::Quotient::None, ir::Flow::Have);
                    self.spatial_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .update_scalar_node(
                            *buffer_impl_node_id,
                            ir::Quotient::Node{node_id: *spatial_spec_node_id + offset},
                            ir::Flow::Have,
                        );
                }
            }
            ir::Node::StaticDealloc {
                spatial_operation,
                nodes: impl_node_ids,
                place,
            } => {
                // Temporary restriction
                match *place {
                    ir::Place::Local => panic!("Unimplemented"),
                    _ => {}
                }

                assert!(impl_node_ids.len() > 0);

                let spatial_spec_checker = self.spatial_spec_checker_opt.as_mut().unwrap();
                let ir::Node::SeparatedBufferSpaces{count: space_count, space: space_spec_node_id} = & spatial_spec_checker.spec_funclet.nodes[*spatial_spec_node_id] else {
                    panic!("Must be a separated space\n{}", error_context)
                };

                let buffer_impl_node_id = impl_node_ids[impl_node_ids.len() - 1];
                self.spatial_spec_checker_opt
                .as_mut()
                .unwrap()
                .check_node_tag(error_context, buffer_impl_node_id, ir::Tag{quot: ir::Quotient::Node{node_id: *space_spec_node_id + impl_node_ids.len()}, flow: ir::Flow::Have});

                // Deallocate in reverse order
                for i in (0 .. (impl_node_ids.len() - 1)).rev() {
                    let offset = 1 + i;
                    let impl_node_id = impl_node_ids[i];

                    let Some(NodeType::Slot(Slot {
                        storage_type,
                        queue_place,
                    })) = self.node_types.remove(impl_node_id);

                    assert_eq!(*queue_place, *place);

                    let Some(NodeType::Buffer(buffer)) = self.node_types.get_mut(buffer_impl_node_id) else { panic!("{}", error_context); };
                    buffer.dealloc_static(&self.program.native_interface, error_context, *storage_type)?;

                    assert!(self.value_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .can_drop_node(impl_node_id));
                    self.timeline_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .check_node_tag(impl_node_id, ir::Quotient::None, ir::Flow::Have);
                    self.spatial_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .check_node_tag(
                            impl_node_id,
                            ir::Quotient::Node{node_id: *spatial_spec_node_id + offset},
                            ir::Flow::Have,
                        );
                }

                {
                    self.value_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .update_scalar_node(buffer_impl_node_id, ir::Quotient::None, ir::Flow::Have);
                    self.timeline_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .update_scalar_node(buffer_impl_node_id, ir::Quotient::None, ir::Flow::Have);
                    self.spatial_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .update_scalar_node(
                            buffer_impl_node_id,
                            ir::Quotient::Node{node_id: *spatial_spec_node_id},
                            ir::Flow::Have,
                        );
                }

                panic!("Unimplemented")
            }*/
            ir::Node::StaticSubAlloc {
                node: buffer_node_id,
                place,
                storage_type,
            } => {
                // Temporary restriction
                match *place {
                    ir::Place::Local => panic!("Unimplemented"),
                    _ => {}
                }

                let buffer_spatial_tag =
                    self.spatial_spec_checker_opt.as_mut().unwrap().scalar_nodes[buffer_node_id];
                assert_ne!(buffer_spatial_tag.flow, ir::Flow::Met); // A continuation must own the space

                if let Some(NodeType::Buffer(buffer)) = self.node_types.get_mut(buffer_node_id)
                {
                    assert_eq!(buffer.storage_place, *place);
                    let buffer_flags = buffer.buffer_flags;
                    buffer.alloc_static(&self.program.native_interface, error_context, *storage_type)?;

                    self.value_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .update_scalar_node(current_node_id, ir::Quotient::None, ir::Flow::Have);
                    self.timeline_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .update_scalar_node(current_node_id, ir::Quotient::None, ir::Flow::Have);
                    self.spatial_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .update_scalar_node(
                            current_node_id,
                            buffer_spatial_tag.quot,
                            ir::Flow::Met,
                        );
                    self.node_types.insert(
                        current_node_id,
                        NodeType::Slot(Slot {
                            storage_type: *storage_type,
                            queue_place: *place,
                            buffer_flags
                        }),
                    );
                } else {
                    panic!("No static buffer at node #{}", buffer_node_id)
                }
            }
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
                self.value_spec_checker_opt
                    .as_mut()
                    .unwrap()
                    .initialize_default_join(current_node_id);
                self.timeline_spec_checker_opt
                    .as_mut()
                    .unwrap()
                    .initialize_default_join(current_node_id);
                self.spatial_spec_checker_opt
                    .as_mut()
                    .unwrap()
                    .initialize_default_join(current_node_id);
                self.node_join_points.insert(current_node_id, join_point);
                self.node_types.insert(current_node_id, NodeType::JoinPoint);
            }
            ir::Node::InlineJoin {
                funclet: join_funclet_id,
                captures,
                continuation: continuation_join_node_id,
            } => self.handle_join(
                error_context,
                *join_funclet_id,
                captures,
                *continuation_join_node_id,
                JoinKind::Inline,
            )?,
            ir::Node::SerializedJoin {
                funclet: join_funclet_id,
                captures,
                continuation: continuation_join_node_id,
            } => self.handle_join(
                error_context,
                *join_funclet_id,
                captures,
                *continuation_join_node_id,
                JoinKind::Serialized,
            )?,
            _ => panic!("Unimplemented"),
        }

        self.current_node_id += 1;
        return Ok(());
    }

    fn handle_join(
        &mut self,
        error_context: &ErrorContext,
        join_funclet_id: ir::FuncletId,
        captures: &[ir::NodeId],
        continuation_join_node_id: ir::NodeId,
        join_kind: JoinKind,
    ) -> Result<(), Error> {
        let Some(join_funclet) = self.program.funclets.get(join_funclet_id) else { return Err(Error::Generic{message: format!("No funclet {}\n{}", join_funclet_id, error_context)}) };
        //let join_funclet = &self.program.funclets[join_funclet_id];
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
                error_context
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
        self.value_spec_checker_opt.as_mut().unwrap().join(
            error_context,
            self.current_node_id,
            captures,
            join_funclet_value_spec,
            continuation_join_node_id,
        )?;
        self.timeline_spec_checker_opt.as_mut().unwrap().join(
            error_context,
            self.current_node_id,
            captures,
            join_funclet_timeline_spec,
            continuation_join_node_id,
        )?;
        self.spatial_spec_checker_opt.as_mut().unwrap().join(
            error_context,
            self.current_node_id,
            captures,
            join_funclet_spatial_spec,
            continuation_join_node_id,
        )?;
        self.node_join_points
            .insert(self.current_node_id, join_point);
        self.node_types
            .insert(self.current_node_id, NodeType::JoinPoint);

        return Ok(());
    }

    pub fn check_tail_edge(&mut self, error_context: &ErrorContext) -> Result<(), Error> {
        assert_eq!(self.current_node_id, self.scheduling_funclet.nodes.len());
        match &self.scheduling_funclet.tail_edge {
            ir::TailEdge::Return { return_values } => {
                for (return_index, return_node_id) in return_values.iter().enumerate() {
                    let Some(node_type) = self.node_types.remove(return_node_id) else {
                        return Err(error_context.generic_error(& format!("Returning nonexistent node #{}. Was it already used?", return_node_id)));
                        //return Err(Error::Generic{message: format!("Returning nonexistent node #{}. Was it already used?", return_node_id)});
                    };
                    check_slot_type(
                        &self.program,
                        self.scheduling_funclet.output_types[return_index],
                        &node_type,
                        error_context
                    );
                }

                self.value_spec_checker_opt
                    .as_mut()
                    .unwrap()
                    .check_return(error_context, return_values)?;
                self.timeline_spec_checker_opt
                    .as_mut()
                    .unwrap()
                    .check_return(error_context, return_values)?;
                self.spatial_spec_checker_opt
                    .as_mut()
                    .unwrap()
                    .check_return(error_context, return_values)?;
            }
            ir::TailEdge::Jump { join, arguments } => {

                if let Some(NodeType::JoinPoint) = self.node_types.remove(join) {
                    // Nothing, for now...
                } else {
                    panic!("Node at #{} is not a join point\n{}", join, error_context)
                }

                let join_point = &self.node_join_points[join];

                for (argument_index, argument_node_id) in arguments.iter().enumerate() {
                    let node_type = self.node_types.remove(argument_node_id).unwrap();
                    check_slot_type(
                        &self.program,
                        join_point.input_types[argument_index],
                        &node_type,
                        error_context
                    );
                }

                self.value_spec_checker_opt.as_mut().unwrap().check_jump(
                    error_context,
                    *join,
                    arguments,
                )?;
                self.timeline_spec_checker_opt
                    .as_mut()
                    .unwrap()
                    .check_jump(error_context, *join, arguments)?;
                self.spatial_spec_checker_opt.as_mut().unwrap().check_jump(
                    error_context,
                    *join,
                    arguments,
                )?;
            }
            ir::TailEdge::ScheduleCall {
                value_operation,
                timeline_operation,
                spatial_operation,
                callee_funclet_id: callee_scheduling_funclet_id_ref,
                callee_arguments,
                continuation_join: continuation_join_node_id,
            } => {
                let callee_scheduling_funclet_id = *callee_scheduling_funclet_id_ref;
                let continuation_join_point = &self.node_join_points[continuation_join_node_id];

                if let Some(NodeType::JoinPoint) = self.node_types.remove(continuation_join_node_id)
                {
                    // Nothing, for now...
                } else {
                    panic!("Node at #{} is not a join point", continuation_join_node_id)
                }

                let callee_funclet = &self.program.funclets[callee_scheduling_funclet_id];
                assert_eq!(callee_funclet.kind, ir::FuncletKind::ScheduleExplicit);

                /*{
                    let value_spec = self.get_funclet_value_spec(callee_funclet);
                    let callee_value_funclet_id = value_spec.funclet_id_opt.unwrap();
                    assert_eq!(self.value_funclet_id, callee_value_funclet_id);
                    let callee_value_funclet = & self.program.funclets[callee_value_funclet_id];
                    assert_eq!(callee_value_funclet.kind, ir::FuncletKind::Value);
                    let spec_checker = self.value_spec_checker_opt.as_mut().unwrap();
                    let e = match value_operation {
                        ir::Quotient::Node{node_id: value_operation_node_id} => {
                            if let ir::Node::CallFunctionClass{function_id, arguments} = &callee_value_funclet.nodes[*value_operation_node_id] {
                                spec_checker.check_vertical_call(*continuation_join_node_id, callee_arguments, value_spec, arguments, *value_operation_node_id)
                            }
                            else {
                                panic!("Not a call")
                            }
                        }
                        ir::Quotient::None => spec_checker.check_interior_call(*continuation_join_node_id, callee_arguments, value_spec),
                        _ => panic!(""),
                    };
                    e.map_err(|e| self.contextualize_error(e))?;
                }*/

                let callee_value_spec = self.get_funclet_value_spec(callee_funclet);
                self.value_spec_checker_opt.as_mut().unwrap().check_call(
                    error_context,
                    *value_operation,
                    *continuation_join_node_id,
                    callee_arguments,
                    callee_value_spec,
                )?;

                let callee_timeline_spec = self.get_funclet_timeline_spec(callee_funclet);
                self.timeline_spec_checker_opt
                    .as_mut()
                    .unwrap()
                    .check_call(
                        error_context,
                        *timeline_operation,
                        *continuation_join_node_id,
                        callee_arguments,
                        callee_timeline_spec,
                    )?;

                let callee_spatial_spec = self.get_funclet_spatial_spec(callee_funclet);
                self.spatial_spec_checker_opt.as_mut().unwrap().check_call(
                    error_context,
                    *spatial_operation,
                    *continuation_join_node_id,
                    callee_arguments,
                    callee_spatial_spec,
                )?;

                // Step 1: Check current -> callee edge
                for (argument_index, argument_node_id) in callee_arguments.iter().enumerate() {
                    let node_type = self.node_types.remove(argument_node_id).unwrap();
                    check_slot_type(
                        &self.program,
                        callee_funclet.input_types[argument_index],
                        &node_type,
                        error_context
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
                value_operation:
                    ir::Quotient::Node {
                        node_id: value_operation_node_id,
                    },
                timeline_operation: ir::Quotient::None,
                spatial_operation: ir::Quotient::None,
                condition: condition_slot_node_id,
                callee_funclet_ids,
                callee_arguments,
                continuation_join: continuation_join_node_id,
            } => {
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

                let current_value_funclet = &self.program.funclets[self.value_funclet_id];
                assert_eq!(current_value_funclet.kind, ir::FuncletKind::Value);

                //let condition_value_tag = self.value_spec_checker_opt.as_mut().unwrap().scalar_nodes[condition_slot_node_id];

                assert_eq!(
                    self.value_funclet_id,
                    true_funclet_value_spec.funclet_id_opt.unwrap()
                );
                assert_eq!(
                    self.value_funclet_id,
                    true_funclet_value_spec.funclet_id_opt.unwrap()
                );

                assert_eq!(callee_arguments.len(), true_funclet.input_types.len());
                assert_eq!(callee_arguments.len(), false_funclet.input_types.len());

                if let ir::Node::Select {
                    condition,
                    true_case,
                    false_case,
                } = &current_value_funclet.nodes[*value_operation_node_id]
                {
                    let cast_to_tag = ir::Quotient::Node {
                        node_id: *value_operation_node_id,
                    };
                    self.value_spec_checker_opt
                        .as_mut()
                        .unwrap()
                        .check_node_tag(
                            error_context,
                            *condition_slot_node_id,
                            ir::Tag {
                                quot: ir::Quotient::Node {
                                    node_id: *condition,
                                },
                                flow: ir::Flow::Have,
                            },
                        );
                    self.value_spec_checker_opt.as_mut().unwrap().check_choice(
                        error_context,
                        *continuation_join_node_id,
                        callee_arguments,
                        &[
                            &[(*true_case, *value_operation_node_id)],
                            &[(*false_case, *value_operation_node_id)],
                        ],
                        &[true_funclet_value_spec, false_funclet_value_spec],
                    )?;
                } else {
                    panic!("Not a select")
                };
                self.timeline_spec_checker_opt
                    .as_mut()
                    .unwrap()
                    .check_choice(
                        error_context,
                        *continuation_join_node_id,
                        callee_arguments,
                        &[&[], &[]],
                        &[true_funclet_value_spec, false_funclet_value_spec],
                    )?;
                self.spatial_spec_checker_opt
                    .as_mut()
                    .unwrap()
                    .check_choice(
                        error_context,
                        *continuation_join_node_id,
                        callee_arguments,
                        &[&[], &[]],
                        &[true_funclet_value_spec, false_funclet_value_spec],
                    )?;

                for (argument_index, argument_node_id) in callee_arguments.iter().enumerate() {
                    let node_type = self.node_types.remove(argument_node_id).unwrap();
                    check_slot_type(
                        &self.program,
                        true_funclet.input_types[argument_index],
                        &node_type,
                        error_context
                    );
                    check_slot_type(
                        &self.program,
                        false_funclet.input_types[argument_index],
                        &node_type,
                        error_context
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
            ir::TailEdge::ScheduleCallYield {
                value_operation,
                timeline_operation,
                spatial_operation,
                external_function_id,
                yielded_nodes: yielded_impl_node_ids,
                continuation_join: continuation_impl_node_id,
            } => {
                /*let callee_funclet = &self.program.funclets[callee_scheduling_funclet_id];
                assert_eq!(callee_funclet.kind, ir::FuncletKind::ScheduleExplicit);

                let callee_value_spec = self.get_funclet_value_spec(callee_funclet);
                self.value_spec_checker_opt.as_mut().unwrap().check_call(*value_operation, *continuation_join_node_id, yielded_node_ids, callee_value_spec).map_err(|e| self.contextualize_error(e))?;

                let callee_timeline_spec = self.get_funclet_timeline_spec(callee_funclet);
                self.timeline_spec_checker_opt.as_mut().unwrap().check_call(*timeline_operation, *continuation_join_node_id, yielded_node_ids, callee_timeline_spec).map_err(|e| self.contextualize_error(e))?;

                let callee_spatial_spec = self.get_funclet_spatial_spec(callee_funclet);
                self.spatial_spec_checker_opt.as_mut().unwrap().check_call(*spatial_operation, *continuation_join_node_id, yielded_node_ids, callee_spatial_spec).map_err(|e| self.contextualize_error(e))?;*/

                self.check_yield_for_spec(
                    error_context,
                    *external_function_id,
                    *value_operation,
                    self.value_spec,
                    self.value_spec_checker_opt.as_ref(),
                    yielded_impl_node_ids,
                    *continuation_impl_node_id,
                )?;
                self.check_yield_for_spec(
                    error_context,
                    *external_function_id,
                    *timeline_operation,
                    self.timeline_spec,
                    self.timeline_spec_checker_opt.as_ref(),
                    yielded_impl_node_ids,
                    *continuation_impl_node_id,
                )?;
                self.check_yield_for_spec(
                    error_context,
                    *external_function_id,
                    *spatial_operation,
                    self.spatial_spec,
                    self.spatial_spec_checker_opt.as_ref(),
                    yielded_impl_node_ids,
                    *continuation_impl_node_id,
                )?;

                let ir::ffi::ExternalFunction::CpuEffectfulOperation(effectful_operation) = & self.program.native_interface.external_functions[external_function_id.0] else { panic!("Not effectful operation"); };

                // Step 1: Check current -> callee edge
                assert_eq!(
                    effectful_operation.input_types.len(),
                    yielded_impl_node_ids.len()
                );
                for (argument_index, argument_node_id) in yielded_impl_node_ids.iter().enumerate() {
                    let node_type = self.node_types.remove(argument_node_id).unwrap();
                    check_slot_storage_type(
                        &self.program,
                        effectful_operation.input_types[argument_index],
                        &node_type,
                    );
                }

                // Check continuation against outputs
                let continuation_join_point = &self.node_join_points[continuation_impl_node_id];
                assert_eq!(
                    continuation_join_point.input_types.len(),
                    effectful_operation.output_types.len()
                );

                if let Some(NodeType::JoinPoint) = self.node_types.remove(continuation_impl_node_id)
                {
                    // Nothing, for now...
                } else {
                    panic!("Node at #{} is not a join point", continuation_impl_node_id)
                }

                // Step 2: Check callee -> continuation edge
                for (callee_output_index, callee_output_type) in
                    effectful_operation.output_types.iter().enumerate()
                {
                    /*assert_eq!(
                        *callee_output_type,
                        continuation_join_point.input_types[callee_output_index]
                    );*/
                }
            }
            ir::TailEdge::DebugHole { inputs } => {
                for input in inputs.iter() {
                    if let Some(_) = self.node_types.remove(input) {
                        // Nothing, for now...
                    } else {
                        panic!("Node at #{} does not exist", *input)
                    }
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
