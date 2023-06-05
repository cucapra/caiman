extern crate clap;

use std::path::Path;

use caiman::assembly::ast as asm;
use caiman_frontend::error;
use caiman_frontend::scheduling_language;
use caiman_frontend::to_ir;
use caiman_frontend::value_language;
use clap::Parser;

#[derive(Parser)]
#[clap(version)]
struct Arguments
{
    filename: String,

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

    #[clap(long, takes_value=true)]
    output: Option<String>,

    // By default it just prints, for now
    #[clap(long)]
    run: bool,
}

fn value_language_stage(args: &Arguments) -> value_language::compiler::Stage
{
    use value_language::compiler::Stage::*;
    let last = Parse;
    if args.parse
    {
        Parse
    }
    else if args.typeelab
    {
        TypeElaborate
    }
    else
    {
        last
    }
}

fn scheduling_language_stage(args: &Arguments) -> scheduling_language::compiler::Stage
{
    use scheduling_language::compiler::Stage::*;
    let last = Parse;
    if args.parse
    {
        Parse
    }
    else
    {
        last
    }
}

fn filenames(filename: &str) -> (String, String)
{
    let value_filename = filename.to_string() + ".vl";
    let sched_filename = filename.to_string() + ".sl";
    (value_filename, sched_filename)
}

fn main()
{
    let args = Arguments::parse();
    let vl_stage = value_language_stage(&args);
    let sl_stage = scheduling_language_stage(&args);
    if args.value_language_only
    {
        value_language::compiler::run_until_stage(&args.filename, vl_stage);
    }
    else if args.scheduling_language_only
    {
        scheduling_language::compiler::run_until_stage(&args.filename, sl_stage);
    }
    else if args.vil
    {
        let run = || -> Result<to_ir::vil::Program, error::Error> {
            let value_ast = value_language::compiler::parse_and_elaborate(&args.filename)?;
            Ok(to_ir::to_vil::value_ast_to_vil(&value_ast))
        };
        match run()
        {
            Ok(p) => println!("{:?}", p),
            Err(e) => println!("Error: {}", e),
        }
    }
    else
    {
        let run = || -> Result<asm::Program, error::Error> {
            let (value_file, scheduling_file) = filenames(&args.filename);
            let value_ast = value_language::compiler::parse_and_elaborate(&value_file)?;
            let schedule_ast = scheduling_language::compiler::parse(&scheduling_file)?;
            let ir = to_ir::go(&value_ast, &schedule_ast).map_err(|e| error::Error {
                kind: e.error.kind,
                location: e.error.location,
                filename: match e.file_kind
                {
                    error::FileKind::Value => value_file,
                    error::FileKind::Scheduling => scheduling_file,
                },
            })?;
            Ok(ir)
        };
        match run()
        {
            Err(e) => println!("{}", e),
            Ok(program) =>
            {
                if !args.run
                {
                    println!("{:#?}", program);
                }
                else
                {
                    explicate_and_execute(args.output, program);
                }
            },
        }
    }
}

fn explicate_and_execute(output: Option<String>, program: asm::Program)
{
    let version = &program.version;
    assert_eq!((version.major, version.minor, version.detailed), (0, 0, 1));

    let definition = caiman::assembly::lowering_pass::lower(program);
    caiman::ir::validation::validate_program(&definition.program);
    let mut codegen = caiman::rust_wgpu_backend::codegen::CodeGen::new(&definition.program);
    codegen.set_print_codgen_debug_info(true);
    let output_string = codegen.generate();
    match output
    {
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