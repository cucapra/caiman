use crate::assembly::explication_context::Context;
use crate::assembly::parser;
use crate::assembly_ast::FFIType;
use crate::assembly_ast::Hole;
use crate::assembly_context::FuncletLocation;
use crate::ir::ffi;
use crate::{assembly_ast, assembly_context, frontend, ir};
use std::any::Any;
use std::collections::HashMap;

pub fn explicate_encode_do(
    place_hole: Hole<ir::Place>,
    operation_hole: Hole<assembly_ast::RemoteNodeId>,
    inputs_hole: Hole<Box<[Hole<assembly_ast::OperationId>]>>,
    outputs_hole: Hole<Box<[Hole<assembly_ast::OperationId>]>>,
    context : Context
) -> Option<ir::Node> {
    let place = place_hole.unwrap_or(todo!());
}
