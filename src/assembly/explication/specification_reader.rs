mod setup;
mod getters;

use crate::assembly::ast;
use crate::assembly::ast::Hole;
use crate::assembly::ast::{
    ExternalFunctionId, FFIType, FuncletId, FunctionClassId, NodeId, RemoteNodeId, StorageTypeId,
    TypeId,
};
use crate::assembly::table::Table;
use crate::ir;
use std::collections::{HashMap, HashSet};
use serde_derive::{Deserialize, Serialize};

// Immutable object that holds all the static (non-explicated) information about a given program
#[derive(Serialize, Deserialize, Debug, Clone)]
struct SpecInfo {
    funclets: HashMap<FuncletId, FuncletTypeInfo>
}

// Organization
#[derive(Serialize, Deserialize, Debug, Clone)]
struct FuncletTypeInfo {
    kind: ir::FuncletKind,
    nodes: HashMap<NodeId, NodeTypeInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct NodeTypeInfo {
    dependencies: Vec<NodeId>,
    typ: ast::TypeId
}