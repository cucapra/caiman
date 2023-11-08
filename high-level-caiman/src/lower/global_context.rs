use std::collections::HashMap;

use crate::parse::ast::{ClassMembers, TopLevel};

/// The type of a spec.
pub enum SpecType {
    Value,
    Timeline,
    Spatial,
}

/// A global context for a caiman program. This contains information about constants,
/// type aliases, and function signatures.
pub struct Context {
    pub specs: HashMap<String, SpecType>,
}

/// Creates a global context from a list of top-level declarations.
pub fn gen_context(tl: &[TopLevel]) -> Context {
    let mut ctx = Context {
        specs: HashMap::new(),
    };
    for decl in tl {
        match decl {
            TopLevel::SpatialFunclet { name, .. } => {
                ctx.specs.insert(name.to_string(), SpecType::Value);
            }
            TopLevel::TimelineFunclet { name, .. } => {
                ctx.specs.insert(name.to_string(), SpecType::Timeline);
            }
            TopLevel::FunctionClass { members, .. } => {
                for m in members {
                    match m {
                        ClassMembers::ValueFunclet { name, .. } => {
                            ctx.specs.insert(name.to_string(), SpecType::Value);
                        }
                        ClassMembers::Extern { .. } => (),
                    }
                }
            }
            _ => (),
        }
    }
    ctx
}
