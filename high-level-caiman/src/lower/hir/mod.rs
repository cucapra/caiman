pub mod cfg;
#[allow(clippy::module_inception)]
mod hir;

pub use hir::*;

use crate::parse::ast::{Arg, FullType, SchedulingFunc};
use caiman::assembly::ast as asm;
use caiman::ir;

use self::{
    analysis::{analyze, LiveVars},
    cfg::{BasicBlock, Cfg},
};

use super::{global_context::SpecType, lower_schedule::hlc_arg_to_asm_arg};
mod analysis;
#[cfg(test)]
mod test;

pub use analysis::RET_VAR;

/// Scheduling funclet specs
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Specs {
    pub value: asm::FuncletId,
    pub timeline: asm::FuncletId,
    pub spatial: asm::FuncletId,
}

impl Specs {
    /// Gets the type of spec the funclet with `spec_name` is or `None`
    /// if it is not a spec funclet.
    pub fn get_spec_type(&self, spec_name: &str) -> Option<SpecType> {
        if spec_name == self.value.0 {
            Some(SpecType::Value)
        } else if spec_name == self.spatial.0 {
            Some(SpecType::Spatial)
        } else if spec_name == self.timeline.0 {
            Some(SpecType::Timeline)
        } else {
            None
        }
    }
}

/// Gets a vector of tags for each of the specs, all with a none quotient
/// and a usable flow
fn none_usable(specs: &Specs) -> Vec<asm::Tag> {
    vec![
        asm::Tag {
            quot: asm::Quotient::None(Some(asm::RemoteNodeId {
                funclet: Some(specs.value.clone()),
                node: None,
            })),
            flow: ir::Flow::Usable,
        },
        asm::Tag {
            quot: asm::Quotient::None(Some(asm::RemoteNodeId {
                funclet: Some(specs.timeline.clone()),
                node: None,
            })),
            flow: ir::Flow::Usable,
        },
        asm::Tag {
            quot: asm::Quotient::None(Some(asm::RemoteNodeId {
                funclet: Some(specs.spatial.clone()),
                node: None,
            })),
            flow: ir::Flow::Usable,
        },
    ]
}

/// Information about a high level caiman function
struct FuncInfo {
    name: String,
    input: Vec<Arg<FullType>>,
    output: Arg<FullType>,
}

/// The funclets of a scheduling function.
/// Combines the scheduling function's CFG with its analysis information
pub struct Funclets {
    cfg: Cfg,
    live_vars: analysis::InOutFacts<LiveVars>,
    finfo: FuncInfo,
    specs: Specs,
}

/// A specific funclet in a scheduling function.
/// Combines the funclet's basic block with is analysis information.
pub struct Funclet<'a> {
    parent: &'a Funclets,
    block: &'a BasicBlock,
}

impl<'a> Funclet<'a> {
    /// Gets the next blocks in the cfg as `FuncletIds`
    pub fn next_blocks(&self) -> Vec<asm::Hole<asm::FuncletId>> {
        match &self.block.terminator {
            Terminator::FinalReturn => vec![],
            Terminator::Select(_) => {
                let mut e = self
                    .parent
                    .cfg
                    .successors(self.block.id)
                    .into_iter()
                    .map(|id| asm::FuncletId(self.parent.funclet_name(id)));
                let mut res = vec![];
                if let Some(true_block) = e.next() {
                    res.push(Some(true_block));
                }
                if let Some(false_block) = e.next() {
                    res.push(Some(false_block));
                }
                assert_eq!(res.len(), 2);
                res
            }
            Terminator::Call(..) => todo!(),
            Terminator::None | Terminator::Return(..) => {
                let e = self
                    .parent
                    .cfg
                    .successors(self.block.id)
                    .into_iter()
                    .map(|id| Some(asm::FuncletId(self.parent.funclet_name(id))));
                let res: Vec<_> = e.collect();
                assert!(res.len() <= 1);
                res
            }
        }
    }

