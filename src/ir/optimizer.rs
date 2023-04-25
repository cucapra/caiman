use crate::ir;
use std::collections::{HashMap, HashSet};

pub fn optimize_program(prog: &mut ir::Program) {
    for (funcId, func) in prog.funclets.iter() {
        if (func.kind == ir::FuncletKind::ScheduleExplicit) {
            let fused = kernel_fusion(prog, &func.nodes);
            // TODO: I don't need to remap nodes for kernel kernel_fusion
            // if I insert none nodes instead.
        }
    }
}

/*
struct DispatchSchedule {
    /// The external function ID of the dispatched kernel.
    id: usize,
    /// IDs of the scheduling nodes providing the kernel dimensions,
    /// if they're assigned.
    dims: [Option<ir::NodeId>; 3],
    /// IDs of the kernel inputs in the order they appear.
    inputs: Vec<ir::NodeId>,
    /// IDs of the kernel outputs in the order they appear.
    outputs: Vec<ir::NodeId>,
}

impl DispatchSchedule {
    fn from_node(prog: &ir::Program, node: &ir::Node) -> Option<Self> {
        use ir::Node::{CallExternalGpuCompute, EncodeDo};
        let EncodeDo { operation, inputs, outputs, .. } = node else {
            return None;
        };
        let remote = prog.funclets[operation.funclet_id].nodes[operation.node_id];
        let CallExternalGpuCompute { external_function_id, dimensions, .. } = node else {
            return None;
        };
        let mut true_dims = [None; 3];
        for i in 0..dimensions.len() {
            true_dims[i] = Some(inputs[i]);
        }
        let true_inputs = Vec::from(&inputs[dimensions.len()..]);
        let true_outputs = Vec::from(&outputs[..]);
        return Some(Self {
            id: *external_function_id,
            dims: true_dims,
            inputs: true_inputs,
            outputs: true_outputs,
        });
    }
    fn merge(&self, prog: &ir::Program, other: &Self) -> Option<Self> {
        for (ours, theirs) in self.dims.iter().zip(other.dims.iter()) {
            // This could be improved via constant propagation, but that's
            // a task for another time...
            if ours != theirs {
                return None;
            }
        }

        // Dimensions match... let's try to fuse!
        todo!("cheesy petes!");
    }
}
*/
/// Attempts to fuse GPU kernel dispatches. (CPU dispatches should be "fused" by
/// Rust inlining if it's actually a performance win.)
///
/// Since we respect existing schedules, some constraints must hold:
///     1. The kernel dispatches must occur sequentially.
///     2. The kernel dispatches' dimensions must match.
///     3. The kernels' local sizes must match.
///
/// Kernel fusion can conservatively elide arguments if those arguments are
/// initially uninitialized, written before any reads, and scalar.
fn kernel_fusion(prog: &ir::Program, nodes: &[ir::Node]) -> Vec<ir::Node> {
    use ir::Node::*;

    let mut out = Vec::with_capacity(nodes.len());

    todo!("kernel fusion unimplemented");
    return out;
}

/// Eliminates unused temporaries (and any copies into them).
///
/// This is a scheduling-exclusive optimization. We could easily extend it to
/// the value language, as done in the `transformations` branch, but that can
/// disrupt schedules and mess with the "specification" of value functions, so
/// it's probably not a great idea.
///
/// This could also be extended to general DCE, but post-explication schedules
/// should be explicit. Temporaries are an exception since the compiler gets to
/// choose how to allocate them (including to *not* allocate them).
fn unused_temp_elimination(prog: &ir::Program, nodes: &[ir::Node]) -> Vec<ir::Node> {
    dbg!("unused temp elimination unimplemented");
    return Vec::from(nodes);
}
