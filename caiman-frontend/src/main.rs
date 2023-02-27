extern crate clap;

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
    else
    {
        let run = || -> Result<caiman::ir::Program, error::Error> {
            let (value_file, scheduling_file) = filenames(&args.filename);
            let value_ast = value_language::compiler::parse(&value_file)?;
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
            Ok(ir) => println!("{:#?}", ir),
            Err(e) => println!("{}", e),
        }
    }
}