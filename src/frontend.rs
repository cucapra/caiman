use crate::ir;
use std::collections::HashMap;
use std::default::Default;
use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
pub struct Definition
{
	pub version : (u32, u32, u32),
	pub program : ir::Program
}


#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CompileOptions
{
	pub print_codegen_debug_info : bool,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CompileError
{
	pub message : String
}

impl std::fmt::Display for CompileError
{
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
	{
		write!(f, "{}", self.message)
	}
}

pub fn explicate_compare(input_string1 : &str, input_string2 : &str) -> Result<bool, CompileError> {
	let result1 = crate::assembly::parser::parse(input_string1);
	let result2 = crate::assembly::parser::parse(input_string2);
	let mut definition1 = match result1 {
		Err(e) => { return Err(e) }
		Ok(definition) => definition
	};
	let mut definition2 = match result2 {
		Err(e) => { return Err(e) }
		Ok(definition) => definition
	};
	crate::rust_wgpu_backend::explicate_scheduling::explicate_scheduling(&mut definition1.program);
	crate::rust_wgpu_backend::explicate_scheduling::explicate_scheduling(&mut definition2.program);
	Ok(definition1 == definition2)
}

pub fn compile_ron_definition(input_string : &str,
                              options_opt : Option<CompileOptions>,
                              assembly : bool) -> Result<String, CompileError>
{
	let result : Result<Definition, CompileError> =
		if assembly {
			crate::assembly::parser::parse(input_string)
		}
		else {
			match ron::from_str(& input_string) {
				Err(why) => Err(CompileError{ message: format!("Parse error: {}", why) }),
				Ok(v) => Ok(v)
			}
		};
	match result
	{
		Err(why) => Err(why),
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