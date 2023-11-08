extern crate clap;
use std::path::Path;

use clap::Parser;
use hlc::{error, lower::lower, parse};

#[derive(Parser)]
#[clap(version)]
struct Arguments {
    /// The file to compile.
    filename: String,

    /// When this flag is enabled, the compiler will only parse the file and
    /// print the AST.
    #[clap(long)]
    parse: bool,

    /// When this flag is enabled, the compiler will only lower the AST and
    /// print the lowered caiman assembly AST.
    #[clap(long)]
    lower: bool,

    /// When this parameter is set, outputs the compiled code to the given file.
    #[clap(long, short, takes_value = true)]
    output: Option<String>,
}

fn main() -> Result<(), error::Error> {
    let args = Arguments::parse();
    compile_new_lang(args)
}

fn compile_new_lang(args: Arguments) -> Result<(), error::Error> {
    let ast = parse::parse_file(&args.filename)?;
    if args.parse {
        println!("{:#?}", ast);
    } else {
        let lowered = lower(ast).map_err(|e| error::Error {
            error: e,
            filename: args.filename.clone(),
        })?;
        if args.lower {
            println!("{:#?}", lowered);
        } else {
            explicate_and_execute(args.output, lowered);
        }
    }
    Ok(())
}

// TODO: unified CLI
fn explicate_and_execute(output: Option<String>, program: caiman::assembly::ast::Program) {
    let version = &program.version;
    assert_eq!((version.major, version.minor, version.detailed), (0, 0, 2));

    let definition = caiman::assembly::lowering_pass::lower(program);
    caiman::ir::validation::validate_program(&definition.program);
    let mut codegen = caiman::rust_wgpu_backend::codegen::CodeGen::new(&definition.program);
    codegen.set_print_codgen_debug_info(true);
    let output_string = codegen.generate();
    match output {
        None => println!("{}", output_string),
        Some(path_str) => {
            // Copied from caiman/src/main.rs
            let path = Path::new(&path_str);
            let prefix = path.parent().unwrap();
            std::fs::create_dir_all(prefix).unwrap();
            std::fs::write(path, output_string).unwrap();
        }
    }
}
