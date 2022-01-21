use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LanguageSet
{
	pub implicit : bool,
	pub explicit : bool
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PlaceKind
{
	Local,
	Cpu,
	Gpu,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OperationInputKind
{
	Type,
	Place,
	ImmediateI64,
	ImmediateU64,
	Operation{ required_place_kind : Option<PlaceKind> },
	ExternalCpuFunction,
	ExternalGpuFunction,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OperationInput
{
	pub name : String,
	pub kind : OperationInputKind,
	pub is_array : bool
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OperationOutput
{
	Single{assured_place_kind : Option<PlaceKind>},
	Multiple{assured_place_kind : Option<PlaceKind>},
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Operation
{
	pub name : String,
	pub inputs : Vec<OperationInput>,
	pub output : OperationOutput,
	pub language_set : LanguageSet
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Spec
{
	pub operations : Vec<Operation>
}
