use crate::debug_info::DebugInfo;
use crate::ir::{self, Funclet, Program, Type};
use crate::rust_wgpu_backend::code_generator;
use crate::rust_wgpu_backend::code_generator::{CodeGenerator, SubmissionId, VarId};
use crate::shadergen;
use crate::stable_vec::StableVec;
use crate::type_system;
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::default::{self, Default};
use std::fmt::Write;
use std::future::pending;

use crate::rust_wgpu_backend::ffi;

use super::code_generator::DispatcherId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JoinPointId(usize);

#[derive(Debug, Clone)]
enum NodeResult {
    None,
    LocalValue {
        var_id: VarId,
        storage_type: ir::ffi::TypeId,
    },
    Ref {
        var_id: VarId,
        storage_place: ir::Place,
        storage_type: ir::ffi::TypeId,
    },
    Fence {
        place: ir::Place,
        fence_id: code_generator::VarId,
    },
    Encoder {
        place: ir::Place,
    },
    Join {
        join_point_id: JoinPointId,
    },
    Buffer {
        storage_place: ir::Place,
        static_layout_opt: Option<ir::StaticBufferLayout>,
        var_id: VarId,
    },
}

impl NodeResult {
    fn get_var_id(&self) -> Option<VarId> {
        match self {
            NodeResult::LocalValue { var_id, .. } => Some(*var_id),
            NodeResult::Ref { var_id, .. } => Some(*var_id),
            NodeResult::Buffer { var_id, .. } => Some(*var_id),
            NodeResult::Fence { fence_id, .. } => Some(*fence_id),
            _ => None,
        }
    }

    fn get_storage_type(&self) -> Option<ir::StorageTypeId> {
        match self {
            NodeResult::LocalValue { storage_type, .. } => Some(*storage_type),
            NodeResult::Ref { storage_type, .. } => Some(*storage_type),
            _ => None,
        }
    }

    fn collect_vars(node_results: &[NodeResult]) -> Box<[VarId]> {
        let mut var_ids = Vec::<VarId>::new();

        for node_result in node_results.iter() {
            if let Some(var_id) = node_result.get_var_id() {
                var_ids.push(var_id);
            } else {
                if !matches!(node_result, NodeResult::Encoder { .. }) {
                    panic!(
                        "Node Result {:?} does not have an associated variable",
                        node_result
                    );
                }
            }
        }

        var_ids.into_boxed_slice()
    }
}

#[derive(Debug, Clone)]
struct RootJoinPoint {
    value_funclet_id: ir::FuncletId,
    input_types: Box<[ir::TypeId]>,
}

#[derive(Debug, Clone)]
struct SimpleJoinPoint {
    value_funclet_id: ir::FuncletId,
    scheduling_funclet_id: ir::FuncletId,
    captures: Box<[NodeResult]>,
    continuation_join_point_id: JoinPointId,
}

#[derive(Debug, Clone)]
struct SerializedJoinPoint {
    value_funclet_id: ir::FuncletId,
    scheduling_funclet_id: ir::FuncletId,
    //captures : Box<[NodeResult]>,
    continuation_join_point_id: JoinPointId,
    argument_ffi_types: Box<[ir::ffi::TypeId]>,
}

#[derive(Debug)]
enum JoinPoint {
    RootJoinPoint(RootJoinPoint),
    SimpleJoinPoint(SimpleJoinPoint),
    SerializedJoinPoint(SerializedJoinPoint),
    CallJoinPoint(SimpleJoinPoint),
}

#[derive(Debug, Default)]
struct JoinGraph {
    join_points: Vec<Option<JoinPoint>>,
}

impl JoinGraph {
    fn new() -> Self {
        Self {
            join_points: vec![],
        }
    }

    fn create(&mut self, join_point: JoinPoint) -> JoinPointId {
        let index = self.join_points.len();
        self.join_points.push(Some(join_point));
        JoinPointId(index)
    }

    fn move_join(&mut self, join_point_id: JoinPointId) -> JoinPoint {
        let mut join_point = None;
        std::mem::swap(&mut join_point, &mut self.join_points[join_point_id.0]);
        join_point.unwrap()
    }

    fn get_join(&self, join_point_id: JoinPointId) -> &JoinPoint {
        self.join_points[join_point_id.0].as_ref().unwrap()
    }
}

#[derive(Debug)]
enum SplitPoint {
    Call {
        return_node_results: Box<[NodeResult]>,
        continuation_join_point_id_opt: Option<JoinPointId>,
    },
    Jump {
        return_node_results: Box<[NodeResult]>,
        continuation_join_point_id: Option<JoinPointId>,
        funclet_id: ir::FuncletId,
    },
    Return {
        return_node_results: Box<[NodeResult]>,
        continuation_join_point_id_opt: Option<JoinPointId>,
        argument_ffi_types: Box<[ir::ffi::TypeId]>,
    },
    Yield {
        external_function_id: ir::ExternalFunctionId,
        yielded_node_results: Box<[NodeResult]>,
        continuation_join_point_id_opt: Option<JoinPointId>,
    },
    Select {
        return_node_results: Box<[NodeResult]>,
        condition_slot_id: VarId,
        true_funclet_id: ir::FuncletId,
        false_funclet_id: ir::FuncletId,
        continuation_join_point_id_opt: Option<JoinPointId>,
    },
    DynAlloc {
        buffer_node_result: NodeResult,
        success_funclet_id: ir::FuncletId,
        failure_funclet_id: ir::FuncletId,
        argument_node_results: Box<[NodeResult]>,
        dynamic_allocation_size_slot_ids: Box<[Option<VarId>]>,
        continuation_join_point_id_opt: Option<JoinPointId>,
    },
}

enum TraversalState {
    SelectIf {
        branch_input_node_results: Box<[NodeResult]>,
        condition_slot_id: VarId,
        true_funclet_id: ir::FuncletId,
        false_funclet_id: ir::FuncletId,
        continuation_join_point_id_opt: Option<JoinPointId>,
    },
    SelectElse {
        output_node_results: Box<[NodeResult]>,
        branch_input_node_results: Box<[NodeResult]>,
        false_funclet_id: ir::FuncletId,
        continuation_join_point_id_opt: Option<JoinPointId>,
    },
    SelectEnd {
        output_node_results: Box<[NodeResult]>,
        continuation_join_point_id_opt: Option<JoinPointId>,
    },
    DynAllocIf {
        buffer_node_result: NodeResult,
        success_funclet_id: ir::FuncletId,
        failure_funclet_id: ir::FuncletId,
        argument_node_results: Box<[NodeResult]>,
        dynamic_allocation_size_slot_ids: Box<[Option<VarId>]>,
        continuation_join_point_id_opt: Option<JoinPointId>,
    },
    DynAllocElse {
        output_node_results: Box<[NodeResult]>,
        failure_funclet_id: ir::FuncletId,
        argument_node_results: Box<[NodeResult]>,
        continuation_join_point_id_opt: Option<JoinPointId>,
    },
    DynAllocEnd {
        output_node_results: Box<[NodeResult]>,
        continuation_join_point_id_opt: Option<JoinPointId>,
    },
}

/// The state of the current funclet being inlined into a single Rust function.
struct InlineFuncletState {
    /// The funclet id of the current funclet
    funclet_id: ir::FuncletId,
    /// The output results of the current funclet which are the inputs to the next funclet
    current_out_node_results: Box<[NodeResult]>,
    /// The join point id of the current funclet
    join_point_id: Option<JoinPointId>,
    /// The traversal state of the current funclet if it is part of a branch
    traversal_state: Option<TraversalState>,
    /// Whether the funclet is inlined in a function call
    func_inline: bool,
    /// Whether the funclet is inlined in a branch
    branch_inline: bool,
}

#[derive(Debug)]
struct FuncletScopedState {
    value_funclet_id: ir::FuncletId,
    scheduling_funclet_id: ir::FuncletId,
    node_results: HashMap<ir::NodeId, NodeResult>,
}

impl FuncletScopedState {
    fn new(value_funclet_id: ir::FuncletId, scheduling_funclet_id: ir::FuncletId) -> Self {
        Self {
            value_funclet_id,
            scheduling_funclet_id,
            node_results: Default::default(),
        }
    }

    fn move_node_result(&mut self, node_id: ir::NodeId) -> Option<NodeResult> {
        self.node_results.remove(&node_id)
    }

    fn get_node_result(&self, node_id: ir::NodeId) -> Option<&NodeResult> {
        self.node_results.get(&node_id)
    }

    fn get_node_var_id(&self, node_id: ir::NodeId) -> Option<VarId> {
        self.node_results
            .get(&node_id)
            .map(|x| x.get_var_id())
            .flatten()
    }

    fn get_node_join_point_id(&self, node_id: ir::NodeId) -> Option<JoinPointId> {
        match &self.node_results[&node_id] {
            NodeResult::Join { join_point_id } => Some(*join_point_id),
            _ => None,
        }
    }

    fn move_node_join_point_id(&mut self, node_id: ir::NodeId) -> Option<JoinPointId> {
        let node_result_opt = self.node_results.remove(&node_id);

        if let Some(node_result) = node_result_opt {
            if let NodeResult::Join { join_point_id } = node_result {
                self.node_results.insert(node_id, NodeResult::None);
                return Some(join_point_id);
            } else {
                self.node_results.insert(node_id, node_result);
                return None;
            }
        }

        return None;
    }

    fn collect_vars_for_node_ids(&self, node_ids: &[ir::NodeId]) -> Box<[VarId]> {
        let mut var_ids = Vec::<VarId>::new();

        for &node_id in node_ids.iter() {
            if let Some(var_id) = self.get_node_var_id(node_id) {
                var_ids.push(var_id);
            } else {
                panic!(
                    "Node #{} (content: {:?}) does not have an associated variable",
                    node_id, self.node_results[&node_id]
                );
            }
        }

        var_ids.into_boxed_slice()
    }
}

