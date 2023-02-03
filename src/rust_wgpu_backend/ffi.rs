use serde_derive::{Serialize, Deserialize};
use crate::stable_vec::StableVec;

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
	// Value types
	F32,
	F64,
	U8,
	U16,
	U32,
	U64,
	USize,
	I8,
	I16,
	I32,
	I64,
	Array { element_type : TypeId, length : usize },
	ErasedLengthArray { element_type : TypeId },
	Struct { fields : Box<[StructField]>, byte_alignment : Option<usize>, byte_size : Option<usize> },
	Tuple { fields : Box<[TypeId]> },

	// Reference types
	ConstRef { element_type : TypeId },
	MutRef { element_type : TypeId },
	ConstSlice { element_type : TypeId },
	MutSlice { element_type : TypeId },
	GpuBufferRef { element_type : TypeId },
	GpuBufferSlice { element_type : TypeId },
	GpuBufferAllocator,
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

impl ExternalGpuFunction
{
	// Once we stop needing to directly program in .ron files, we can make this more efficient
	pub fn output_of_forwarding_input(& self, input_index : usize) -> Option<usize>
	{
		for resource_binding in self.resource_bindings.iter()
		{
			if let Some(index) = resource_binding.input
			{
				if index == input_index
				{
					return resource_binding.output;
				}
			}
		}

		return None;
	}
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct NativeInterface
{
	#[serde(default)]
	pub types : StableVec<Type>,
	#[serde(default)]
	pub external_cpu_functions : StableVec<ExternalCpuFunction>,
	#[serde(default)]
	pub external_gpu_functions : StableVec<ExternalGpuFunction>,
}

pub struct TypeBindingInfo
{
	pub size : usize,
	pub alignment : usize,
}

impl NativeInterface
{
	// For the sake of an MVP, this is assuming that we're compiling for the machine we're running on

	pub fn calculate_type_binding_info(&self, type_id : TypeId) -> TypeBindingInfo
	{
		match & self.types[type_id.0]
		{
			Type::F32 => TypeBindingInfo { size : std::mem::size_of::<f32>(), alignment : std::mem::align_of::<f32>() },
			Type::F64 => TypeBindingInfo { size : std::mem::size_of::<f64>(), alignment : std::mem::align_of::<f64>() },
			Type::U8 => TypeBindingInfo { size : std::mem::size_of::<u8>(), alignment : std::mem::align_of::<u8>() },
			Type::U16 => TypeBindingInfo { size : std::mem::size_of::<u16>(), alignment : std::mem::align_of::<u16>() },
			Type::U32 => TypeBindingInfo { size : std::mem::size_of::<u32>(), alignment : std::mem::align_of::<u32>() },
			Type::U64 => TypeBindingInfo { size : std::mem::size_of::<u64>(), alignment : std::mem::align_of::<u64>() },
			Type::USize => TypeBindingInfo { size : std::mem::size_of::<usize>(), alignment : std::mem::align_of::<usize>() },
			Type::I8 => TypeBindingInfo { size : std::mem::size_of::<i8>(), alignment : std::mem::align_of::<i8>() },
			Type::I16 => TypeBindingInfo { size : std::mem::size_of::<i16>(), alignment : std::mem::align_of::<i16>() },
			Type::I32 => TypeBindingInfo { size : std::mem::size_of::<i32>(), alignment : std::mem::align_of::<i32>() },
			Type::I64 => TypeBindingInfo { size : std::mem::size_of::<i64>(), alignment : std::mem::align_of::<i64>() },
			Type::ConstRef { element_type } => panic!("Unimplemented"),
			Type::MutRef { element_type } => panic!("Unimplemented"),
			Type::ConstSlice { element_type } => panic!("Unimplemented"),
			Type::MutSlice { element_type } => panic!("Unimplemented"),
			Type::Array { element_type, length } => panic!("Unimplemented"),
			Type::Struct { fields, byte_alignment, byte_size } => panic!("Unimplemented"),
			_ => panic!("Unimplemented")
		}
	}

	pub fn calculate_type_alignment_bits(&self, type_id : TypeId) -> usize
	{
		self.calculate_type_binding_info(type_id).alignment.trailing_zeros() as usize
	}

	pub fn calculate_type_byte_size(&self, type_id : TypeId) -> usize
	{
		self.calculate_type_binding_info(type_id).size
	}
}