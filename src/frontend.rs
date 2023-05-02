use crate::ir;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::default::Default;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Definition {
    pub version: (u32, u32, u32),
    pub program: ir::Program,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub enum CompileMode {
    #[default]
    Assembly,
    RON,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CompileOptions {
    pub print_codegen_debug_info: bool,
    pub compile_mode: CompileMode,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CompileError {
    pub message: String,
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

fn read_definition(
    input_string: &str,
    compile_mode: CompileMode,
) -> Result<Definition, CompileError> {
    match compile_mode {
        CompileMode::Assembly => crate::assembly::parser::parse(input_string),
        CompileMode::RON => match ron::from_str(&input_string) {
            Err(why) => Err(CompileError {
                message: format!("Parse error: {}", why),
            }),
            Ok(v) => Ok(v),
        },
    }
}

pub fn compile_caiman(input_string: &str, options: CompileOptions) -> Result<String, CompileError> {
    let mut definition = read_definition(input_string, options.compile_mode)?;
    assert_eq!(definition.version, (0, 0, 1));
    ir::validation::validate_program(&definition.program);
    let mut codegen = crate::rust_wgpu_backend::codegen::CodeGen::new(&definition.program);
    codegen.set_print_codgen_debug_info(options.print_codegen_debug_info);
    let output_string = codegen.generate();
    Ok(output_string)
}

pub fn explicate_caiman(
    input_string: &str,
    options: CompileOptions,
) -> Result<String, CompileError> {
    let pretty = ron::ser::PrettyConfig::new().enumerate_arrays(true);
    let mut definition = read_definition(input_string, options.compile_mode)?;
    assert_eq!(definition.version, (0, 0, 1));
    let output_string_result = ron::ser::to_string_pretty(&definition, pretty);
    Ok(output_string_result.unwrap())
}
