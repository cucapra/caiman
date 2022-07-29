use crate::ir;
use crate::value::*;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct AbNodeId(usize);

enum AbNodeKind {
    Primitive(Primitive),
    Funclet(ir::FuncletId),
}
/// A node in the A-PEG
struct AbNode {
    kind: AbNodeKind,
    deps: Box<[AbNodeId]>,
}

fn compute_apeg(program: &ir::Program, entry: ir::FuncletId) -> Vec<AbNode> {
    let loops = ir::utils::identify_loops(program, entry);
    //let mut abnodes = Vec::new();
    todo!();
}
