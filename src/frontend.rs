use crate::ir;
use std::collections::HashMap;
use std::default::Default;
use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Definition
{
	version : (u32, u32, u32),
	program : ir::Program
}


#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CompileOptions
{
	pub print_codegen_debug_info : bool,
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

pub fn compile_ron_definition(input_string : &str,
                              options_opt : Option<CompileOptions>,
                              assembly : bool) -> Result<String, CompileError>
{
	let result : Result<Definition, ron::de::Error> = 
		if assembly {
			crate::assembly::parser::parse(input_string)
		}
		else {
			ron::from_str(& input_string)
		};
	match result
	{
		Err(why) => Err(CompileError{ message: format!("Parse error: {}", why)}),
		Ok(mut definition) =>
		{
			assert_eq!(definition.version, (0, 0, 1));
			crate::rust_wgpu_backend::explicate_scheduling::
				explicate_scheduling(&mut definition.program);
			ir::validation::validate_program(& definition.program);
			let mut codegen = crate::rust_wgpu_backend::codegen::CodeGen::new(& definition.program);
			if let Some(options) = options_opt
			{
				codegen.set_print_codgen_debug_info(options.print_codegen_debug_info);
			}
			let output_string = codegen.generate();
			Ok(output_string)
		}
	}
}

pub fn explicate_ron_definition(input_string : &str, options : Option<CompileOptions>) -> Result<String, CompileError>
{
	let pretty = ron::ser::PrettyConfig::new().enumerate_arrays(true);

	let mut result : Result<Definition, ron::de::Error> = ron::from_str(& input_string);
	match result
	{
		Err(why) => Err(CompileError{ message: format!("Parse error: {}", why)}),
		Ok(mut definition) =>
		{
			assert_eq!(definition.version, (0, 0, 1));
			crate::rust_wgpu_backend::explicate_scheduling::
				explicate_scheduling(&mut definition.program);
			let output_string_result = ron::ser::to_string_pretty(& definition, pretty);
			Ok(output_string_result.unwrap())
		}
	}
}