use crate::ir;
use std::collections::HashMap;
use std::default::Default;
use thiserror::Error;
use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Default)]
struct Definition
{
	version : (u32, u32, u32),
	program : ir::Program
}


#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CompileOptions
{
	pub print_codegen_debug_info : bool
}

#[derive(Error, Debug)]
pub enum CompileError
{
	#[error("failed to parse ron definition: {source}")]
	Parse { 
		#[from]
		source: ron::de::Error 
	},
	#[error("failed to apply transformation: {source}")]
	Transformation {
		#[from]
		source: crate::transformations::Error 
	}
}


pub fn compile_ron_definition(input_string : &str, options_opt : Option<CompileOptions>) -> Result<String, CompileError>
{
	let mut definition: Definition = ron::from_str(&input_string)?;
	assert_eq!(definition.version, (0, 0, 1));
	crate::transformations::optimize(&mut definition.program)?;
	crate::rust_wgpu_backend::explicate_scheduling::explicate_scheduling(&mut definition.program);
	let mut codegen = crate::rust_wgpu_backend::codegen::CodeGen::new(&definition.program);
	if let Some(options) = options_opt
	{
		codegen.set_print_codgen_debug_info(options.print_codegen_debug_info);
	}
	let output_string = codegen.generate();
	Ok(output_string)
}

pub fn explicate_ron_definition(input_string : &str, options : Option<CompileOptions>) -> Result<String, CompileError>
{
	let pretty = ron::ser::PrettyConfig::new().enumerate_arrays(true);
	let mut definition: Definition = ron::from_str(&input_string)?;
	assert_eq!(definition.version, (0, 0, 1));
	crate::transformations::optimize(&mut definition.program)?;
	crate::rust_wgpu_backend::explicate_scheduling::explicate_scheduling(&mut definition.program);
	let output_string_result = ron::ser::to_string_pretty(&definition, pretty);
	Ok(output_string_result.unwrap())
}