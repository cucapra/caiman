use crate::{
    error::LocalError,
    lower::sched_hir::{cfg::Cfg, HirBody},
    parse::ast::FullType,
    typing::{Context, NodeEnv, SpecInfo},
};

use super::continuations::compute_pretinuations;

pub fn deduce_val_quots(
    inputs: &mut [(String, Option<FullType>)],
    outputs: &mut [FullType],
    cfg: &mut Cfg,
    spec_info: &SpecInfo,
    ctx: &Context,
) {
    let mut env = spec_info.nodes.clone();
}

fn unify_nodes(
    inputs: &[(String, Option<FullType>)],
    outputs: &[FullType],
    cfg: &Cfg,
    mut env: NodeEnv,
) -> Result<NodeEnv, LocalError> {
    let pretinuations = compute_pretinuations(cfg);
    for block in cfg.blocks.values() {
        for stmt in &block.stmts {
            // match stmt {
            //     HirBody::ConstDecl { info, lhs, lhs_tag, rhs } => {

            //     }
            // }
        }
    }
    Ok(env)
}
