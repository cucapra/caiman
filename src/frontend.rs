use crate::{
    ir, optimizations::*, rust_wgpu_backend::codegen, rust_wgpu_backend::explicate_scheduling,
};
use serde_derive::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, Default)]
struct Definition {
    version: (u32, u32, u32),
    program: ir::Program,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    /// Only optimize the IR.
    Optimize,
    /// Optimize the IR, then explicate scheduling.
    Explicate,
    /// Optimize the IR, explicate scheduling, and run codegen.
    Compile,
}
impl Default for Action {
    fn default() -> Self {
        Self::Compile
    }
}

#[derive(Default)]
pub struct Options {
    pub action: Action,
    pub optimizer: Optimizer,
    pub print_codegen_debug_info: bool,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to parse ron definition: {source}")]
    Parse {
        #[from]
        source: ron::de::Error,
    },
    #[error("failed to apply optimizations: {source}")]
    Transform {
        #[from]
        source: OptError,
    },
}

pub fn compile(options: &Options, input_string: &str) -> Result<String, Error> {
    let pretty = ron::ser::PrettyConfig::new().enumerate_arrays(true);

    // Load IR
    let mut definition: Definition = ron::from_str(&input_string)?;
    assert_eq!(definition.version, (0, 0, 1));

    // Apply transforms
    options.optimizer.apply(&mut definition.program)?;
    if options.action == Action::Optimize {
        return Ok(ron::ser::to_string_pretty(&definition, pretty).unwrap());
    }

    // Explicate
    explicate_scheduling::explicate_scheduling(&mut definition.program);
    if options.action == Action::Explicate {
        return Ok(ron::ser::to_string_pretty(&definition, pretty).unwrap());
    }

    // Run codegen
    let mut codegen = codegen::CodeGen::new(&definition.program);
    codegen.set_print_codgen_debug_info(options.print_codegen_debug_info);
    Ok(codegen.generate())
}
