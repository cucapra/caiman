use crate::assembly;
use crate::explication::expir;
use crate::ir;
use crate::rust_wgpu_backend::ffi;
use itertools::Itertools;
use paste::paste;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

fn unknown(index: &usize) -> String {
    format!("__UNNAMED")
}

fn unknown_quot(quot: &ir::Quotient) -> String {
    format!("__UNNAMED")
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct DebugInfo {
    // Maps from program indices to original strings in the assembly AST
    // The intention here is purely to recover error messages
    // This structure is decoupled from any one IR/AST
    pub type_map: HashMap<usize, String>,
    pub ffi_type_map: HashMap<usize, assembly::ast::FFIType>,
    pub function_class_map: HashMap<usize, String>,
    pub external_function_map: HashMap<usize, String>,
    pub funclet_map: HashMap<usize, FuncletDebugMap>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct FuncletDebugMap {
    // Debug information about a funclet
    pub name: String,
    // we need to use the quotient to differentiate which thing to index into
    pub node_map: HashMap<ir::Quotient, String>,
}

impl DebugInfo {
    pub fn ffi_typ(&self, index: &usize) -> String {
        self.ffi_type_map
            .get(index)
            .as_ref()
            .map(|f| format!("{:?} : (type {})", f, index))
            .unwrap_or(unknown(index))
    }
    pub fn typ(&self, index: &usize) -> String {
        self.type_map.get(index).unwrap_or(&unknown(index)).clone()
    }
    pub fn function_class(&self, index: &usize) -> String {
        self.function_class_map
            .get(index)
            .unwrap_or(&unknown(index))
            .clone()
    }
    pub fn external_function(&self, index: &usize) -> String {
        self.external_function_map
            .get(index)
            .unwrap_or(&unknown(index))
            .clone()
    }
    pub fn funclet(&self, index: &usize) -> String {
        self.funclet_map
            .get(index)
            .map(|f| f.name.clone())
            .unwrap_or(unknown(index))
    }
    pub fn quot(&self, funclet_index: &usize, quotient: &ir::Quotient) -> String {
        match self.funclet_map.get(funclet_index) {
            None => format!("{}.{} : (funclet {}, quotient {:?})", unknown(funclet_index), unknown_quot(quotient), funclet_index, quotient),
            Some(f) => format!(
                "{}.{} : (funclet {}, quotient {:?})",
                &f.name,
                f.node_map
                    .get(quotient)
                    .unwrap_or(&unknown_quot(quotient))
                    .clone(),
                funclet_index,
                quotient
            ),
        }
    }
    pub fn node(&self, funclet_index: &usize, node_index: usize) -> String {
        self.quot(
            funclet_index,
            &ir::Quotient::Node {
                node_id: node_index,
            },
        )
    }
    pub fn tag(&self, funclet_index: &usize, tag: &ir::Tag) -> String {
        format!(
            "Tag: {}, {:?}",
            self.quot(funclet_index, &tag.quot),
            &tag.flow
        )
    }
    pub fn spec(&self, spec: &ir::FuncletSpec) -> String {
        match &spec.funclet_id_opt {
            None => "FuncletSpec { no_id }".to_string(),
            Some(funclet_index) => format!("FuncletSpec {{ Funclet {}, inputs: {:?}, outputs: {:?}, implicit_in: {}, implicit_out: {}}}",
                self.funclet(funclet_index),
                spec.input_tags.iter().map(|t| self.tag(&funclet_index, &t)).collect_vec(),
                spec.output_tags.iter().map(|t| self.tag(&funclet_index, &t)).collect_vec(),
                self.tag(funclet_index, &spec.implicit_in_tag),
                self.tag(funclet_index, &spec.implicit_out_tag))
        }
    }
}

macro_rules! recover_element_ir {
    ($arg:ident [$arg_type:ident] $self:ident $funclet:ident) => {
        Some($arg.iter().map(|v| recover_element_ir!(v $arg_type $self $funclet)).collect()).into()
    };
    ($arg:ident Immediate $self:ident $funclet:ident) => {
        Some(format!("{:?}", $arg.clone())).into()
    };
    ($arg:ident Type $self:ident $funclet:ident) => {
        $self.type_map.get($arg).map(|s| assembly::ast::TypeId::Local(s.to_string())).into()
    };
    ($arg:ident Index $self:ident $funclet:ident) => {
        Some($arg.clone()).into()
    };
    ($arg:ident ExternalFunction $self:ident $funclet:ident) => {
        $self.external_function_map.get(&$arg.0)
        .map(|s| assembly::ast::ExternalFunctionId(s.to_string())).into()
    };
    ($arg:ident ValueFunction $self:ident $funclet:ident) => {
        $self.function_class_map.get(&$arg).map(|s| assembly::ast::FunctionClassId(s.to_string())).into()
    };
    ($arg:ident Operation $self:ident $funclet:ident) => {
        $self.funclet_map.get(&$funclet)
        .and_then(|f| f.node_map.get(&ir::Quotient::Node { node_id: *$arg })
        .map(|s| assembly::ast::NodeId(s.to_string()))).into()
    };
    ($arg:ident RemoteOperation $self:ident $funclet:ident) => {
        $self.funclet_map.get(&$funclet)
        .and_then(|f| Some(
            assembly::ast::RemoteNodeId {
                funclet: assembly::ast::MetaId(f.name.clone()),
                node: Some(f.node_map.get(&$arg)
                    .map(|s| assembly::ast::NodeId(s.to_string()))
                    .into())
        }) ).into()
    };
    ($arg:ident Place $self:ident $funclet:ident) => {
        Some($arg.clone()).into()
    };
    ($arg:ident Funclet $self:ident $funclet:ident) => {
        $self.funclet_map.get(&$arg).map(|f| assembly::ast::FuncletId(f.name.clone())).into()
    };
    ($arg:ident StorageType $self:ident $funclet:ident) => {
        $self.ffi_type_map.get(&$arg.clone().0).map(|f| assembly::ast::TypeId::FFI(f.clone())).into()
    };
    ($arg:ident BufferFlags $self:ident $context:ident) => {
        Some($arg.clone()).into()
    };
}

macro_rules! setup_node_ir {
    ($($_lang:ident $name:ident ($($arg:ident : $arg_type:tt,)*) -> $_output:ident;)*) => {
        paste! {
            /*
             * Expands out the node with full debug information
             */
            impl DebugInfo {
                pub fn node_ir(&self, funclet_id: usize, node : &ir::Node) -> String {
                    match node {
                        $(ir::Node::$name { $($arg,)* } => {
                            format!("{}", assembly::ast::Node::$name {
                                $($arg : recover_element_ir!($arg $arg_type self funclet_id),)*
                            })
                        }),*
                    }
                }
            }
        }
    }
}

with_operations!(setup_node_ir);

macro_rules! recover_element_expir {
    ($arg:ident [$arg_type:ident] $self:ident $funclet:ident) => {
        $arg.as_ref().opt().map(|o| o.iter().map(|v| recover_element_expir!(v $arg_type $self $funclet)).collect()).into()
    };
    ($arg:ident Immediate $self:ident $funclet:ident) => {
        $arg.as_ref().opt().map(|o| format!("{:?}", o)).into()
    };
    ($arg:ident Type $self:ident $funclet:ident) => {
        $arg.as_ref().opt().and_then(|o| $self.type_map.get(o).map(|s| assembly::ast::TypeId::Local(s.to_string()))).into()
    };
    ($arg:ident Index $self:ident $funclet:ident) => {
        $arg.clone()
    };
    ($arg:ident ExternalFunction $self:ident $funclet:ident) => {
        $arg.as_ref().opt().and_then(|o| $self.external_function_map.get(&o.0)
        .map(|s| assembly::ast::ExternalFunctionId(s.to_string()))).into()
    };
    ($arg:ident ValueFunction $self:ident $funclet:ident) => {
        $arg.as_ref().opt().and_then(|o| $self.function_class_map.get(o)
        .map(|s| assembly::ast::FunctionClassId(s.to_string()))).into()
    };
    ($arg:ident Operation $self:ident $funclet:ident) => {
        $arg.as_ref().opt().and_then(|o|
            $self.funclet_map.get(&$funclet)
            .and_then(|f| f.node_map.get(&ir::Quotient::Node { node_id: *o })
            .map(|s| assembly::ast::NodeId(s.to_string())))).into()
    };
    ($arg:ident RemoteOperation $self:ident $funclet:ident) => {
        $arg.as_ref().opt().and_then(|o|
            $self.funclet_map.get(&$funclet)
            .and_then(|f| Some(
                assembly::ast::RemoteNodeId {
                    funclet: assembly::ast::MetaId(f.name.clone()),
                    node: Some(f.node_map.get(o)
                        .map(|s| assembly::ast::NodeId(s.to_string()))
                        .into())
        }) )).into()
    };
    ($arg:ident Place $self:ident $funclet:ident) => {
        $arg.clone()
    };
    ($arg:ident Funclet $self:ident $funclet:ident) => {
        $arg.as_ref().opt().and_then(|o| $self.funclet_map.get(o)
        .map(|f| assembly::ast::FuncletId(f.name.clone()))).into()
    };
    ($arg:ident StorageType $self:ident $funclet:ident) => {
        $arg.as_ref().opt().and_then(|o| $self.ffi_type_map.get(&o.0)
        .map(|f| assembly::ast::TypeId::FFI(f.clone()))).into()
    };
    ($arg:ident BufferFlags $self:ident $context:ident) => {
        $arg.clone()
    };
}

macro_rules! setup_node_expir {
    ($($_lang:ident $name:ident ($($arg:ident : $arg_type:tt,)*) -> $_output:ident;)*) => {
        paste! {
            /*
             * Expands out the node with full debug information
             */
            impl DebugInfo {
                pub fn node_expir(&self, funclet_id: usize, node : &expir::Node) -> String {
                    match node {
                        $(expir::Node::$name { $($arg,)* } => {
                            format!("{}", assembly::ast::Node::$name {
                                $($arg : recover_element_expir!($arg $arg_type self funclet_id),)*
                            })
                        }),*
                    }
                }
            }
        }
    }
}

with_operations!(setup_node_expir);
