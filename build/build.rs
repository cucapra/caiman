use caiman_spec::spec;
use std::fs::File;
use std::io::Write;

fn operation_language(operation: &spec::Operation) -> &'static str {
    match (
        operation.language_set.functional,
        operation.language_set.scheduling,
        operation.language_set.timeline,
        operation.language_set.spatial,
    ) {
        (true, true, true, true) => "mixed",
        (true, false, false, false) => "functional",
        (false, true, false, false) => "scheduling",
        (false, false, true, false) => "timeline",
        (false, false, false, true) => "spatial",
        (_, _, _, _) => panic!("Unknown language combination {:?}", operation),
    }
}

fn operation_output(operation: &spec::Operation) -> &'static str {
    match operation.output {
        spec::OperationOutput::None => "None",
        spec::OperationOutput::Single => "Single",
        spec::OperationOutput::Multiple => "Multiple",
    }
}

fn input_type(input: &spec::OperationInput) -> String {
    use spec::OperationInputKind as OK;
    let base = match input.kind {
        OK::Type => "Type",
        OK::Place => "Place",
        OK::ImmediateI64 => "ImmediateI64",
        OK::ImmediateI32 => "ImmediateI32",
        OK::ImmediateU64 => "ImmediateU64",
        OK::Index => "Index",
        OK::Operation => "Operation",
        OK::RemoteOperation => "RemoteOperation",
        OK::ExternalCpuFunction => "ExternalCpuFunction",
        OK::ExternalGpuFunction => "ExternalGpuFunction",
        OK::ValueFunction => "ValueFunction",
        OK::Funclet => "Funclet",
        OK::StorageType => "StorageType",
    };
    if input.is_array {
        format!("[{base}]")
    } else {
        base.to_owned()
    }
}

fn write_with_operations(out: &mut File, spec: &spec::Spec) -> std::io::Result<()> {
    write!(out, "macro_rules! with_operations {{\n")?;
    write!(out, "\t($macro:ident) => {{\n")?;
    write!(out, "\t\t$macro! {{\n")?;
    for operation in spec.operations.iter() {
        write!(
            out,
            "\t\t\t{} {} (",
            operation_language(operation),
            operation.name,
        )?;
        for input in operation.inputs.iter() {
            write!(out, "{}: {}, ", input.name, input_type(input))?;
        }
        write!(out, ") -> {};\n", operation_output(operation))?;
    }
    write!(out, "\t\t}}\n")?;
    write!(out, "\t}}\n")?;
    write!(out, "}}\n")
}

fn main() {
    println!("cargo:rerun-if-changed=build/build.rs");
    let spec = caiman_spec::content::build_spec();
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let gen_dir = format!("{out_dir}/generated");
    let _ = std::fs::create_dir(&gen_dir);
    {
        let path = format!("{gen_dir}/with_operations.rs");
        let mut out = File::create(path).unwrap();
        write_with_operations(&mut out, &spec).unwrap();
    }
}
