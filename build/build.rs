use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::io::Write;
use caiman_spec::spec;

fn get_input_kind_type_name(kind : &spec::OperationInputKind) -> String
{
	use spec::OperationInputKind;
	match kind
	{
		OperationInputKind::Type => String::from("TypeId"),
		OperationInputKind::ImmediateI64 => String::from("i64"),
		OperationInputKind::ImmediateU64 => String::from("u64"),
		OperationInputKind::ExternalCpuFunction => String::from("ExternalCpuFunctionId"),
		OperationInputKind::ExternalGpuFunction => String::from("ExternalGpuFunctionId"),
		OperationInputKind::Operation{ required_place_kind } => String::from("OperationId"),
		_ => panic!("Unimplemented input kind: {:?}", kind)
	}
}

fn write_ir_definition(output_file : &mut File, specification : &spec::Spec)
{
	let built_in_node_string = "
	Phi { index : usize },
	ComputedResult { node_ids : Box<[NodeId]> },
	ExtractResult { node_id : NodeId, index : usize },

	GpuTaskStart{ local_variable_node_ids : Box<[NodeId]>, gpu_resident_node_ids : Box<[NodeId]> },
	GpuTaskEnd{ task_node_id : NodeId, local_variable_node_ids : Box<[NodeId]>, gpu_resident_node_ids : Box<[NodeId]> },
";

	write!(output_file, "#[derive(Serialize, Deserialize, Debug, Clone)]\n");
	write!(output_file, "pub enum Node\n{{");
	write!(output_file, "{}", built_in_node_string);
	for operation in specification.operations.iter()
	{
		write!(output_file, "\t{}", operation.name);
		if operation.inputs.len() > 0
		{
			write!(output_file, "{}", " {");
			for input in operation.inputs.iter()
			{
				if input.is_array
				{
					write!(output_file, "{} : Box<[{}]>, ", input.name, get_input_kind_type_name(& input.kind));
				}
				else
				{
					write!(output_file, "{} : {}, ", input.name, get_input_kind_type_name(& input.kind));
				}
			}
			write!(output_file, "{}", "}");
		}
		write!(output_file, "{}", ",\n");
	}
	write!(output_file, "{}", "}\n\n");
	write!(output_file, "{}", "");
}

fn main()
{
	println!("cargo:rerun-if-changed=build/build.rs");

	let specification = caiman_spec::content::build_spec();
	let out_dir = std::env::var("OUT_DIR").unwrap();
	let generated_path = format!("{}/generated", out_dir);
	std::fs::create_dir(&generated_path);
	let mut output_file = File::create(format!("{}/generated/ir.txt", out_dir)).unwrap();
	write_ir_definition(&mut output_file, & specification);
}
