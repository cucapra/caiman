#![warn(warnings)]
use crate::ir;
use petgraph::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {}

/// Applies a set of optimizing transformations to the IR. These transformations only operate
/// on value operations; after the value/scheduling language split, this will become specific
/// to the value language.
///
/// I plan to eventually apply these value transformations:
/// 1. Constant folding
/// 2. Dead operation elimination
/// 3. Operation deduplication
/// 4. Value function inlining
///
///    Loop 1-4 as long as step 4 inlines at least one function.
///    Only reapply 1-3 to funclets affected in the prior step ("dirty")
///
/// 5. Constant folding
/// 6. Dead funclet elimination
/// 7. Funclet
///
/// We assume that CallExternal* operations have no side effects, so they can be constant folded --
/// in theory. It may be challenging to implement this in practice.
pub fn apply(program: &mut ir::Program) -> Result<(), Error> {
    todo!()
}