    /// Gets the input arguments for each block based on the block's live in variables
    pub fn inputs(&self) -> Vec<asm::FuncletArgument> {
        if self.id() == cfg::START_BLOCK_ID {
            self.parent
                .finfo
                .input
                .iter()
                .map(hlc_arg_to_asm_arg)
                .collect()
        } else {
            self.parent
                .live_vars
                .get_in_fact(self.id())
                .live_set()
                .iter()
                .map(|var| {
                    asm::FuncletArgument {
                        name: Some(asm::NodeId(var.clone())),
                        // TODO: support more types
                        typ: asm::TypeId::Local(String::from("i64")),
                        // TODO: don't hardcode this
                        tags: none_usable(&self.parent.specs),
                    }
                })
                .collect()
        }
    }

    /// Gets the return arguments of a funclet based on the block's live out variables
    pub fn outputs(&self) -> Vec<asm::FuncletArgument> {
        if self.id() == cfg::START_BLOCK_ID || self.id() == cfg::FINAL_BLOCK_ID {
            vec![hlc_arg_to_asm_arg(&self.parent.finfo.output)]
        } else {
            // I don't think this is right
            self.parent
                .live_vars
                .get_out_fact(self.id())
                .live_set()
                .iter()
                .map(|var| asm::FuncletArgument {
                    name: Some(asm::NodeId(var.clone())),
                    // TODO: support more types
                    typ: asm::TypeId::Local(String::from("i64")),
                    // TODO: don't hardcode this
                    tags: none_usable(&self.parent.specs),
                })
                .collect()
        }
    }

    /// Gets the nodes that exit this funclet
    pub fn output_args(&self) -> Vec<asm::Hole<asm::NodeId>> {
        self.parent
            .live_vars
            .get_out_fact(self.block.id)
            .live_set()
            .iter()
            .cloned()
            .map(asm::NodeId)
            .map(Some)
            .collect()
    }

    #[inline]
    pub fn stmts(&self) -> &[hir::Hir] {
        &self.block.stmts
    }

    #[inline]
    pub const fn terminator(&self) -> &hir::Terminator {
        &self.block.terminator
    }

    /// Numeric id of the funclet, which is how its represented at the HIR level
    #[inline]
    pub const fn id(&self) -> usize {
        self.block.id
    }

    /// Gets the name of the funclet
    #[inline]
    pub fn name(&self) -> String {
        self.parent.funclet_name(self.id())
    }

    /// Gets the specs of this funclet
    #[inline]
    pub const fn specs(&self) -> &Specs {
        &self.parent.specs
    }

    /// Gets the funclet id of the join point
    #[inline]
    pub fn join_funclet(&self) -> Option<asm::FuncletId> {
        self.block
            .join_block
            .map(|id| asm::FuncletId(self.parent.funclet_name(id)))
    }
}

impl Funclets {
    pub fn new(f: SchedulingFunc, specs: Specs) -> Self {
        let cfg = Cfg::new(f.statements);
        let live_vars = analyze(&cfg, &analysis::LiveVars::top());
        let finfo = FuncInfo {
            name: f.name,
            input: f.input,
            output: (
                String::new(),
                f.output.expect("Functions must return values for now"),
            ),
        };
        Self {
            cfg,
            live_vars,
            finfo,
            specs,
        }
    }

    pub fn funclets(&self) -> Vec<Funclet<'_>> {
        self.cfg
            .blocks
            .values()
            .map(|blk| Funclet {
                parent: self,
                block: blk,
            })
            .collect()
    }

    /// Gets the name of the scheduling funclet for a given block
    fn funclet_name(&self, block_id: usize) -> String {
        if block_id == cfg::START_BLOCK_ID {
            self.finfo.name.clone()
        } else {
            format!("_{}{block_id}", self.finfo.name)
        }
    }
}
