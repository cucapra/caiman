use serde_derive::{Serialize, Deserialize};

fn default_true() -> bool
{
	true
}

fn default_false() -> bool
{
	false
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LanguageSet
{
	pub functional : bool,
	pub scheduling : bool,
	pub timeline : bool,
	pub spatial : bool,
	pub intrinsic : bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PlaceKind
{
	Local,
	Cpu,
	Gpu,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum OperationInputKind
{
	Type,
	Place,
	ImmediateI64,
	ImmediateU64,
	Index,
	Operation,
	ExternalCpuFunction,
	ExternalGpuFunction,
	ValueFunction,
	Funclet,
	RemoteOperation,
	StorageType
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OperationInput
{
	pub name : String,
	pub kind : OperationInputKind,
	#[serde(default = "default_false")]
	pub is_array : bool,
	#[serde(default = "default_false")]
	pub is_inferable : bool
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OperationOutput
{
	Single,
	Multiple,
	None,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Operation
{
	pub name : String,
	pub inputs : Vec<OperationInput>,
	pub output : OperationOutput,
	pub language_set : LanguageSet,
	pub has_local_side_effect : bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ValueType
{

}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResourceType
{
	
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Spec
{
	pub operations : Vec<Operation>
}
