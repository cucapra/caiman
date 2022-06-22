use crate::ir;
use petgraph::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {}

pub fn apply(program: &mut ir::Program) -> Result<(), Error> {
    todo!()
}
