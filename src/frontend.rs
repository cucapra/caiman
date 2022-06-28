use crate::ir;
use std::collections::HashMap;
use std::default::Default;
use serde_derive::{Serialize, Deserialize};

use caiman_frontend;
use crate::frontend_to_ir;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Definition
{
	pub version : (u32, u32, u32),
	pub program : ir::Program
}


#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CompileOptions
{
	pub print_codegen_debug_info : bool
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CompileError
{
	message : String
}

impl std::fmt::Display for CompileError
{
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
	{
		write!(f, "{}", self.message)
	}
}

fn compile_general(
  mut definition: Definition,
  options_opt: Option<CompileOptions>
) -> String
{
  assert_eq!(definition.version, (0, 0, 1));
  crate::rust_wgpu_backend::explicate_scheduling::explicate_scheduling(
      &mut definition.program
  );
  let mut codegen = crate::rust_wgpu_backend::codegen::CodeGen::new(
      & definition.program
  );
  if let Some(options) = options_opt
  {
    codegen.set_print_codgen_debug_info(options.print_codegen_debug_info);
  }
  codegen.generate()
}

pub fn compile_ron_definition(
  input_string : &str, 
  options_opt : Option<CompileOptions>
) -> Result<String, CompileError>
{
	let result : Result<Definition, ron::de::Error> = ron::from_str(
    & input_string
  );
	match result
	{
		Err(why) => Err(CompileError{ message: format!("Parse error: {}", why)}),
		Ok(mut definition) => Ok(compile_general(definition, options_opt)),
	}
}

pub fn compile_frontend_language(
    input_string : &str, 
    options_opt : Option<CompileOptions>,
) -> Result<String, CompileError>
{
    match caiman_frontend::parse_string(String::from(input_string))
    {
        Err(why) => {
            Err(CompileError{ message: format!("Parsing Error: {}", why) })
        },
        Ok(ast) => {
            match frontend_to_ir::from_ast(ast)
            {
                Err(why) => {
                    let why_s = frontend_to_ir::semantic_error_to_string(why);
                    Err(CompileError { 
                        message: format!("Semantic Error: {}", why_s) 
                    })
                },
                Ok(program) => {
                    let mut definition = Definition { 
                        version: (0, 0, 1), 
                        program 
                    };
                    Ok(compile_general(definition, options_opt))
                },
            }
        },
    }
}

pub fn explicate_ron_definition(input_string : &str, options : Option<CompileOptions>) -> Result<String, CompileError>
{
	let pretty = ron::ser::PrettyConfig::new().enumerate_arrays(true);

	let mut result : Result<Definition, ron::de::Error> = ron::from_str(
      & input_string
  );
	match result
	{
		Err(why) => Err(CompileError{ message: format!("Parse error: {}", why)}),
		Ok(mut definition) =>
		{
			assert_eq!(definition.version, (0, 0, 1));
			crate::rust_wgpu_backend::explicate_scheduling::explicate_scheduling(
          &mut definition.program
      );
			let output_string_result = ron::ser::to_string_pretty(
          & definition, 
          pretty
      );
			Ok(output_string_result.unwrap())
		}
	}
}
