// User friendly definition of pipeline blocks
use std::collections::HashMap;
use std::default::Default;
use serde::{Serialize, Deserialize};
use serde_derive::{Serialize, Deserialize};
use crate::ir as ir;

#[derive(Serialize, Deserialize, Debug)]
pub struct StructField
{
	name : String,
	type_name : String,
	byte_offset : usize,
	byte_size : usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Type
{
	F32,
	F64,
	U8,
	U16,
	U32,
	U64,
	I8,
	I16,
	I32,
	I64,
	//Float { exponent_width : usize, mantissa_width : usize },
	//Integer { bit_width : usize, is_signed : bool },
	ConstRef { element_type : String },
	MutRef { element_type : String },
	ConstSlice { element_type : String },
	MutSlice { element_type : String },
	Array { element_type : String, length : usize },
	Struct { fields : Box<[StructField]>, byte_alignment : Option<usize>, byte_size : Option<usize> }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Variable
{
	type_name : String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Source
{
	Variable(String),
	UInt(usize),
	SInt(isize),
	F64(f64)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GpuCallBinding
{
	name : String,
	source : Source,
	is_input : bool,
	is_output : bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Instruction
{
	CpuFunctionCall,
	GpuRasterCall,
	GpuComputeCall { compute_kernel : String, dimensions : [Source; 3], bindings : Box<[GpuCallBinding]> },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pipeline
{
	variables : HashMap<String, Variable>,
	arguments : Box<[String]>,
	instructions : Box<[Instruction]>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Definition
{
	types : HashMap<String, Type>,
	pipelines : HashMap<String, Pipeline>,
}

#[derive(Debug, Default)]
struct Compiler
{
	type_ids : HashMap<String, usize>,
	program : ir::Program
}

impl Compiler
{

	fn compile_types(&mut self, definition : &Definition)
	{
		for (type_name, _) in definition.types.iter()
		{
			self.type_ids.insert(type_name.to_string(), self.type_ids.len());
		}

		for (type_name, definition_type) in definition.types.iter()
		{
			let ir_type : ir::Type = match definition_type
			{
				Type::F32 => ir::Type::F32,
				Type::F64 => ir::Type::F64,
				Type::U8 => ir::Type::U8,
				Type::U16 => ir::Type::U16,
				Type::U32 => ir::Type::U32,
				Type::U64 => ir::Type::U64,
				Type::I8 => ir::Type::I8,
				Type::I16 => ir::Type::I16,
				Type::I32 => ir::Type::I32,
				Type::I64 => ir::Type::I64,
				Type::ConstRef { element_type } => ir::Type::ConstRef { element_type: *self.type_ids.get(element_type).unwrap() },
				Type::MutRef { element_type } => ir::Type::MutRef { element_type: *self.type_ids.get(element_type).unwrap() },
				Type::ConstSlice { element_type } => ir::Type::ConstSlice { element_type: *self.type_ids.get(element_type).unwrap() },
				Type::MutSlice { element_type } => ir::Type::MutSlice { element_type: *self.type_ids.get(element_type).unwrap() },
				Type::Array { element_type, length } => ir::Type::Array { element_type: *self.type_ids.get(element_type).unwrap(), length: *length },
				Type::Struct { fields, byte_alignment, byte_size } => ir::Type::F32
			};
			
			self.program.types.insert(*self.type_ids.get(type_name).unwrap(), ir_type);
		}
	}

	fn compile_pipelines(&mut self, definition : &Definition)
	{
		for (pipeline_name, pipeline) in definition.pipelines.iter()
		{
			let variable_ids = HashMap::<String, usize>::new();
			
		}
	}

	fn compile_to_ir_program(&self) -> ir::Program
	{
		let mut program = ir::Program::new();
		//let mut compilation_state : CompilationState = Defaults:default();

		program
	}
}