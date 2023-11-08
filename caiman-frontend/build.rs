use caiman_spec::spec;
use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::Path;

fn op_is_value_expr(op: &spec::Operation) -> bool {
    if let spec::OperationOutput::None = op.output {
        return false;
    }
    op.language_set.functional
}

fn write_spec_nodes_module(spec: &spec::Spec) -> Result<(), std::io::Error> {
    let mut f = fs::File::create("src/spec/nodes.rs")?;
    write!(
        f,
        "// This file was auto-generated by the build script!\n\n"
    )?;
    write!(f, "use caiman::ir;\nuse super::input;\n\n")?;
    write!(f, "#[derive(Debug, Clone)]\n")?;
    write!(f, "pub enum FunctionalExprNodeKind {{\n")?;
    for op in spec.operations.iter() {
        if op_is_value_expr(op) {
            write!(f, "    {},\n", &op.name)?;
        }
    }
    write!(f, "}}\n\n")?;
    write!(f, "impl FunctionalExprNodeKind {{\n")?;
    write!(
        f,
        "    pub fn to_ir(&self, mut v: Vec<input::SpecNodeInput>) -> ir::Node {{\n"
    )?;
    write!(f, "        use FunctionalExprNodeKind::*;")?;
    write!(f, "        match self {{\n")?;
    for op in spec.operations.iter() {
        if op_is_value_expr(op) {
            write!(f, "            {} => ir::Node::{} {{", &op.name, &op.name)?;
            // This should have been type-checked by this point so we can
            // safely index the vector all crazy like this
            for (i, input) in op.inputs.iter().enumerate() {
                use spec::OperationInputKind as OIK;
                let type_base = match &input.kind {
                    /*OIK::ImmediateI64 => "i64",
                    OIK::ImmediateU64 => "u64",
                    OIK::ImmediateI32 => "i32",*/
                    OIK::ImmediateI64 | OIK::ImmediateU64 | OIK::Immediate | OIK::ImmediateI32 => {
                        "irconstant"
                    }
                    OIK::ExternalFunction => "externalfunction",
                    _ => "usize",
                };
                let type_extn = if input.is_array { "_slice" } else { "" };
                write!(
                    f,
                    "{}: std::mem::take(&mut v[{}]).unwrap_{}{}(), ",
                    input.name, i, type_base, type_extn
                )?;
            }
            write!(f, "}},\n")?;
        }
    }
    write!(f, "        }}\n")?;
    write!(f, "    }}\n\n")?;
    write!(f, "}}\n\n")?;
    Ok(())
}

fn append_lalrpop_parser(f: &mut fs::File, spec: &spec::Spec) -> Result<(), std::io::Error> {
    write!(f, "IRNodeE: spec::nodes::FunctionalExprNodeKind = {{\n")?;
    for op in spec.operations.iter() {
        if op_is_value_expr(op) {
            write!(
                f,
                "\"IR::{}\" => spec::nodes::FunctionalExprNodeKind::{},\n",
                &op.name, &op.name
            )?;
        }
    }
    write!(f, "}}\n")?;
    Ok(())
}

fn build_lalrpop_parser(src_dir: &Path, spec: &spec::Spec) {
    let nodeless_parser = src_dir.join("parser.lalrpop");
    let generated_parser = src_dir.join("generated_parser.lalrpop");
    fs::copy(&nodeless_parser, &generated_parser).unwrap();
    let mut file_to_append = fs::OpenOptions::new()
        .append(true)
        .open(generated_parser)
        .unwrap();
    append_lalrpop_parser(&mut file_to_append, spec).unwrap();
}

fn build_rust_parser(src_dir: &Path) -> Result<(), Box<dyn Error>> {
    let /*mut*/ config = lalrpop::Configuration::new();
    config.process_file(src_dir.join("generated_parser.lalrpop"))
}

fn main() {
    // let spec = caiman_spec::content::build_spec();
    // write_spec_nodes_module(&spec).unwrap();
    // let src_dir = Path::new("./src/value_language/");
    // build_lalrpop_parser(&src_dir, &spec);
    // build_rust_parser(&src_dir).unwrap();
    // fs::rename(
    //     src_dir.join("generated_parser.rs"),
    //     src_dir.join("parser.rs"),
    // )
    // .unwrap();

    // // Just make scheduling language parser normally
    // let sched_parser_config = lalrpop::Configuration::new();
    // sched_parser_config
    //     .process_file(Path::new("./src/scheduling_language/parser.lalrpop"))
    //     .unwrap();

    // Make "both parser" normally too
    let parser_config = lalrpop::Configuration::new();
    parser_config
        .process_file(Path::new("./src/parse/parser.lalrpop"))
        .unwrap();
}
