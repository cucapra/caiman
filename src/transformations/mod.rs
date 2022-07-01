#![warn(warnings)]
use crate::ir;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {}

type Result<T> = std::result::Result<T, Error>;

mod funclet;

/// Applies a set of optimizing transformations to the IR. These transformations only operate
/// on value operations; after the value/scheduling language split, this will become specific
/// to the value language.
pub fn apply(program: &mut ir::Program) -> Result<()> {
    for (_, f) in program.funclets.iter_mut() {
        funclet::prune_unused_nodes(f)?;
    }
    Ok(())
}