fn check_storage_type_implements_value_type(
    program: &ir::Program,
    storage_type_id: ir::ffi::TypeId,
    value_type_id: ir::TypeId,
) {
    let storage_type = &program.native_interface.types[storage_type_id.0];
    let value_type = &program.types[value_type_id];
    /*match value_type
    {
        ir::Type::Integer{signed, width} =>
        {
            ()
        }
        _ => panic!("Unsupported")
    }*/
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
struct AbstractContinuationLink {
    funclet_id: ir::FuncletId,
    capture_count: usize,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
struct AbstractEntryPoint {
    continuation_chain: Vec<AbstractContinuationLink>,
}

#[derive(Default, Debug)]
struct PipelineContext {
    pending_funclet_ids: Vec<ir::FuncletId>,
    join_graph: JoinGraph,
    inline_joins: HashMap<usize, JoinPoint>,
}

impl PipelineContext {
    fn new() -> Self {
        Self {
            pending_funclet_ids: Default::default(),
            join_graph: JoinGraph::new(),
            inline_joins: Default::default(),
        }
    }
}

pub struct CodeGen<'program> {
    program: &'program ir::Program,
    debug_info: &'program DebugInfo,
    code_generator: CodeGenerator<'program>,
    print_codegen_debug_info: bool,
    generated_local_slot_ffi_type_map: HashMap<ir::TypeId, ir::ffi::TypeId>,
    default_usize_ffi_type_id: ir::ffi::TypeId,
    default_u64_ffi_type_id: ir::ffi::TypeId,
}

impl<'program> CodeGen<'program> {
    pub fn new(program: &'program ir::Program, debug_info: &'program DebugInfo) -> Self {
        let mut code_generator = CodeGenerator::new(&program.native_interface);
        let default_usize_ffi_type_id = code_generator.create_ffi_type(ir::ffi::Type::USize);
        let default_u64_ffi_type_id = code_generator.create_ffi_type(ir::ffi::Type::U64);
        Self {
            program: &program,
            debug_info,
            code_generator,
            print_codegen_debug_info: false,
            generated_local_slot_ffi_type_map: HashMap::new(),
            default_usize_ffi_type_id,
            default_u64_ffi_type_id,
        }
    }

    pub fn set_print_codgen_debug_info(&mut self, to: bool) {
        self.print_codegen_debug_info = to;
    }

    // The (rust) ffi type we need to refer to the data from a cpu function
    fn get_cpu_useable_type(&mut self, type_id: ir::TypeId) -> ir::ffi::TypeId {
        if let Some(ffi_type_id) = self.generated_local_slot_ffi_type_map.get(&type_id) {
            return *ffi_type_id;
        }

        let typ = &self.program.types[type_id];
        let ffi_type_id = match typ {
            ir::Type::NativeValue { storage_type } => *storage_type,
            ir::Type::Ref {
                storage_type,
                storage_place,
                buffer_flags,
            } => {
                let is_dynamically_sized =
                    match &self.program.native_interface.types[storage_type.0] {
                        ir::ffi::Type::ErasedLengthArray { element_type } => true,
                        _ => false,
                    };

                let typ = match (storage_place, is_dynamically_sized) {
                    (ir::Place::Local, true) => ir::ffi::Type::MutSlice {
                        element_type: *storage_type,
                    },
                    (ir::Place::Local, false) => ir::ffi::Type::MutRef {
                        element_type: *storage_type,
                    },
                    (ir::Place::Cpu, true) => ir::ffi::Type::CpuBufferSlice {
                        element_type: *storage_type,
                    },
                    (ir::Place::Cpu, false) => ir::ffi::Type::CpuBufferRef {
                        element_type: *storage_type,
                    },
                    (ir::Place::Gpu, true) => ir::ffi::Type::GpuBufferSlice {
                        element_type: *storage_type,
                    },
                    (ir::Place::Gpu, false) => ir::ffi::Type::GpuBufferRef {
                        element_type: *storage_type,
                    },
                };

                self.code_generator.create_ffi_type(typ)
            }
            ir::Type::Buffer {
                storage_place: ir::Place::Gpu,
                ..
            } => self
                .code_generator
                .create_ffi_type(ir::ffi::Type::GpuBufferAllocator),
            ir::Type::Buffer {
                storage_place: ir::Place::Cpu,
                ..
            } => self
                .code_generator
                .create_ffi_type(ir::ffi::Type::CpuBufferAllocator),
            ir::Type::Fence {
                queue_place: ir::Place::Gpu,
            } => self.code_generator.create_ffi_type(ir::ffi::Type::GpuFence),
            ir::Type::Encoder { .. } => self.code_generator.get_encoder_type(),
            _ => panic!("Not a valid type for referencing from the CPU: {:?}", typ),
        };

        self.generated_local_slot_ffi_type_map
            .insert(type_id, ffi_type_id);

        ffi_type_id
    }

    fn encode_do_node_gpu(
        &mut self,
        funclet_scoped_state: &mut FuncletScopedState,
        funclet_checker: &mut type_system::scheduling::FuncletChecker,
        node: &ir::Node,
        external_function_id: ir::ffi::ExternalFunctionId,
        input_slot_ids: &[VarId],
        output_slot_ids: &[VarId],
    ) {
        match node {
            ir::Node::CallFunctionClass {
                function_id,
                arguments,
                //dimensions,
            } => {
                assert!(self.program.function_classes[*function_id]
                    .external_function_ids
                    .contains(&external_function_id));

                let function =
                    &self.program.native_interface.external_functions[external_function_id.0];
                let kernel = function.get_gpu_kernel().unwrap();

                assert_eq!(input_slot_ids.len(), arguments.len());
                assert_eq!(output_slot_ids.len(), kernel.output_types.len());

                /*let mut input_slot_counts = HashMap::<VarId, usize>::from_iter(input_slot_ids.iter().chain(output_slot_ids.iter()).map(|slot_id| (* slot_id, 0usize)));
                let mut output_slot_bindings = HashMap::<VarId, Option<usize>>::from_iter(output_slot_ids.iter().map(|slot_id| (* slot_id, None)));
                for (binding_index, resource_binding) in function.resource_bindings.iter().enumerate()
                {
                    if let Some(index) = resource_binding.input
                    {
                        * input_slot_counts.get_mut(& input_slot_ids[index]).unwrap() += 1;
                    }

                    if let Some(index) = resource_binding.output
                    {
                        * output_slot_bindings.get_mut(& output_slot_ids[index]).unwrap() = Some(binding_index);
                    }
                }

                for (binding_index, resource_binding) in function.resource_bindings.iter().enumerate()
                {
                    if let Some(output_index) = resource_binding.output
                    {
                        let output_slot_id = output_slot_ids[output_index];
                        assert_eq!(input_slot_counts[& output_slot_id], 0);
                        assert_eq!(output_slot_bindings[& output_slot_id], Some(binding_index));

                        if let Some(input_index) = resource_binding.input
                        {
                            let input_slot_id = input_slot_ids[input_index];
                            assert_eq!(input_slot_counts[& input_slot_id], 1);

                            assert_eq!(placement_state.scheduling_state.get_slot_type_id(input_slot_id), placement_state.scheduling_state.get_slot_type_id(output_slot_id));

                            placement_state.scheduling_state.forward_slot(output_slot_id, input_slot_id);
                            let var_id = placement_state.slot_variable_ids.remove(& input_slot_id).unwrap();
                            let old = placement_state.slot_variable_ids.insert(output_slot_id, var_id);
                            assert!(old.is_none());
                        }
                    }
                }*/

                for (input_index, _) in arguments[kernel.dimensionality..].iter().enumerate() {
                    if let Some(forwarded_output_index) =
                        kernel.output_of_forwarding_input(input_index)
                    {
                        let input_slot_id = input_slot_ids[kernel.dimensionality + input_index];
                        let output_slot_id = output_slot_ids[forwarded_output_index];
                        //let var_id = placement_state.slot_variable_ids[&input_slot_id];
                        assert_eq!(output_slot_id, input_slot_id);
                        /* let old = placement_state
                            .slot_variable_ids
                            .insert(output_slot_id, var_id);
                        assert!(old.is_none());*/
                    }
                }

                use std::convert::TryInto;
                use std::iter::FromIterator;
                //use core::slice::SlicePattern;
                let dimension_map = |(index, x)| input_slot_ids[index];
                let argument_map = |(index, x)| input_slot_ids[kernel.dimensionality + index];

                let mut dimension_var_ids = Vec::from_iter(
                    arguments[..kernel.dimensionality]
                        .iter()
                        .enumerate()
                        .map(dimension_map),
                );
                if dimension_var_ids.len() < 3 {
                    for i in dimension_var_ids.len()..3 {
                        dimension_var_ids.push(
                            self.code_generator
                                .build_constant_int(1, self.default_usize_ffi_type_id),
                        );
                    }
                }
                let dimension_var_ids = dimension_var_ids.into_boxed_slice();
                let argument_var_ids = Vec::from_iter(
                    arguments[kernel.dimensionality..]
                        .iter()
                        .enumerate()
                        .map(argument_map),
                )
                .into_boxed_slice();
                let output_var_ids = output_slot_ids.iter().map(|x| *x).collect::<Box<[VarId]>>();

                let dimensions_slice: &[VarId] = &dimension_var_ids;
                self.code_generator.build_compute_dispatch_with_outputs(
                    kernel,
                    dimensions_slice
                        .try_into()
                        .expect("Expected 3 elements for dimensions"),
                    &argument_var_ids,
                    &output_var_ids,
                );
            }
            _ => panic!("Node cannot be encoded to the gpu"),
        }
    }

    fn encode_do_node_local_builtin(
        &mut self,
        funclet_scoped_state: &mut FuncletScopedState,
        funclet_checker: &mut type_system::scheduling::FuncletChecker,
        node: &ir::Node,
        input_slot_ids: &[VarId],
        output_slot_ids: &[VarId],
        output_slot_node_ids: &[ir::NodeId],
    ) {
        // To do: Do something about the value
        match node {
            ir::Node::Constant { value, type_id } => {
                let storage_type_id = funclet_scoped_state
                    .get_node_result(output_slot_node_ids[0])
                    .unwrap()
                    .get_storage_type()
                    .unwrap();

                let variable_id = match value {
                    ir::Constant::I64(value) => self
                        .code_generator
                        .build_constant_int(*value, storage_type_id),
                    ir::Constant::U64(value) => self
                        .code_generator
                        .build_constant_int(*value, storage_type_id),
                    ir::Constant::I32(value) => self
                        .code_generator
                        .build_constant_int(*value, storage_type_id),
                };
                check_storage_type_implements_value_type(&self.program, storage_type_id, *type_id);

                self.code_generator
                    .build_write_local_ref(output_slot_ids[0], variable_id);
            }
            ir::Node::Select {
                condition,
                true_case,
                false_case,
            } => {
                let true_case_storage_type = funclet_scoped_state
                    .get_node_result(*true_case)
                    .map(|x| x.get_storage_type())
                    .flatten();
                let false_case_storage_type = funclet_scoped_state
                    .get_node_result(*false_case)
                    .map(|x| x.get_storage_type())
                    .flatten();
                assert!(true_case_storage_type.is_some());
                assert_eq!(true_case_storage_type, false_case_storage_type);

                let variable_id = self.code_generator.build_select_hack(
                    input_slot_ids[0],
                    input_slot_ids[1],
                    input_slot_ids[2],
                );

                self.code_generator
                    .build_write_local_ref(output_slot_ids[0], variable_id);
            }
            _ => panic!("Cannot be scheduled as local builtin"),
        }
    }

    fn encode_do_node_local_external(
        &mut self,
        funclet_scoped_state: &mut FuncletScopedState,
        funclet_checker: &mut type_system::scheduling::FuncletChecker,
        node: &ir::Node,
        external_function_id: ir::ffi::ExternalFunctionId,
        input_slot_ids: &[VarId],
        output_slot_ids: &[VarId],
        output_slot_node_ids: &[ir::NodeId],
    ) {
        // To do: Do something about the value
        match node {
            ir::Node::CallFunctionClass {
                function_id,
                arguments,
            } => {
                assert!(self.program.function_classes[*function_id]
                    .external_function_ids
                    .contains(&external_function_id));

                let function =
                    &self.program.native_interface.external_functions[external_function_id.0];
                let cpu_operation = function.get_cpu_pure_operation().unwrap();

                use std::iter::FromIterator;

                let argument_var_ids = Vec::from_iter(
                    arguments
                        .iter()
                        .enumerate()
                        .map(|(index, x)| input_slot_ids[index]),
                )
                .into_boxed_slice();
                let raw_outputs = self
                    .code_generator
                    .build_external_cpu_function_call(external_function_id, &argument_var_ids);

                for (index, output_type_id) in cpu_operation.output_types.iter().enumerate() {
                    self.code_generator
                        .build_write_local_ref(output_slot_ids[index], raw_outputs[index]);
                }
            }
            _ => panic!("Cannot be scheduled as local external"),
        }
    }

    fn collect_join_graph_path(
        &mut self,
        join_graph: &mut JoinGraph,
        traversal_state_stack: &[TraversalState],
        mut default_join_point_id_opt: Option<JoinPointId>,
    ) -> Box<[JoinPointId]> {
        let mut path = Vec::<JoinPointId>::new();
        let mut traversal_state_iterator = traversal_state_stack.iter().rev();
        'outer_loop: loop {
            while let Some(default_join_point_id) = default_join_point_id_opt {
                default_join_point_id_opt = None;

                let join_point = join_graph.get_join(default_join_point_id);
                path.push(default_join_point_id);

                match join_point {
                    JoinPoint::RootJoinPoint(_) => {
                        // Depends on if we always interpret the root join as return?
                        //break 'outer_loop
                    }
                    JoinPoint::SimpleJoinPoint(simple_join_point) => {
                        default_join_point_id_opt =
                            Some(simple_join_point.continuation_join_point_id);
                    }
                    JoinPoint::SerializedJoinPoint(simple_join_point) => {
                        default_join_point_id_opt =
                            Some(simple_join_point.continuation_join_point_id);
                    }
                    _ => panic!(
                        "Jump to invalid join point #{:?}: {:?}",
                        default_join_point_id, join_point
                    ),
                }
            }

            assert!(default_join_point_id_opt.is_none());

            if let Some(traversal_state) = traversal_state_iterator.next() {
                default_join_point_id_opt = match traversal_state {
                    TraversalState::SelectIf {
                        continuation_join_point_id_opt,
                        ..
                    } => panic!("SelectIf shouldn't remain in the traversal stack"),
                    TraversalState::SelectElse {
                        continuation_join_point_id_opt,
                        ..
                    } => *continuation_join_point_id_opt,
                    TraversalState::SelectEnd {
                        continuation_join_point_id_opt,
                        ..
                    } => *continuation_join_point_id_opt,
                    TraversalState::DynAllocIf {
                        continuation_join_point_id_opt,
                        ..
                    } => panic!("DynAllocIf shouldn't remain in the traversal stack"),
                    TraversalState::DynAllocElse {
                        continuation_join_point_id_opt,
                        ..
                    } => *continuation_join_point_id_opt,
                    TraversalState::DynAllocEnd {
                        continuation_join_point_id_opt,
                        ..
                    } => *continuation_join_point_id_opt,
                    _ => panic!("Unimplemented"),
                }
            }

            if default_join_point_id_opt.is_none() {
                break 'outer_loop;
            }
        }
        path.into_boxed_slice()
    }

    fn build_push_serialized_join(&mut self, funclet_id: ir::FuncletId, captures: &[NodeResult]) {
        let mut input_storage_types = Vec::<ir::ffi::TypeId>::new();
        let mut output_storage_types = Vec::<ir::ffi::TypeId>::new();
        let mut capture_var_ids = Vec::<VarId>::new();
        let join_point_scheduling_funclet = &self.program.funclets[funclet_id];
        for (capture_index, captured_node_result) in captures.iter().enumerate() {
            let var_id = captured_node_result.get_var_id().unwrap();
            capture_var_ids.push(var_id);
            match captured_node_result {
                NodeResult::LocalValue { .. } => (),
                NodeResult::Ref { .. } => (),
                NodeResult::Buffer { .. } => (),
                NodeResult::Fence { .. } => (),
                _ => panic!("Not yet supported"),
            }
        }

        for (input_index, input_type) in
            join_point_scheduling_funclet.input_types.iter().enumerate()
        {
            input_storage_types.push(self.get_cpu_useable_type(*input_type));
        }

        for (output_index, output_type) in join_point_scheduling_funclet
            .output_types
            .iter()
            .enumerate()
        {
            output_storage_types.push(self.get_cpu_useable_type(*output_type));
        }

        self.code_generator.build_push_serialized_join(
            funclet_id,
            capture_var_ids.as_slice(),
            &input_storage_types[0..captures.len()],
            &input_storage_types[captures.len()..],
            &output_storage_types[0..],
        )
    }

    /// Compiles the current funclet and returns a tuple for
    /// any unhandled split points, which will be either returns or schedule calls.
    /// # Arguments
    /// * `current_funclet_id` - The funclet to compile
    /// * `current_output_node_results` - The output node results of the funclet which are the
    ///   inputs to the next funclet
    /// * `pipeline_context` - The pipeline context
    /// * `traversal_state_stack` - The traversal state stack, which will be pushed to if we need to
    ///   process an inlined funclet after the current funclet
    /// * `default_join_point_id_opt` - The default join point id to jump to after the current funclet
    ///   is compiled
    /// * `func_inline` - Whether the current funclet is being inlined as a call
    /// * `branch_inline` - Whether the current funclet is being inlined as a branch
    /// * `pipeline_rets` - The return types of the pipeline. Used to determine if its possible to
    ///  return early from the pipeline when we encounter a caiman `return`.
    /// # Returns
    /// * None for no unhandled split points, otherwise, a tuple of the
    ///     unhandled split point node results and a boolean indicating whether
    ///     the split point is a call
    fn process_current_funclet(
        &mut self,
        current_funclet_id: ir::FuncletId,
        current_output_node_results: &[NodeResult],
        pipeline_context: &mut PipelineContext,
        traversal_state_stack: &mut Vec<InlineFuncletState>,
        default_join_point_id_opt: &mut Option<JoinPointId>,
        func_inline: bool,
        branch_inline: bool,
        pipeline_rets: &[ffi::TypeId],
    ) -> Option<(Box<[NodeResult]>, bool)> {
        let split_point = self.compile_scheduling_funclet(
            current_funclet_id,
            &current_output_node_results,
            pipeline_context,
            default_join_point_id_opt,
            func_inline || branch_inline,
        );

        //println!("Split point: {:?}", split_point);
        match split_point {
            SplitPoint::Call {
                return_node_results,
                continuation_join_point_id_opt,
            } => {
                // schedule call
                *default_join_point_id_opt = continuation_join_point_id_opt;
                return Some((return_node_results, true));
            }
            SplitPoint::Jump {
                return_node_results,
                continuation_join_point_id,
                funclet_id,
            } => {
                traversal_state_stack.push(InlineFuncletState {
                    funclet_id,
                    current_out_node_results: return_node_results,
                    join_point_id: continuation_join_point_id,
                    traversal_state: None,
                    func_inline,
                    branch_inline,
                });
                return None;
            }
            SplitPoint::Return {
                return_node_results,
                continuation_join_point_id_opt,
                argument_ffi_types,
            } => {
                // if in a branch, return now, otherwise continue to the
                // next funclet (if there is one)
                if branch_inline {
                    *default_join_point_id_opt = continuation_join_point_id_opt;
                    let argument_var_ids = NodeResult::collect_vars(&return_node_results);
                    // build return so that we can return if we optimize away the continuation
                    self.code_generator.build_return(
                        &argument_var_ids,
                        &argument_ffi_types,
                        pipeline_rets.iter().eq(argument_ffi_types.iter()),
                    );
                    return None;
                } else {
                    return Some((return_node_results, true));
                }
            }
            SplitPoint::Yield {
                external_function_id,
                yielded_node_results,
                mut continuation_join_point_id_opt,
            } => {
                // Inline joins behave like serialized joins right now, so we don't
                // need to serialize anything here.

                let mut yielded_var_ids = Vec::<VarId>::new();
                for node_result in yielded_node_results.iter() {
                    let var_id = node_result.get_var_id().unwrap();
                    yielded_var_ids.push(var_id);
                }

                self.code_generator
                    .build_yield(external_function_id, yielded_var_ids.as_slice());

                // To do: Technically, we should insert a join point that recursively forces a split of the funclet once branches merge.
                // Should probably fix this by adding a new type of ir::Node::Join instead of inserting here and sifting upwards.

                //panic!("Not yet implemented");
                //current_funclet_id_opt = None;
                *default_join_point_id_opt = None;
                return None;
            }
            SplitPoint::Select {
                return_node_results,
                condition_slot_id,
                true_funclet_id,
                false_funclet_id,
                continuation_join_point_id_opt,
            } => {
                assert!(default_join_point_id_opt.is_none());
                traversal_state_stack.push(InlineFuncletState {
                    funclet_id: true_funclet_id,
                    current_out_node_results: return_node_results.clone(),
                    join_point_id: continuation_join_point_id_opt,
                    traversal_state: Some(TraversalState::SelectIf {
                        branch_input_node_results: return_node_results,
                        condition_slot_id,
                        true_funclet_id,
                        false_funclet_id,
                        continuation_join_point_id_opt,
                    }),
                    func_inline,
                    branch_inline: true,
                });
                return None;
            }
            SplitPoint::DynAlloc { .. } => {
                unreachable!();
                // assert!(default_join_point_id_opt.is_none());
                // traversal_state_stack.push(TraversalState::DynAllocIf {
                //     buffer_node_result,
                //     success_funclet_id,
                //     failure_funclet_id,
                //     argument_node_results,
                //     dynamic_allocation_size_slot_ids,
                //     continuation_join_point_id_opt,
                // });
                // return None;
            }
        }
    }

    /// Returns true if we should compile the current funclet, otherwise returns false
    /// # Arguments
    /// * `pipeline_context` - The pipeline context
    /// * `traversal_state` - The traversal state of the current funclet
    /// * `funclet_stack` - The funclet stack, which may be pushed to with a
    ///    funclet to process after the current funclet
    /// * `current_out_node_results` - The output node results of the current funclet
    ///     which are the inputs to the next funclet
    /// * `func_inline` - Whether the current funclet is being inlined as a call
    /// * `branch_inline` - Whether the current funclet is being inlined as a branch
    /// # Returns
    /// * True if we should compile the current funclet, otherwise returns false
    fn process_traversal_state(
        &mut self,
        pipeline_context: &mut PipelineContext,
        traversal_state: TraversalState,
        funclet_stack: &mut Vec<InlineFuncletState>,
        current_out_node_results: &[NodeResult],
        func_inline: bool,
        branch_inline: bool,
    ) -> bool {
        match traversal_state {
            TraversalState::SelectIf {
                branch_input_node_results,
                condition_slot_id: condition_var_id,
                true_funclet_id,
                false_funclet_id,
                continuation_join_point_id_opt,
            } => {
                let true_funclet = &self.program.funclets[true_funclet_id];
                let output_types = true_funclet
                    .output_types
                    .iter()
                    .map(|type_id| self.get_cpu_useable_type(*type_id))
                    .collect::<Box<[ir::ffi::TypeId]>>();
                let output_var_ids = self
                    .code_generator
                    .begin_if_else(condition_var_id, &output_types);
                let mut output_node_results = Vec::<NodeResult>::new();
                for (output_index, output_type) in true_funclet.output_types.iter().enumerate() {
                    // Joins capture by slot.  This is ok because joins can't escape the scope they were created in.  We'll reach None before leaving the scope.
                    let node_result = match &self.program.types[*output_type] {
                        ir::Type::NativeValue { storage_type, .. } => NodeResult::LocalValue {
                            var_id: output_var_ids[output_index],
                            storage_type: *storage_type,
                        },
                        ir::Type::Ref {
                            storage_type,
                            storage_place,
                            ..
                        } => NodeResult::Ref {
                            var_id: output_var_ids[output_index],
                            storage_place: *storage_place,
                            storage_type: *storage_type,
                        },
                        ir::Type::Fence { queue_place } => {
                            let fence_id = output_var_ids[output_index]; //self.code_generator.convert_var_to_gpu_fence(output_var_ids[output_index]);
                            NodeResult::Fence {
                                place: *queue_place,
                                fence_id,
                            }
                        }
                        ir::Type::Encoder { queue_place } => NodeResult::Encoder {
                            place: *queue_place,
                        },
                        x => panic!("Incorrect type: {:?}", x),
                    };
                    output_node_results.push(node_result);
                }
                funclet_stack.push(InlineFuncletState {
                    funclet_id: false_funclet_id,
                    current_out_node_results: branch_input_node_results.clone(),
                    join_point_id: continuation_join_point_id_opt,
                    traversal_state: Some(TraversalState::SelectElse {
                        output_node_results: output_node_results.into_boxed_slice(),
                        branch_input_node_results,
                        false_funclet_id,
                        continuation_join_point_id_opt,
                    }),
                    func_inline,
                    branch_inline: true,
                });
            }
            TraversalState::SelectElse {
                output_node_results,
                branch_input_node_results,
                false_funclet_id,
                continuation_join_point_id_opt,
            } => {
                self.code_generator
                    .end_if_begin_else(&NodeResult::collect_vars(&current_out_node_results));
                // self.build_push_serialized_join(false_funclet_id, &branch_input_node_results);
                funclet_stack.push(InlineFuncletState {
                    funclet_id: false_funclet_id,
                    current_out_node_results: branch_input_node_results,
                    join_point_id: continuation_join_point_id_opt,
                    traversal_state: Some(TraversalState::SelectEnd {
                        output_node_results,
                        continuation_join_point_id_opt,
                    }),
                    func_inline,
                    branch_inline: true,
                });
            }
            TraversalState::SelectEnd {
                output_node_results,
                continuation_join_point_id_opt,
            } => {
                self.code_generator
                    .end_else(&NodeResult::collect_vars(current_out_node_results));
                // do not process funclet (return false)
                return false;
            }
            _ => unimplemented!(),
        }
        true
    }

    fn compile_externally_visible_scheduling_funclet(
        &mut self,
        funclet_id: ir::FuncletId,
        pipeline_context: &mut PipelineContext,
        pipeline_rets: &[ffi::TypeId],
    ) {
        let funclet = &self.program.funclets[funclet_id];
        assert_eq!(funclet.kind, ir::FuncletKind::ScheduleExplicit);
        //let funclet_extra = & self.program.scheduling_funclet_extras[& funclet_id];
        let input_types = funclet
            .input_types
            .iter()
            .map(|type_id| self.get_cpu_useable_type(*type_id))
            .collect::<Box<[ir::ffi::TypeId]>>();
        let output_types = funclet
            .output_types
            .iter()
            .map(|type_id| self.get_cpu_useable_type(*type_id))
            .collect::<Box<[ir::ffi::TypeId]>>();
        let argument_variable_ids =
            self.code_generator
                .begin_funclet(funclet_id, &input_types, &output_types);

        let mut argument_node_results = Vec::<NodeResult>::new();

        for (index, input_type_id) in funclet.input_types.iter().enumerate() {
            let result = {
                use ir::Type;

                match &self.program.types[*input_type_id] {
                    ir::Type::NativeValue { storage_type } => {
                        argument_node_results.push(NodeResult::LocalValue {
                            var_id: argument_variable_ids[index],
                            storage_type: *storage_type,
                        });
                    }
                    ir::Type::Ref {
                        storage_type,
                        //queue_stage,
                        storage_place,
                        buffer_flags,
                    } => {
                        argument_node_results.push(NodeResult::Ref {
                            var_id: argument_variable_ids[index],
                            storage_place: *storage_place,
                            storage_type: *storage_type,
                        });
                    }
                    ir::Type::Fence { queue_place } => {
                        //panic!("Fences cannot currently cross a rust function boundary");
                        // To do
                        let fence_id = argument_variable_ids[index]; //self.code_generator.convert_var_to_gpu_fence(argument_variable_ids[index]);
                        argument_node_results.push(NodeResult::Fence {
                            place: *queue_place,
                            fence_id,
                        });
                    }
                    ir::Type::Buffer {
                        storage_place,
                        static_layout_opt,
                        ..
                    } => {
                        argument_node_results.push(NodeResult::Buffer {
                            storage_place: *storage_place,
                            static_layout_opt: *static_layout_opt,
                            var_id: argument_variable_ids[index],
                        });
                    }
                    ir::Type::Encoder { queue_place } => {
                        argument_node_results.push(NodeResult::Encoder {
                            place: *queue_place,
                        });
                    }
                    _ => panic!("Unimplemented"),
                }
            };
        }

        let mut default_join_point_id_opt = {
            let input_types = funclet.output_types.clone();
            let value_funclet_id = funclet
                .spec_binding
                .get_value_spec()
                .funclet_id_opt
                .unwrap();
            let join_point_id =
                pipeline_context
                    .join_graph
                    .create(JoinPoint::RootJoinPoint(RootJoinPoint {
                        value_funclet_id,
                        input_types,
                    }));
            Option::<JoinPointId>::Some(join_point_id)
        };

        let mut inline_funclet_stack = vec![InlineFuncletState {
            funclet_id,
            current_out_node_results: argument_node_results.into_boxed_slice(),
            join_point_id: default_join_point_id_opt,
            traversal_state: None,
            func_inline: false,
            branch_inline: false,
        }];

        while let Some(InlineFuncletState {
            funclet_id,
            mut current_out_node_results,
            join_point_id,
            traversal_state,
            func_inline,
            branch_inline,
        }) = inline_funclet_stack.pop()
        {
            let mut default_join_point_id_opt = join_point_id;
            if let Some(traversal_state) = traversal_state {
                if !self.process_traversal_state(
                    pipeline_context,
                    traversal_state,
                    &mut inline_funclet_stack,
                    &current_out_node_results,
                    func_inline,
                    branch_inline,
                ) {
                    continue;
                }
            }
            let mut force_process_join = false;
            if let Some((cur_out, force_process)) = self.process_current_funclet(
                funclet_id,
                &current_out_node_results,
                pipeline_context,
                &mut inline_funclet_stack,
                &mut default_join_point_id_opt,
                func_inline,
                branch_inline,
                pipeline_rets,
            ) {
                current_out_node_results = cur_out;
                force_process_join = force_process;
            }

            if inline_funclet_stack.is_empty() || force_process_join {
                if let Some(join_point_id) = default_join_point_id_opt {
                    default_join_point_id_opt = None;
                    let join_point = pipeline_context.join_graph.move_join(join_point_id);
                    println!("Continuing to {:?} {:?}", join_point_id, join_point);
                    // jump and select are handled
                    // so split point here is either for call or return
                    match &join_point {
                        JoinPoint::RootJoinPoint(_) | JoinPoint::SimpleJoinPoint(_) => {
                            // tail edge must be a return
                            let return_var_ids =
                                NodeResult::collect_vars(&current_out_node_results);
                            let types = &self.program.funclets[funclet_id].output_types;
                            let types: Vec<_> = types
                                .iter()
                                .map(|x| self.get_cpu_useable_type(*x))
                                .collect();
                            // if the output types don't match the pipeline return types,
                            // then there's no way for us to return adn we must
                            // call something off the join stack
                            let may_return = pipeline_rets.iter().eq(types.iter());
                            self.code_generator.build_return(
                                &return_var_ids,
                                &types.into_boxed_slice(),
                                may_return,
                            );
                        }
                        JoinPoint::CallJoinPoint(simple_join_point) => {
                            // schedule call
                            let mut input_node_results = Vec::<NodeResult>::new();
                            input_node_results.extend_from_slice(&simple_join_point.captures);
                            input_node_results.extend_from_slice(&current_out_node_results);

                            inline_funclet_stack.push(InlineFuncletState {
                                funclet_id: simple_join_point.scheduling_funclet_id,
                                current_out_node_results: input_node_results.into_boxed_slice(),
                                join_point_id: Some(simple_join_point.continuation_join_point_id),
                                traversal_state: None,
                                func_inline: true,
                                branch_inline,
                            });
                        }
                        JoinPoint::SerializedJoinPoint(serialized_join_point) => {
                            let argument_var_ids =
                                NodeResult::collect_vars(&current_out_node_results);
                            self.code_generator
                                .build_indirect_stack_jump_to_popped_serialized_join(
                                    &argument_var_ids,
                                    &serialized_join_point.argument_ffi_types,
                                );
                        }
                        _ => panic!(
                            "Jump to invalid join point #{:?}: {:?}",
                            join_point_id, join_point
                        ),
                    }

                    println!(
                        "{:?} {:?} {:?}",
                        funclet_id, default_join_point_id_opt, current_out_node_results
                    );
                }
            }
        }

        //assert!(current_funclet_id_opt.is_none());

        self.code_generator.end_funclet();
    }

    fn collect_local_inputs(
        &mut self,
        funclet_scoped_state: &FuncletScopedState,
        inputs: &[ir::NodeId],
    ) -> Box<[VarId]> {
        todo!("Collect local inputs");
        let mut input_var_ids = Vec::<VarId>::new();
        for input in inputs.iter() {
            let node_result = funclet_scoped_state.get_node_result(*input).unwrap();
            match node_result {
                NodeResult::LocalValue {
                    storage_type,
                    var_id,
                    ..
                } => {
                    input_var_ids.push(
                        self.code_generator
                            .build_borrow_local_ref(*var_id, *storage_type),
                    );
                }
                NodeResult::Ref { var_id, .. } => {
                    input_var_ids.push(*var_id);
                }
                _ => panic!("Unsupported"),
            }
        }

        return input_var_ids.into_boxed_slice();
    }

    /// Determines if a continuation of the given funclet can be optimized away.
    /// A funclet is trivial if it has no captures and it
    /// only contains phis, returns of all phi nodes, or jumps to other trivial funclets.
    /// # Arguments
    /// * `funclet_id` - The continuation target
    /// * `captures_len` - The number of captures for the continuation
    /// # Returns
    /// * True if the continuation can be optimized away, otherwise false
    fn is_trivial_funclet(&self, funclet_id: usize, captures_len: usize) -> bool {
        if captures_len != 0 {
            return false;
        }
        let f = &self.program.funclets[funclet_id];
        for node in f.nodes.iter() {
            if !matches!(
                node,
                ir::Node::Phi { .. } | ir::Node::DefaultJoin { .. } | ir::Node::InlineJoin { .. }
            ) {
                return false;
            }
        }
        match &f.tail_edge {
            ir::TailEdge::Return { return_values } => {
                let mut v: Vec<_> = return_values.iter().cloned().collect();
                v.sort();
                return_values.len() == f.input_types.len()
                    && return_values.iter().eq(v.iter())
                    && return_values
                        .iter()
                        .all(|x| matches!(f.nodes[*x], ir::Node::Phi { .. }))
            }
            ir::TailEdge::Jump {
                arguments: return_values,
                join,
            } => {
                let mut v: Vec<_> = return_values.iter().cloned().collect();
                v.sort();
                return_values.len() == f.input_types.len()
                    && return_values
                        .iter()
                        .all(|x| matches!(f.nodes[*x], ir::Node::Phi { .. }))
                    && return_values.iter().eq(v.iter())
                    && match &f.nodes[*join] {
                        ir::Node::InlineJoin {
                            funclet, captures, ..
                        } => self.is_trivial_funclet(*funclet, captures.len()),
                        _ => false,
                    }
            }
            _ => false,
        }
    }

    /// Inlines the pending join point
    /// # Arguments
    /// * `pending_inline_join` - The pending join which will be inlined. Must not be None.
    ///    The tuple contains the funclet id, the captures, the continuation join node id, and the current node id.
    fn do_inline_join(
        &mut self,
        funclet_scoped_state: &mut FuncletScopedState,
        pipeline_context: &mut PipelineContext,
        pending_inline_join: Option<(&ir::FuncletId, &Box<[ir::NodeId]>, &ir::NodeId, ir::NodeId)>,
    ) {
        let (funclet_id, captures, continuation_join_node_id, current_node_id) =
            pending_inline_join.unwrap();
        let mut captured_node_results = Vec::<NodeResult>::new();
        let join_funclet = &self.program.funclets[*funclet_id];

        // Join points can only be constructed for the value funclet they are created in
        assert_eq!(
            join_funclet
                .spec_binding
                .get_value_spec()
                .funclet_id_opt
                .unwrap(),
            funclet_scoped_state.value_funclet_id
        );

        for (capture_index, capture_node_id) in captures.iter().enumerate() {
            let node_result = funclet_scoped_state
                .move_node_result(*capture_node_id)
                .unwrap();
            captured_node_results.push(node_result);
        }

        let continuation_join_point_id = funclet_scoped_state
            .move_node_join_point_id(*continuation_join_node_id)
            .unwrap();
        let continuation_join_point = pipeline_context
            .join_graph
            .get_join(continuation_join_point_id);

        let join_point_id = pipeline_context
            .join_graph
            .create(JoinPoint::SimpleJoinPoint(SimpleJoinPoint {
                value_funclet_id: join_funclet
                    .spec_binding
                    .get_value_spec()
                    .funclet_id_opt
                    .unwrap(),
                scheduling_funclet_id: *funclet_id,
                captures: captured_node_results.into_boxed_slice(),
                continuation_join_point_id,
            }));
        funclet_scoped_state
            .node_results
            .insert(current_node_id, NodeResult::Join { join_point_id });
    }

    /// Serializes the given join point
    /// # Arguments
    /// * `pending_inline_join` - The pending join which will be serialized. Must not be None.
    ///     The tuple contains the funclet id, the captures, the continuation join node id, and the current node id.
    fn do_serialized_join(
        &mut self,
        funclet_scoped_state: &mut FuncletScopedState,
        pipeline_context: &mut PipelineContext,
        pending_inline_join: Option<(&ir::FuncletId, &Box<[ir::NodeId]>, &ir::NodeId, ir::NodeId)>,
    ) {
        let (funclet_id, captures, continuation_join_node_id, current_node_id) =
            pending_inline_join.unwrap();
        let mut captured_node_results = Vec::<NodeResult>::new();
        let join_funclet = &self.program.funclets[*funclet_id];

        // Join points can only be constructed for the value funclet they are created in
        assert_eq!(
            join_funclet
                .spec_binding
                .get_value_spec()
                .funclet_id_opt
                .unwrap(),
            funclet_scoped_state.value_funclet_id
        );

        for (capture_index, capture_node_id) in captures.iter().enumerate() {
            let node_result = funclet_scoped_state
                .move_node_result(*capture_node_id)
                .unwrap();
            captured_node_results.push(node_result);
        }

        let argument_ffi_types = join_funclet.input_types[captures.len()..]
            .iter()
            .map(|type_id| self.get_cpu_useable_type(*type_id))
            .collect::<Box<[ir::ffi::TypeId]>>();

        let continuation_join_point_id = funclet_scoped_state
            .move_node_join_point_id(*continuation_join_node_id)
            .unwrap();
        let continuation_join_point = pipeline_context
            .join_graph
            .get_join(continuation_join_point_id);

        self.build_push_serialized_join(*funclet_id, captured_node_results.as_slice());
        pipeline_context.pending_funclet_ids.push(*funclet_id);

        let join_point_id = pipeline_context
            .join_graph
            .create(JoinPoint::SerializedJoinPoint(SerializedJoinPoint {
                value_funclet_id: join_funclet
                    .spec_binding
                    .get_value_spec()
                    .funclet_id_opt
                    .unwrap(),
                scheduling_funclet_id: *funclet_id,
                argument_ffi_types,
                continuation_join_point_id,
            }));
        funclet_scoped_state
            .node_results
            .insert(current_node_id, NodeResult::Join { join_point_id });
    }

    /// Compiles the current funclet and returns the split point for the tail edge
    /// # Arguments
    /// * `funclet_id` - The funclet to compile
    /// * `argument_node_results` - The output node results of the previous funclet
    /// * `pipeline_context` - The pipeline context
    /// * `default_join_point_id_opt` - The default join point id to jump to after the current funclet
    ///   is compiled
    /// * `inline` - Whether the current funclet is being inlined as a call or branch
    fn compile_scheduling_funclet(
        &mut self,
        funclet_id: ir::FuncletId,
        argument_node_results: &[NodeResult],
        pipeline_context: &mut PipelineContext,
        default_join_point_id_opt: &mut Option<JoinPointId>,
        inline: bool,
    ) -> SplitPoint {
        let funclet = &self.program.funclets[funclet_id];
        assert_eq!(funclet.kind, ir::FuncletKind::ScheduleExplicit);

        let mut funclet_scoped_state = FuncletScopedState::new(
            funclet
                .spec_binding
                .get_value_spec()
                .funclet_id_opt
                .unwrap(),
            funclet_id,
        );
        let mut funclet_checker = type_system::scheduling::FuncletChecker::new(
            &self.program,
            funclet_id,
            funclet,
            &self.debug_info,
        );

        if self.print_codegen_debug_info {
            println!(
                "Compiling Funclet #{} with join {:?}...\n{:?}\n",
                funclet_id, default_join_point_id_opt, funclet
            );
        }

        // Keep track of any inline joins and decide what to do with them
        // based on the split point
        let mut pending_inline_join = None;
        for (current_node_id, node) in funclet.nodes.iter().enumerate() {
            self.code_generator
                .insert_comment(format!(" node #{}: {:?}", current_node_id, node).as_str());

            if self.print_codegen_debug_info {
                println!(
                    "#{} {:?} : {:?} {:?}",
                    current_node_id, node, pipeline_context, funclet_scoped_state
                );
            }

            match node {
                ir::Node::None => (),
                ir::Node::Phi { index } => {
                    // Phis must appear at the start of a scheduling funclet (so that node order reflects scheduling order)
                    assert_eq!(current_node_id, *index as usize);
                    funclet_scoped_state.node_results.insert(
                        current_node_id,
                        argument_node_results[*index as usize].clone(),
                    );
                }
                ir::Node::ExtractResult { node_id, index } => {
                    // Extracts must appear directly after the call (so that node order reflects scheduling order)
                    assert_eq!(current_node_id, *node_id + (*index as usize));

                    match &funclet_scoped_state.node_results[node_id] {
                        _ => panic!(
                            "Funclet #{} at node #{} {:?}: Node #{} does not have multiple returns",
                            funclet_id, current_node_id, node, node_id
                        ),
                    }
                }
                ir::Node::AllocTemporary {
                    place,
                    storage_type,
                    buffer_flags,
                } => match place {
                    ir::Place::Cpu => panic!("Unimplemented"),
                    ir::Place::Local => {
                        let var_id = self
                            .code_generator
                            .build_alloc_temp_local_ref(*storage_type);

                        funclet_scoped_state.node_results.insert(
                            current_node_id,
                            NodeResult::Ref {
                                var_id,
                                storage_place: *place,
                                storage_type: *storage_type,
                            },
                        );
                    }
                    ir::Place::Gpu => {
                        let buffer_var_id = self
                            .code_generator
                            .build_alloc_temp_gpu(*storage_type, *buffer_flags);
                        let var_id = self
                            .code_generator
                            .build_buffer_ref(buffer_var_id, *storage_type);

                        funclet_scoped_state.node_results.insert(
                            current_node_id,
                            NodeResult::Ref {
                                var_id,
                                storage_place: *place,
                                storage_type: *storage_type,
                            },
                        );
                    }
                },
                ir::Node::Drop {
                    node: dropped_node_id,
                } => {
                    funclet_scoped_state.move_node_result(*dropped_node_id);
                }
                ir::Node::LocalDoBuiltin {
                    operation:
                        ir::Quotient::Node {
                            node_id: operation_node_id,
                        },
                    inputs,
                    outputs,
                } => {
                    let operation_funclet_id = funclet
                        .spec_binding
                        .get_value_spec()
                        .funclet_id_opt
                        .unwrap();

                    let encoded_funclet = &self.program.funclets[operation_funclet_id];
                    let encoded_node = &encoded_funclet.nodes[*operation_node_id];

                    let input_slot_ids = funclet_scoped_state.collect_vars_for_node_ids(inputs);
                    //let input_slot_ids = self.collect_local_inputs(&funclet_scoped_state, inputs);
                    let output_slot_ids = funclet_scoped_state.collect_vars_for_node_ids(outputs);

                    self.encode_do_node_local_builtin(
                        &mut funclet_scoped_state,
                        &mut funclet_checker,
                        encoded_node,
                        &input_slot_ids,
                        &output_slot_ids,
                        outputs,
                    );
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
                    let operation_funclet_id = funclet
                        .spec_binding
                        .get_value_spec()
                        .funclet_id_opt
                        .unwrap();

                    let encoded_funclet = &self.program.funclets[operation_funclet_id];
                    let encoded_node = &encoded_funclet.nodes[*operation_node_id];

                    /*let mut input_slot_ids = Vec::<VarId>::new();
                    for input in inputs.iter() {
                        let node_result = funclet_scoped_state.get_node_result(*input).unwrap();
                        match node_result {
                            NodeResult::LocalValue { storage_type, var_id, .. } => { input_slot_ids.push(self.code_generator.build_borrow_local_ref(* var_id, * storage_type)); }
                            NodeResult::Ref { var_id, .. } => { input_slot_ids.push(* var_id); }
                            _ => panic!("Unsupported")
                        }
                    }
                    //funclet_scoped_state.collect_vars_for_node_ids(inputs);*/

                    let input_slot_ids = funclet_scoped_state.collect_vars_for_node_ids(inputs);
                    //let input_slot_ids = self.collect_local_inputs(&funclet_scoped_state, inputs);
                    let output_slot_ids = funclet_scoped_state.collect_vars_for_node_ids(outputs);

                    self.encode_do_node_local_external(
                        &mut funclet_scoped_state,
                        &mut funclet_checker,
                        encoded_node,
                        *external_function_id,
                        &input_slot_ids,
                        &output_slot_ids,
                        outputs,
                    );
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
                    let Some(NodeResult::Encoder{place}) = funclet_scoped_state.get_node_result(*encoder) else { panic!("No encoder"); };

                    assert_eq!(*place, ir::Place::Gpu);
                    let operation_funclet_id = funclet
                        .spec_binding
                        .get_value_spec()
                        .funclet_id_opt
                        .unwrap();

                    let encoded_funclet = &self.program.funclets[operation_funclet_id];
                    let encoded_node = &encoded_funclet.nodes[*operation_node_id];

                    let input_slot_ids = funclet_scoped_state.collect_vars_for_node_ids(inputs);
                    //let input_slot_ids = self.collect_local_inputs(&funclet_scoped_state, inputs);
                    let output_slot_ids = funclet_scoped_state.collect_vars_for_node_ids(outputs);

                    match place {
                        ir::Place::Local => panic!("EncodeDoExternal to Local is unsupported"),
                        ir::Place::Gpu => {
                            /*let opportunity = fusion_info.opportunities.iter().find(|schema| {
                                schema.bounds.start <= current_node_id
                                    && current_node_id <= schema.bounds.end
                            });
                            if let Some(schema) = opportunity {
                                if schema.bounds.start == current_node_id {
                                    self.encode_fuse_header_gpu(
                                        placement_state,
                                        &mut funclet_scoped_state,
                                        &mut funclet_checker,
                                        schema,
                                        *external_function_id
                                    );
                                }
                                self.encode_fused_node_gpu(
                                    placement_state,
                                    encoded_node,
                                    *external_function_id,
                                    & input_slot_ids,
                                    & output_slot_ids,
                                );
                            } else {
                                self.encode_do_node_gpu(
                                    placement_state,
                                    &mut funclet_scoped_state,
                                    &mut funclet_checker,
                                    encoded_node,
                                    *external_function_id,
                                    & input_slot_ids,
                                    & output_slot_ids,
                                );
                            }*/

                            self.encode_do_node_gpu(
                                &mut funclet_scoped_state,
                                &mut funclet_checker,
                                encoded_node,
                                *external_function_id,
                                &input_slot_ids,
                                &output_slot_ids,
                            );
                        }
                        ir::Place::Cpu => panic!("EncodeDoExternal to CPU is unsupported"),
                    }
                }
                ir::Node::LocalCopy { input, output } => {
                    let src_slot_id = funclet_scoped_state.get_node_var_id(*input).unwrap();
                    let dst_slot_id = funclet_scoped_state.get_node_var_id(*output).unwrap();

                    let (src_slot_id, src_place, src_type) = if let Some(NodeResult::Ref {
                        var_id,
                        storage_place,
                        storage_type,
                        ..
                    }) =
                        funclet_scoped_state.get_node_result(*input)
                    {
                        (*var_id, *storage_place, *storage_type)
                    } else {
                        panic!("Not a slot")
                    };

                    let (dst_slot_id, dst_place, dst_type) = if let Some(NodeResult::Ref {
                        var_id,
                        storage_place,
                        storage_type,
                        ..
                    }) =
                        funclet_scoped_state.get_node_result(*output)
                    {
                        (*var_id, *storage_place, *storage_type)
                    } else {
                        panic!("Not a slot")
                    };

                    assert_eq!(dst_type, src_type);

                    match (ir::Place::Local, dst_place, src_place) {
                        (ir::Place::Local, ir::Place::Local, ir::Place::Local) => {
                            let temp_var_id = self
                                .code_generator
                                .build_read_local_ref(src_slot_id, dst_type);
                            self.code_generator
                                .build_write_local_ref(dst_slot_id, temp_var_id);
                        }
                        (ir::Place::Local, ir::Place::Local, ir::Place::Gpu) => {
                            let temp_var_id = self
                                .code_generator
                                .encode_clone_local_data_from_buffer(src_slot_id, dst_type);
                            self.code_generator
                                .build_write_local_ref(dst_slot_id, temp_var_id);
                        }
                        _ => panic!("Unimplemented"),
                    }
                }
                ir::Node::ReadRef {
                    source,
                    storage_type,
                } => {
                    //let src_slot_id = funclet_scoped_state.get_node_slot_id(*source).unwrap();

                    let (src_slot_id, src_place) = if let Some(NodeResult::Ref {
                        var_id,
                        storage_place,
                        ..
                    }) =
                        funclet_scoped_state.get_node_result(*source)
                    {
                        (*var_id, *storage_place)
                    } else {
                        panic!("Not a slot")
                    };

                    assert_eq!(src_place, ir::Place::Local);

                    //panic!("To do: Implement slices");
                    let var_id = self
                        .code_generator
                        .build_read_local_ref(src_slot_id, *storage_type);
                    funclet_scoped_state.node_results.insert(
                        current_node_id,
                        NodeResult::LocalValue {
                            var_id,
                            storage_type: *storage_type,
                        },
                    );
                }
                ir::Node::WriteRef {
                    destination,
                    storage_type,
                    source,
                } => {
                    let (src_slot_id, src_place) =
                        if let Some(NodeResult::LocalValue { var_id, .. }) =
                            funclet_scoped_state.get_node_result(*source)
                        {
                            (*var_id, ir::Place::Local)
                        } else {
                            panic!("Not a slot")
                        };

                    let (dst_slot_id, dst_place) = if let Some(NodeResult::Ref {
                        var_id,
                        storage_place,
                        ..
                    }) =
                        funclet_scoped_state.get_node_result(*destination)
                    {
                        (*var_id, *storage_place)
                    } else {
                        panic!("Not a slot")
                    };

                    assert_eq!(src_place, ir::Place::Local);
                    assert_eq!(dst_place, ir::Place::Local);

                    //panic!("To do: Implement slices");
                    self.code_generator
                        .build_write_local_ref(dst_slot_id, src_slot_id);
                }
                ir::Node::BeginEncoding {
                    place,
                    event,
                    encoded,
                    fences,
                } => {
                    funclet_scoped_state
                        .node_results
                        .insert(current_node_id, NodeResult::Encoder { place: *place });
                }
                ir::Node::EncodeCopy {
                    input,
                    output,
                    encoder,
                } => {
                    let Some(NodeResult::Encoder{place}) = funclet_scoped_state.get_node_result(*encoder) else { panic!("No encoder"); };
                    let src_slot_id = funclet_scoped_state.get_node_var_id(*input).unwrap();
                    let dst_slot_id = funclet_scoped_state.get_node_var_id(*output).unwrap();

                    let (src_slot_id, src_place, src_storage_type) =
                        if let Some(NodeResult::Ref {
                            var_id,
                            storage_place,
                            storage_type,
                            ..
                        }) = funclet_scoped_state.get_node_result(*input)
                        {
                            (*var_id, *storage_place, *storage_type)
                        } else {
                            panic!("Not a slot")
                        };

                    let (dst_slot_id, dst_place, dst_storage_type) =
                        if let Some(NodeResult::Ref {
                            var_id,
                            storage_place,
                            storage_type,
                            ..
                        }) = funclet_scoped_state.get_node_result(*output)
                        {
                            (*var_id, *storage_place, *storage_type)
                        } else {
                            panic!("Not a slot")
                        };

                    assert_eq!(src_storage_type, dst_storage_type);

                    match (*place, dst_place, src_place) {
                        /*(ir::Place::Cpu, ir::Place::Cpu, ir::Place::Local) => {
                            // todo DG: I'm not confident this works
                            let src_var_id = placement_state.get_slot_var_id(src_slot_id).unwrap();
                            placement_state.update_slot_state(
                                dst_slot_id,
                                //ir::ResourceQueueStage::Ready,
                                src_var_id,
                            );
                        }*/
                        /*(ir::Place::Local, ir::Place::Local, ir::Place::Local) => {
                            let src_var_id = placement_state.get_slot_var_id(src_slot_id).unwrap();
                            placement_state.update_slot_state(
                                dst_slot_id,
                                ir::ResourceQueueStage::Ready,
                                src_var_id,
                            );
                        }
                        (ir::Place::Local, ir::Place::Local, ir::Place::Gpu) => {
                            let src_var_id = placement_state.get_slot_var_id(src_slot_id).unwrap();
                            let dst_var_id = self
                                .code_generator
                                .encode_clone_local_data_from_buffer(src_var_id);
                            placement_state.update_slot_state(
                                dst_slot_id,
                                //ir::ResourceQueueStage::Ready,
                                dst_var_id,
                            );
                        }*/
                        (ir::Place::Gpu, ir::Place::Gpu, ir::Place::Local) => {
                            self.code_generator.encode_copy_buffer_from_local_data(
                                dst_slot_id,
                                src_slot_id,
                                src_storage_type,
                            );
                        }
                        (ir::Place::Gpu, ir::Place::Gpu, ir::Place::Gpu) => {
                            self.code_generator.encode_copy_buffer_from_buffer(
                                dst_slot_id,
                                src_slot_id,
                                src_storage_type,
                            );
                        }
                        _ => panic!("Unimplemented"),
                    }
                }
                ir::Node::Submit { event, encoder } => {
                    let Some(NodeResult::Encoder{place}) = funclet_scoped_state.get_node_result(*encoder) else { panic!("No encoder"); };
                    match place {
                        ir::Place::Gpu => {
                            self.code_generator.flush_submission();
                        }
                        ir::Place::Cpu => {
                            self.code_generator.flush_submission();
                        }
                        _ => panic!("Unimplemented"),
                    }

                    let fence_id = self.code_generator.encode_gpu_fence();
                    funclet_scoped_state.node_results.insert(
                        current_node_id,
                        NodeResult::Fence {
                            place: *place,
                            fence_id,
                        },
                    );
                }
                ir::Node::SyncFence { fence, event } => {
                    if let Some(NodeResult::Fence {
                        place: fenced_place,
                        fence_id,
                    }) = funclet_scoped_state.move_node_result(*fence)
                    {
                        self.code_generator.sync_gpu_fence(fence_id);

                        assert_eq!(fenced_place, ir::Place::Gpu);
                    } else {
                        panic!("Expected fence")
                    }
                }
                ir::Node::StaticSplit {
                    spatial_operation:
                        ir::Quotient::Node {
                            node_id: spatial_spec_node_id,
                        },
                    node: buffer_impl_node_id,
                    sizes,
                    place,
                } => {
                    for (i, size) in sizes.iter().enumerate() {
                        let NodeResult::Buffer{static_layout_opt: Some(static_layout), ..} : &mut NodeResult = funclet_scoped_state.node_results.get_mut(&buffer_impl_node_id).unwrap() else { panic!("") };
                        let predecessor_layout =
                            static_layout.split_static(&self.program.native_interface, sizes[i]);
                        /*funclet_scoped_state.node_results.insert(
                            current_node_id,
                            NodeResult::Buffer {
                                var_id: ,
                                static_layout_opt: Some(predecessor_layout),
                                storage_place: *place,
                            },
                        );*/
                    }
                    /*funclet_scoped_state.node_results.insert(
                        current_node_id,
                        NodeResult::Buffer {
                            var_id: ,
                            static_layout_opt: Some(predecessor_layout),
                            storage_place: *place,
                        },
                    );*/
                }
                ir::Node::StaticMerge {
                    spatial_operation:
                        ir::Quotient::Node {
                            node_id: spatial_spec_node_id,
                        },
                    nodes: impl_node_ids,
                    place,
                } => {
                    let buffer_node_id = impl_node_ids[impl_node_ids.len() - 1];
                    for i in (0..(impl_node_ids.len() - 1)).rev() {
                        let NodeResult::Buffer{static_layout_opt: Some(predecessor_static_layout), ..} = funclet_scoped_state.move_node_result(impl_node_ids[i]).unwrap() else { panic!("") };
                        let NodeResult::Buffer{static_layout_opt: Some(static_layout), ..} : &mut NodeResult = funclet_scoped_state.node_results.get_mut(&buffer_node_id).unwrap() else { panic!("") };
                        static_layout.merge_static_left(
                            &self.program.native_interface,
                            predecessor_static_layout,
                        );
                    }
                }
                ir::Node::StaticSubAlloc {
                    node: buffer_node_id,
                    place,
                    storage_type,
                } => {
                    match *place {
                        ir::Place::Local => panic!("Unimplemented allocating locally"),
                        _ => {}
                    }

                    if let Some(NodeResult::Buffer {
                        var_id,
                        storage_place,
                        static_layout_opt,
                        ..
                    }) = funclet_scoped_state.node_results.get_mut(buffer_node_id)
                    {
                        assert_eq!(*storage_place, *place);
                        if let Some(static_layout) = static_layout_opt {
                            //let NodeResult::Buffer{static_layout_opt: Some(static_layout), ..} : &mut NodeResult = funclet_scoped_state.node_results.get_mut(&buffer_node_id).unwrap() else { panic!("") };
                            static_layout
                                .alloc_static(&self.program.native_interface, *storage_type);
                        }

                        let allocation_var_id = self
                            .code_generator
                            .build_buffer_suballocate_ref(*var_id, *storage_type);

                        funclet_scoped_state.node_results.insert(
                            current_node_id,
                            NodeResult::Ref {
                                var_id: allocation_var_id,
                                storage_place: *place,
                                storage_type: *storage_type,
                            },
                        );
                    } else {
                        panic!("Not a buffer")
                    }
                }
                ir::Node::DefaultJoin => {
                    if let Some(join_point_id) = *default_join_point_id_opt {
                        *default_join_point_id_opt = None;
                        funclet_scoped_state
                            .node_results
                            .insert(current_node_id, NodeResult::Join { join_point_id });
                    } else {
                        panic!("No default join point")
                    }
                }
                ir::Node::InlineJoin {
                    funclet,
                    captures,
                    continuation,
                } => {
                    assert!(pending_inline_join.is_none());
                    pending_inline_join = Some((funclet, captures, continuation, current_node_id));
                }
                ir::Node::SerializedJoin {
                    funclet: funclet_id,
                    captures,
                    continuation: continuation_join_node_id,
                } => {
                    let mut captured_node_results = Vec::<NodeResult>::new();
                    let join_funclet = &self.program.funclets[*funclet_id];

                    // Join points can only be constructed for the value funclet they are created in
                    assert_eq!(
                        join_funclet
                            .spec_binding
                            .get_value_spec()
                            .funclet_id_opt
                            .unwrap(),
                        funclet_scoped_state.value_funclet_id
                    );

                    for (capture_index, capture_node_id) in captures.iter().enumerate() {
                        let node_result = funclet_scoped_state
                            .move_node_result(*capture_node_id)
                            .unwrap();
                        captured_node_results.push(node_result);
                    }

                    let argument_ffi_types = join_funclet.input_types[captures.len()..]
                        .iter()
                        .map(|type_id| self.get_cpu_useable_type(*type_id))
                        .collect::<Box<[ir::ffi::TypeId]>>();

                    let continuation_join_point_id = funclet_scoped_state
                        .move_node_join_point_id(*continuation_join_node_id)
                        .unwrap();
                    let continuation_join_point = pipeline_context
                        .join_graph
                        .get_join(continuation_join_point_id);

                    self.build_push_serialized_join(*funclet_id, captured_node_results.as_slice());
                    pipeline_context.pending_funclet_ids.push(*funclet_id);

                    let join_point_id =
                        pipeline_context
                            .join_graph
                            .create(JoinPoint::SerializedJoinPoint(SerializedJoinPoint {
                                value_funclet_id: join_funclet
                                    .spec_binding
                                    .get_value_spec()
                                    .funclet_id_opt
                                    .unwrap(),
                                scheduling_funclet_id: *funclet_id,
                                argument_ffi_types,
                                continuation_join_point_id,
                            }));
                    funclet_scoped_state
                        .node_results
                        .insert(current_node_id, NodeResult::Join { join_point_id });
                }
                _ => panic!("Unknown node"),
            };
        }

        if self.print_codegen_debug_info {
            println!(
                "{:?} : {:?} {:?}",
                funclet.tail_edge, pipeline_context, funclet_scoped_state
            );
        }

        self.code_generator
            .insert_comment(format!(" tail edge: {:?}", funclet.tail_edge).as_str());
        let split_point = match &funclet.tail_edge {
            ir::TailEdge::Return { return_values } => {
                assert!(pending_inline_join.is_none());
                let encoded_value_funclet_id = funclet
                    .spec_binding
                    .get_value_spec()
                    .funclet_id_opt
                    .unwrap();
                let encoded_value_funclet = &self.program.funclets[encoded_value_funclet_id];

                let mut output_node_results = Vec::<NodeResult>::new();

                let mut argument_ffi_types = Vec::new();
                for ((return_index, return_node_id), return_type) in return_values
                    .iter()
                    .enumerate()
                    .zip(funclet.output_types.iter())
                {
                    let node_result = funclet_scoped_state
                        .move_node_result(*return_node_id)
                        .unwrap();
                    let tid = self.get_cpu_useable_type(*return_type);
                    argument_ffi_types.push(tid);
                    output_node_results.push(node_result);
                }

                SplitPoint::Return {
                    return_node_results: output_node_results.into_boxed_slice(),
                    continuation_join_point_id_opt: *default_join_point_id_opt,
                    argument_ffi_types: argument_ffi_types.into_boxed_slice(),
                }
            }
            ir::TailEdge::ScheduleCallYield {
                value_operation: _,
                timeline_operation: _,
                spatial_operation: _,
                external_function_id,
                yielded_nodes,
                continuation_join: continuation_join_node_id,
            } => {
                if pending_inline_join.is_some() {
                    self.do_serialized_join(
                        &mut funclet_scoped_state,
                        pipeline_context,
                        pending_inline_join,
                    );
                }
                let continuation_join_point_id = funclet_scoped_state
                    .move_node_join_point_id(*continuation_join_node_id)
                    .unwrap();
                let continuation_join_point = pipeline_context
                    .join_graph
                    .get_join(continuation_join_point_id);

                let mut output_node_results = Vec::<NodeResult>::new();

                for (yield_index, yielded_node_id) in yielded_nodes.iter().enumerate() {
                    let node_result = funclet_scoped_state
                        .move_node_result(*yielded_node_id)
                        .unwrap();
                    output_node_results.push(node_result);
                }

                SplitPoint::Yield {
                    external_function_id: *external_function_id,
                    yielded_node_results: output_node_results.into_boxed_slice(),
                    continuation_join_point_id_opt: Some(continuation_join_point_id),
                }
            }
            ir::TailEdge::Jump { join, arguments } => {
                if pending_inline_join.is_some() {
                    self.do_inline_join(
                        &mut funclet_scoped_state,
                        pipeline_context,
                        pending_inline_join,
                    );
                }
                let mut join_point_id = funclet_scoped_state.move_node_join_point_id(*join); //.unwrap();

                let mut argument_node_results = Vec::<NodeResult>::new();

                for (argument_index, argument_node_id) in arguments.iter().enumerate() {
                    let node_result = funclet_scoped_state
                        .move_node_result(*argument_node_id)
                        .unwrap();
                    argument_node_results.push(node_result);
                }

                assert!(join_point_id.is_some());
                assert!(default_join_point_id_opt.is_none());
                let continuation_join_point =
                    pipeline_context.join_graph.get_join(join_point_id.unwrap());
                let funclet_id = match continuation_join_point {
                    JoinPoint::SimpleJoinPoint(s) => s.scheduling_funclet_id,
                    JoinPoint::SerializedJoinPoint(s) => s.scheduling_funclet_id,
                    _ => unreachable!("Invalid join point for jump"),
                };
                SplitPoint::Jump {
                    return_node_results: argument_node_results.into_boxed_slice(),
                    continuation_join_point_id: join_point_id,
                    funclet_id,
                }
            }
            ir::TailEdge::ScheduleCall {
                value_operation,
                timeline_operation,
                spatial_operation,
                callee_funclet_id: callee_scheduling_funclet_id_ref,
                callee_arguments,
                continuation_join: continuation_join_node_id,
            } => {
                if pending_inline_join.is_some() {
                    let (funclet_id, captures, ..) = pending_inline_join.as_ref().unwrap();
                    if self.is_trivial_funclet(**funclet_id, captures.len()) {
                        self.do_inline_join(
                            &mut funclet_scoped_state,
                            pipeline_context,
                            pending_inline_join,
                        );
                    } else {
                        self.do_serialized_join(
                            &mut funclet_scoped_state,
                            pipeline_context,
                            pending_inline_join,
                        );
                    }
                }
                let callee_scheduling_funclet_id = *callee_scheduling_funclet_id_ref;

                let continuation_join_point_id = funclet_scoped_state
                    .move_node_join_point_id(*continuation_join_node_id)
                    .unwrap();
                let continuation_join_point = pipeline_context
                    .join_graph
                    .get_join(continuation_join_point_id);

                let callee_funclet = &self.program.funclets[callee_scheduling_funclet_id];
                assert_eq!(callee_funclet.kind, ir::FuncletKind::ScheduleExplicit);
                let callee_value_funclet_id = callee_funclet
                    .spec_binding
                    .get_value_spec()
                    .funclet_id_opt
                    .unwrap();
                let callee_value_funclet = &self.program.funclets[callee_value_funclet_id];
                assert_eq!(callee_value_funclet.kind, ir::FuncletKind::Value);

                let mut argument_node_results = Vec::<NodeResult>::new();
                for (argument_index, argument_node_id) in callee_arguments.iter().enumerate() {
                    let node_result = funclet_scoped_state
                        .move_node_result(*argument_node_id)
                        .unwrap();
                    argument_node_results.push(node_result);
                }

                assert!(default_join_point_id_opt.is_none());
                let join_point_id =
                    pipeline_context
                        .join_graph
                        .create(JoinPoint::CallJoinPoint(SimpleJoinPoint {
                            value_funclet_id: callee_value_funclet_id,
                            scheduling_funclet_id: callee_scheduling_funclet_id,
                            captures: vec![].into_boxed_slice(),
                            continuation_join_point_id,
                        }));
                SplitPoint::Call {
                    return_node_results: argument_node_results.into_boxed_slice(),
                    continuation_join_point_id_opt: Some(join_point_id),
                }
            }
            ir::TailEdge::ScheduleSelect {
                value_operation,
                timeline_operation,
                spatial_operation,
                condition: condition_slot_node_id,
                callee_funclet_ids,
                callee_arguments,
                continuation_join: continuation_join_node_id,
            } => {
                if pending_inline_join.is_some() {
                    let (funclet_id, captures, ..) = pending_inline_join.as_ref().unwrap();
                    if self.is_trivial_funclet(**funclet_id, captures.len()) {
                        self.do_inline_join(
                            &mut funclet_scoped_state,
                            pipeline_context,
                            pending_inline_join,
                        );
                    } else {
                        self.do_serialized_join(
                            &mut funclet_scoped_state,
                            pipeline_context,
                            pending_inline_join,
                        );
                    }
                }
                let condition_slot_id = funclet_scoped_state
                    .get_node_var_id(*condition_slot_node_id)
                    .unwrap();

                let continuation_join_point_id = funclet_scoped_state
                    .move_node_join_point_id(*continuation_join_node_id)
                    .unwrap();
                let continuation_join_point = pipeline_context
                    .join_graph
                    .get_join(continuation_join_point_id);

                assert_eq!(callee_funclet_ids.len(), 2);
                let true_funclet_id = callee_funclet_ids[0];
                let false_funclet_id = callee_funclet_ids[1];
                let true_funclet = &self.program.funclets[true_funclet_id];
                let false_funclet = &self.program.funclets[false_funclet_id];

                let current_value_funclet =
                    &self.program.funclets[funclet_scoped_state.value_funclet_id];
                assert_eq!(current_value_funclet.kind, ir::FuncletKind::Value);

                assert_eq!(callee_arguments.len(), true_funclet.input_types.len());
                assert_eq!(callee_arguments.len(), false_funclet.input_types.len());

                let mut argument_node_results = Vec::<NodeResult>::new();
                for (argument_index, argument_node_id) in callee_arguments.iter().enumerate() {
                    let node_result = funclet_scoped_state
                        .move_node_result(*argument_node_id)
                        .unwrap();
                    argument_node_results.push(node_result);
                }

                assert!(default_join_point_id_opt.is_none());
                SplitPoint::Select {
                    return_node_results: argument_node_results.into_boxed_slice(),
                    condition_slot_id,
                    true_funclet_id,
                    false_funclet_id,
                    continuation_join_point_id_opt: Some(continuation_join_point_id),
                }
            }
            /*ir::TailEdge::DynamicAllocFromBuffer {
                buffer: buffer_node_id,
                dynamic_allocation_size_slots,
                success_funclet_id,
                failure_funclet_id,
                arguments,
                continuation_join: continuation_join_node_id,
            } => {
                let buffer_node_result = funclet_scoped_state
                    .get_node_result(*buffer_node_id)
                    .unwrap()
                    .clone();

                let continuation_join_point_id = funclet_scoped_state
                    .move_node_join_point_id(*continuation_join_node_id)
                    .unwrap();
                let continuation_join_point = placement_state
                    .join_graph
                    .get_join(continuation_join_point_id);

                let true_funclet_id = success_funclet_id;
                let false_funclet_id = failure_funclet_id;
                let true_funclet = &self.program.funclets[*true_funclet_id];
                let false_funclet = &self.program.funclets[*false_funclet_id];
                //let true_funclet_extra = & self.program.scheduling_funclet_extras[& true_funclet_id];
                //let false_funclet_extra = & self.program.scheduling_funclet_extras[& false_funclet_id];

                assert_eq!(
                    funclet_scoped_state.value_funclet_id,
                    true_funclet
                        .spec_binding
                        .get_value_spec()
                        .funclet_id_opt
                        .unwrap()
                );
                assert_eq!(
                    funclet_scoped_state.value_funclet_id,
                    false_funclet
                        .spec_binding
                        .get_value_spec()
                        .funclet_id_opt
                        .unwrap()
                );

                assert_eq!(
                    arguments.len() + dynamic_allocation_size_slots.len(),
                    true_funclet.input_types.len()
                );
                assert_eq!(arguments.len(), false_funclet.input_types.len());

                // Do these first before they move
                let mut dynamic_allocation_size_slot_ids = Vec::<Option<VarId>>::new();
                for (allocation_index, allocation_size_node_id_opt) in
                    dynamic_allocation_size_slots.iter().enumerate()
                {
                    if let Some(allocation_size_node_id) = allocation_size_node_id_opt {
                        let node_result = funclet_scoped_state
                            .get_node_result(*allocation_size_node_id)
                            .unwrap();
                        if let NodeResult::Slot { slot_id, .. } = node_result {
                            dynamic_allocation_size_slot_ids.push(Some(*slot_id));
                        } else {
                            panic!("Allocation size is not a slot")
                        }
                    } else {
                        dynamic_allocation_size_slot_ids.push(None);
                    }
                }

                let mut argument_node_results = Vec::<NodeResult>::new();
                for (argument_index, argument_node_id) in arguments.iter().enumerate() {
                    let node_result = funclet_scoped_state
                        .move_node_result(*argument_node_id)
                        .unwrap();
                    argument_node_results.push(node_result);
                }

                assert!(default_join_point_id_opt.is_none());
                SplitPoint::DynAlloc {
                    buffer_node_result,
                    success_funclet_id: *success_funclet_id,
                    failure_funclet_id: *failure_funclet_id,
                    argument_node_results: argument_node_results.into_boxed_slice(),
                    dynamic_allocation_size_slot_ids: dynamic_allocation_size_slot_ids
                        .into_boxed_slice(),
                    continuation_join_point_id_opt: Some(continuation_join_point_id),
                }
                //panic!("Unimplemented")
            }*/
            _ => panic!("Umimplemented"),
        };
        split_point
    }

    fn generate_pipeline(&mut self, pipeline: &ir::Pipeline) {
        let entry_funclet_id: ir::FuncletId = pipeline.entry_funclet;
        let pipeline_name: &str = pipeline.name.as_str();
        let entry_funclet = &self.program.funclets[entry_funclet_id];
        assert_eq!(entry_funclet.kind, ir::FuncletKind::ScheduleExplicit);

        let mut pipeline_context = PipelineContext::new();
        pipeline_context.pending_funclet_ids.push(entry_funclet_id);

        self.code_generator.begin_pipeline(pipeline_name);

        let mut visited_funclet_ids = HashSet::<ir::FuncletId>::new();
        let output_types = entry_funclet
            .output_types
            .iter()
            .map(|type_id| self.get_cpu_useable_type(*type_id))
            .collect::<Box<[ir::ffi::TypeId]>>();

        while let Some(funclet_id) = pipeline_context.pending_funclet_ids.pop() {
            if !visited_funclet_ids.contains(&funclet_id) {
                self.compile_externally_visible_scheduling_funclet(
                    funclet_id,
                    &mut pipeline_context,
                    &output_types,
                );

                assert!(visited_funclet_ids.insert(funclet_id));
            }
        }

        // Get effectful operations and use as yield points for now

        if let Some(effect_id) = pipeline.effect_id_opt {
            let mut ffi_yield_points =
                Vec::<(ffi::ExternalFunctionId, code_generator::YieldPoint)>::new();
            match &self.program.native_interface.effects[effect_id.0] {
                ffi::Effect::Unrestricted => {
                    for (function_index, function) in
                        self.program.native_interface.external_functions.iter()
                    {
                        let cpu_effectful_operation =
                            if let Some(op) = function.get_cpu_effectful_operation() {
                                op
                            } else {
                                break;
                            };
                        let mut ffi_yield_point: code_generator::YieldPoint = Default::default();
                        ffi_yield_point.name = cpu_effectful_operation.name.clone();
                        ffi_yield_point.yielded_types = cpu_effectful_operation.input_types.clone();
                        ffi_yield_point.resuming_types =
                            cpu_effectful_operation.output_types.clone();
                        ffi_yield_points
                            .push((ffi::ExternalFunctionId(function_index), ffi_yield_point));
                    }
                }
                ffi::Effect::FullyConnected {
                    effectful_function_ids,
                } => {
                    for &external_function_id in effectful_function_ids.iter() {
                        let cpu_effectful_operation =
                            self.program.native_interface.external_functions
                                [external_function_id.0]
                                .get_cpu_effectful_operation()
                                .unwrap();
                        let mut ffi_yield_point: code_generator::YieldPoint = Default::default();
                        ffi_yield_point.name = cpu_effectful_operation.name.clone();
                        ffi_yield_point.yielded_types = cpu_effectful_operation.input_types.clone();
                        ffi_yield_point.resuming_types =
                            cpu_effectful_operation.output_types.clone();
                        ffi_yield_points.push((external_function_id, ffi_yield_point));
                    }
                }
            }
            let input_types = entry_funclet
                .input_types
                .iter()
                .map(|type_id| self.get_cpu_useable_type(*type_id))
                .collect::<Box<[ir::ffi::TypeId]>>();
            self.code_generator.emit_yieldable_pipeline_entry_point(
                entry_funclet_id,
                &input_types,
                &output_types,
                ffi_yield_points.as_slice(),
            );
        } else {
            let input_types = entry_funclet
                .input_types
                .iter()
                .map(|type_id| self.get_cpu_useable_type(*type_id))
                .collect::<Box<[ir::ffi::TypeId]>>();
            let output_types = entry_funclet
                .output_types
                .iter()
                .map(|type_id| self.get_cpu_useable_type(*type_id))
                .collect::<Box<[ir::ffi::TypeId]>>();
            self.code_generator.emit_oneshot_pipeline_entry_point(
                entry_funclet_id,
                &input_types,
                &output_types,
            );
        }

        self.code_generator.end_pipeline();
    }

    pub fn generate<'codegen>(&'codegen mut self) -> String {
        for pipeline in self.program.pipelines.iter() {
            self.generate_pipeline(pipeline);
        }
        return self.code_generator.finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir;
}
