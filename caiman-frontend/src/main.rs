extern crate clap;

use std::path::Path;

use caiman::assembly::ast as asm;
use caiman_frontend::error;
use caiman_frontend::syntax;
use caiman_frontend::to_ir;
use clap::Parser;

#[derive(Parser)]
#[clap(version)]
struct Arguments
{
    filename: String,

    // XXX this is temporary! eventually it'll just be the default
    #[clap(long)]
    new_lang: bool,

    #[clap(short)]
    value_language_only: bool,
    #[clap(short)]
    scheduling_language_only: bool,

    #[clap(long)]
    parse: bool,
    #[clap(long)]
    typeelab: bool,
    #[clap(long)]
    vil: bool,

    #[clap(long, takes_value = true)]
    output: Option<String>,

    // By default it just prints, for now
    #[clap(long)]
    run: bool,
}

fn filenames(filename: &str) -> (String, String)
{
    let value_filename = filename.to_string() + ".cavl";
    let sched_filename = filename.to_string() + ".casl";
    (value_filename, sched_filename)
}

fn main()
{
    let args = Arguments::parse();

    if args.new_lang {
        compile_new_lang(args);
        return;
    }
    else {
        panic!("Old lang is currently broken :(");
    }
}

fn compile_new_lang(args: Arguments) 
{
    match syntax::compile::parse(args.filename) {
        Err(e) => println!("{}", e),
        Ok(ast) => {
            if args.parse {
                println!("{:#?}", ast);
            } else {
                let program = to_ir::frontend_to_asm(ast);
                if args.run {
                    explicate_and_execute(args.output, program);
                } else {
                    println!("{:#?}", program);
                }
            }
        },
    }
}

fn explicate_and_execute(output: Option<String>, program: asm::Program)
{
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
        },
    }
}
