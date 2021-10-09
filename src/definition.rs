// User friendly definition of pipeline blocks
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_derive::{Serialize, Deserialize};

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
	Float32,
	Float64,
	UInt8,
	UInt16,
	UInt32,
	UInt64,
	SInt8,
	SInt16,
	SInt32,
	SInt64,
	ConstRef { type_name : String },
	MutRef { type_name : String },
	ConstSlice { type_name : String },
	MutSlice { type_name : String },
	Array { type_name : String, length : usize },
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
	Float64(f64)
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