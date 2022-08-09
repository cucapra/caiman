use serde_derive::{Serialize, Deserialize};
use crate::arena::Arena;

#[derive(Serialize, Deserialize, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Default, Hash)]
pub struct TypeId(pub usize); // temporarily exposed internals

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StructField
{
	pub name : String,
	pub type_id : TypeId,
	pub byte_offset : usize,
	pub byte_size : usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
	Array { element_type : TypeId, length : usize },
	Struct { fields : Box<[StructField]>, byte_alignment : Option<usize>, byte_size : Option<usize> },
	Tuple { fields : Box<[TypeId]> },

	ConstRef { element_type : TypeId },
	MutRef { element_type : TypeId },
	ConstSlice { element_type : TypeId },
	MutSlice { element_type : TypeId },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExternalCpuFunction
{
	pub name : String,
	pub input_types : Box<[TypeId]>,
	pub output_types : Box<[TypeId]>,
}

// This describes the initial mapping from the binding in the shader to the IR
// It's expected codegen will emit a module with a different mapping
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExternalGpuFunctionResourceBinding
{
	pub group : usize,
	pub binding : usize,
	pub input : Option<usize>,
	pub output : Option<usize>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ShaderModuleContent
{
	Wgsl(String)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExternalGpuFunction
{
	pub name : String,
	pub input_types : Box<[TypeId]>,
	pub output_types : Box<[TypeId]>,
	// Contains pipeline and single render pass state
	pub entry_point : String,
	pub resource_bindings : Box<[ExternalGpuFunctionResourceBinding]>,
	pub shader_module_content : ShaderModuleContent,
	//pub shader_module : usize,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct NativeInterface
{
	#[serde(default)]
	pub types : Arena<Type>,
	#[serde(default)]
	pub external_cpu_functions : Arena<ExternalCpuFunction>,
	#[serde(default)]
	pub external_gpu_functions : Arena<ExternalGpuFunction>,
}