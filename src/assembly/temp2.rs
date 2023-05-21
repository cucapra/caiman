#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Node {
    None,
    Phi { index: usize },
    ExtractResult { node_id: NodeId, index: usize },
    Constant { value: String, type_id: TypeId },
    CallValueFunction { function_id: ValueFunctionId, arguments: Box<[NodeId]> },
    Select { condition: NodeId, true_case: NodeId, false_case: NodeId },
    CallExternalCpu { external_function_id: ExternalFunctionId, arguments: Box<[NodeId]> },
    CallExternalGpuCompute { external_function_id: ExternalFunctionId, dimensions: Box<[NodeId]>, arguments: Box<[NodeId]> },
    AllocTemporary { place: ir::Place, storage_type: StorageTypeId, operation: RemoteNodeId },
    UnboundSlot { place: ir::Place, storage_type: StorageTypeId, operation: RemoteNodeId },
    Drop { node: NodeId },
    StaticAllocFromStaticBuffer { buffer: NodeId, place: ir::Place, storage_type: StorageTypeId, operation: RemoteNodeId },
    EncodeDo { place: ir::Place, operation: RemoteNodeId, inputs: Box<[NodeId]>, outputs: Box<[NodeId]> },
    EncodeCopy { place: ir::Place, input: NodeId, output: NodeId },
    Submit { place: ir::Place, event: RemoteNodeId },
    EncodeFence { place: ir::Place, event: RemoteNodeId },
    SyncFence { place: ir::Place, fence: NodeId, event: RemoteNodeId },
    InlineJoin { funclet: FuncletId, captures: Box<[NodeId]>, continuation: NodeId },
    SerializedJoin { funclet: FuncletId, captures: Box<[NodeId]>, continuation: NodeId },
    DefaultJoin,
    SubmissionEvent { here_place: ir::Place, there_place: ir::Place, local_past: NodeId },
    SynchronizationEvent { here_place: ir::Place, there_place: ir::Place, local_past: NodeId, remote_local_past: NodeId },
    SeparatedLinearSpace { place: ir::Place, space: NodeId },
    MergedLinearSpace { place: ir::Place, spaces: Box<[NodeId]> },
}
impl Node {
    pub fn map_referenced_nodes(&self,
                                mut f: impl FnMut(NodeId) -> NodeId) -> Self {
        match self {
            Self::None => Self::None,
            Self::Phi { index } => Self::Phi {
                index: index.clone()
            },
            Self::ExtractResult { node_id, index } => Self::ExtractResult {
                node_id: f(node_id.clone()),
                index: index.clone(),
            },
            Self::Constant { value, type_id } => Self::Constant {
                value: value.clone(),
                type_id: type_id.clone(),
            },
            Self::CallValueFunction { function_id, arguments } => Self::CallValueFunction {
                function_id: function_id.clone(),
                arguments: arguments.iter().map(|op| f(op.clone())).collect(),
            },
            Self::Select { condition, true_case, false_case } => Self::Select {
                condition: f(condition.clone()),
                true_case: f(true_case.clone()),
                false_case: f(false_case.clone()),
            },
            Self::CallExternalCpu { external_function_id, arguments } => Self::CallExternalCpu {
                external_function_id: external_function_id.clone(),
                arguments: arguments.iter().map(|op| f(op.clone())).collect(),
            },
            Self::CallExternalGpuCompute { external_function_id, dimensions, arguments } => Self::CallExternalGpuCompute {
                external_function_id: external_function_id.clone(),
                dimensions: dimensions.iter().map(|op| f(op.clone())).collect(),
                arguments: arguments.iter().map(|op| f(op.clone())).collect(),
            },
            Self::AllocTemporary { place, storage_type, operation } => Self::AllocTemporary {
                place: place.clone(),
                storage_type: storage_type.clone(),
                operation: operation.clone(),
            },
            Self::UnboundSlot { place, storage_type, operation } => Self::UnboundSlot {
                place: place.clone(),
                storage_type: storage_type.clone(),
                operation: operation.clone(),
            },
            Self::Drop { node } => Self::Drop {
                node: f(node.clone())
            },
            Self::StaticAllocFromStaticBuffer { buffer, place, storage_type, operation } => Self::StaticAllocFromStaticBuffer {
                buffer: f(buffer.clone()),
                place: place.clone(),
                storage_type: storage_type.clone(),
                operation: operation.clone(),
            },
            Self::EncodeDo { place, operation, inputs, outputs } => Self::EncodeDo {
                place: place.clone(),
                operation: operation.clone(),
                inputs: inputs.iter().map(|op| f(op.clone())).collect(),
                outputs: outputs.iter().map(|op| f(op.clone())).collect(),
            },
            Self::EncodeCopy { place, input, output } => Self::EncodeCopy {
                place: place.clone(),
                input: f(input.clone()),
                output: f(output.clone()),
            },
            Self::Submit { place, event } => Self::Submit {
                place: place.clone(),
                event: event.clone(),
            },
            Self::EncodeFence { place, event } => Self::EncodeFence {
                place: place.clone(),
                event: event.clone(),
            },
            Self::SyncFence { place, fence, event } => Self::SyncFence {
                place: place.clone(),
                fence: f(fence.clone()),
                event: event.clone(),
            },
            Self::InlineJoin { funclet, captures, continuation } => Self::InlineJoin {
                funclet: funclet.clone(),
                captures: captures.iter().map(|op| f(op.clone())).collect(),
                continuation: f(continuation.clone()),
            },
            Self::SerializedJoin { funclet, captures, continuation } => Self::SerializedJoin {
                funclet: funclet.clone(),
                captures: captures.iter().map(|op| f(op.clone())).collect(),
                continuation: f(continuation.clone()),
            },
            Self::DefaultJoin => Self::DefaultJoin,
            Self::SubmissionEvent { here_place, there_place, local_past } => Self::SubmissionEvent {
                here_place: here_place.clone(),
                there_place: there_place.clone(),
                local_past: f(local_past.clone()),
            },
            Self::SynchronizationEvent { here_place, there_place, local_past, remote_local_past } => Self::SynchronizationEvent {
                here_place: here_place.clone(),
                there_place: there_place.clone(),
                local_past: f(local_past.clone()),
                remote_local_past: f(remote_local_past.clone()),
            },
            Self::SeparatedLinearSpace { place, space } => Self::SeparatedLinearSpace {
                place: place.clone(),
                space: f(space.clone()),
            },
            Self::MergedLinearSpace { place, spaces } => Self::MergedLinearSpace {
                place: place.clone(),
                spaces: spaces.iter().map(|op| f(op.clone())).collect(),
            },
        }
    }
}