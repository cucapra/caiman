use crate::explication;
use crate::ir;
use crate::debug_info::DebugInfo;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::default::Default;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ExplicationDefinition {
    pub version: (u32, u32, u32),
    pub debug_info: DebugInfo,
    pub program: explication::expir::Program,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Definition {
    pub version: (u32, u32, u32),
    pub debug_info: DebugInfo,
    pub program: ir::Program,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub enum CompileMode {
    #[default]
    Assembly,
    RON,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CompileData {
    pub path: String,
    pub input_string: String,
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

// #[cfg(feature = "assembly")]
fn read_assembly(compile_data: CompileData) -> Result<ExplicationDefinition, CompileError> {
    let program = crate::assembly::parser::parse(&compile_data.path, &compile_data.input_string);
    // dbg!(&program);
    match program {
        Err(why) => Err(CompileError {
            message: format!("Parse error: {}", why),
        }),
        // Ok(_) => todo!()
        Ok(v) => Ok(crate::assembly::lowering_pass::lower(v)),
    }
}

// #[cfg(not(feature = "assembly"))]
// fn read_assembly(input_string: &str) -> Result<Definition, CompileError> {
//     Result::Err(CompileError {
//         message: String::from("Assembly is unsupported in this build"),
//     })
// }

fn read_definition(
    compile_data: CompileData,
    compile_mode: CompileMode,
) -> Result<Definition, CompileError> {
    match compile_mode {
        CompileMode::Assembly => read_assembly(compile_data).map(explication::explicate),
        CompileMode::RON => match ron::from_str(&compile_data.input_string) {
            Err(why) => Err(CompileError {
                message: format!("Parse error at {}: {}", why.position, why),
            }),
            Ok(v) => Ok(v),
        },
    }
}

pub fn compile_caiman(
    compile_data: CompileData,
    options: CompileOptions,
) -> Result<String, CompileError> {
    let mut definition = read_definition(compile_data, options.compile_mode)?;
    // dbg!(&definition);
    assert_eq!(definition.version, (0, 0, 2));
    //ir::validation::validate_program(&definition.program);
    match crate::type_system::check_program(&definition.program) {
        Ok(_) => (),
        Err(error) => panic!("Type checking failed:\n{}", error),
    }
    let mut codegen = crate::rust_wgpu_backend::codegen::CodeGen::new(&definition.program);
    codegen.set_print_codgen_debug_info(options.print_codegen_debug_info);
    let output_string = codegen.generate();
    Ok(output_string)
}

pub fn explicate_caiman(
    compile_data: CompileData,
    options: CompileOptions,
) -> Result<String, CompileError> {
    let pretty = ron::ser::PrettyConfig::new().enumerate_arrays(true);
    let mut definition = read_definition(compile_data, options.compile_mode)?;
    assert_eq!(definition.version, (0, 0, 2));
    let output_string_result = ron::ser::to_string_pretty(&definition, pretty);
    Ok(output_string_result.unwrap())
}
