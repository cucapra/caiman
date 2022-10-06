extern crate clap;

use caiman_frontend::stage;
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
    if args.parse {
        Parse
    } else if args.check {
        Check
    } else {
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
        panic!("TODO: SCHED");
    } 
    else 
    {
        let (_value_file, _scheduling_file) = filenames(&args.filename);
        panic!("TODO: the real compiler");
    }
}
