pub mod context;
use std::collections::HashMap;
mod specs;

pub use specs::sched_lit_to_str;

use crate::parse::ast::{Binop, DataType};
use caiman::assembly::ast as asm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The type of a spec.
pub enum SpecType {
    Value,
    Timeline,
    Spatial,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SpecNode {
    Input(String),
    Output(String),
    Lit(String),
    Op(String, Vec<String>),
    Extract(String, usize),
}

#[derive(Debug, Clone, Default)]
pub struct NodeMap {
    pub nodes: HashMap<String, SpecNode>,
    pub node_names: HashMap<SpecNode, String>,
}

impl NodeMap {
    #[must_use]
    pub fn get_name(&self, node: &SpecNode) -> Option<&String> {
        self.node_names.get(node)
    }

    #[must_use]
    pub fn get_node(&self, name: &str) -> Option<&SpecNode> {
        self.nodes.get(name)
    }

    pub fn insert(&mut self, name: String, node: SpecNode) {
        self.nodes.insert(name.clone(), node.clone());
        self.node_names.insert(node, name);
    }

    /// Inserts a node if its name and value is unique. If the node is already
    /// present, it is removed.
    pub fn insert_or_remove_if_dup(&mut self, name: String, node: SpecNode) {
        if self.contains(&name) || self.contains_node(&node) {
            self.nodes.remove(&name);
            self.node_names.remove(&node);
        } else {
            self.insert(name, node);
        }
    }

    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.nodes.contains_key(name)
    }

    #[must_use]
    pub fn contains_node(&self, node: &SpecNode) -> bool {
        self.node_names.contains_key(node)
    }
}

#[derive(Debug, Clone)]
pub struct SpecInfo {
    /// Type of the spec
    pub typ: SpecType,
    /// Type signature
    pub sig: (Vec<DataType>, Vec<DataType>),
    /// Map from variable name to node and node to variable name.
    pub nodes: NodeMap,
    /// Map from variable name to type.
    pub types: HashMap<String, DataType>,
}

impl SpecInfo {
    #[must_use]
    pub fn new(typ: SpecType, sig: (Vec<DataType>, Vec<DataType>)) -> Self {
        Self {
            typ,
            sig,
            nodes: NodeMap::default(),
            types: HashMap::new(),
        }
    }
}

/// A global context for a caiman program. This contains information about constants,
/// type aliases, and function signatures.
pub struct Context {
    pub type_decls: Vec<asm::Declaration>,
    /// Signatures of function classes. Map from spec name to (input types, output types).
    pub signatures: HashMap<String, (Vec<DataType>, Vec<DataType>)>,
    /// Map from sched function name to a map from variable name to type.
    /// The variables in this map are the ones present at the source level ONLY.
    pub sched_types: HashMap<String, HashMap<String, DataType>>,
    /// Map from spec name to spec info.
    pub specs: HashMap<String, SpecInfo>,
}

/// Returns the output type of a binary operation.
fn op_output_type(op: Binop, op_l: &DataType, op_r: &DataType) -> DataType {
    match op {
        // TODO: argument promotion
        Binop::Add | Binop::Sub | Binop::Mul | Binop::Div => {
            assert!(matches!(op_l, DataType::Int(_) | DataType::Float(_)));
            assert!(matches!(op_r, DataType::Int(_) | DataType::Float(_)));
            op_l.clone()
        }
        Binop::Lt
        | Binop::Gt
        | Binop::Geq
        | Binop::Leq
        | Binop::Eq
        | Binop::Neq
        | Binop::Land
        | Binop::Lor => DataType::Bool,
        Binop::And
        | Binop::Or
        | Binop::Xor
        | Binop::AShr
        | Binop::Shr
        | Binop::Shl
        | Binop::Mod => {
            assert!(matches!(op_l, DataType::Int(_)));
            assert!(matches!(op_r, DataType::Int(_)));
            op_l.clone()
        }
        Binop::Dot | Binop::Range | Binop::Index | Binop::Cons => todo!(),
    }
}

/// A typed binary operation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TypedBinop {
    op: Binop,
    /// The type of the left operand.
    op_l: DataType,
    /// The type of the right operand.
    op_r: DataType,
    /// The type of the result.
    ret: DataType,
}
