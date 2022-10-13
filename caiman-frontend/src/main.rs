extern crate clap;

use caiman_frontend::scheduling_language;
use caiman_frontend::stage;
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
    check: bool,
}

fn stage(args: &Arguments) -> stage::Stage
{
    use stage::Stage::*;
    let last = Check;
    if args.parse
    {
        Parse
    }
    else if args.check
    {
        Check
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
    let stage = stage(&args);
    if args.value_language_only
    {
        value_language::compiler::run(&args.filename, stage);
    }
    else if args.scheduling_language_only
    {
        scheduling_language::compiler::run(&args.filename, stage);
    }
    else
    {
        let (value_file, scheduling_file) = filenames(&args.filename);
        let value_ast = value_language::compiler::run_output(&value_file);
        let schedule_ast =
            scheduling_language::compiler::run_output(&scheduling_file);
        let ir = to_ir::go(&value_ast, &schedule_ast);
        println!("{:?}", ir)
    }
}
