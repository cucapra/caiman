use crate::assembly::ast;
use crate::assembly::ast::FFIType;
use crate::assembly::ast::Hole;
use crate::assembly::ast::{
    ExternalCpuFunctionId, ExternalGpuFunctionId, FuncletId, NodeId, OperationId, StorageTypeId,
    TypeId, ValueFunctionId,
};
use crate::assembly::context::Context;
use crate::assembly::context::FuncletLocation;
use crate::assembly::explication_explicator;
use crate::assembly::explication_util::*;
use crate::assembly::parser;
use crate::ir::ffi;
use crate::{assembly, frontend, ir};
use std::any::Any;
use std::collections::HashMap;

// it's probably best to do the lowering pass like this,
//   and simply guarantee there won't be any holes left over
pub fn explicate(program: ast::Program, context: &mut Context) -> ast::Program {
    program
}
