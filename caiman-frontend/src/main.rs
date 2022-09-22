extern crate clap;

use clap::Parser;
use caiman_frontend::compiler;

#[derive(Parser)]
#[clap(version)]
struct Arguments 
{
    filename: String,

    #[clap(short)]
    parse: bool,
    check: bool,
}

fn calc_stage(args: &Arguments) -> compiler::Stage
{
    use compiler::Stage::*;
    let last = Check;
    if      args.parse    { Parse }
    else if args.check    { Check }
    else                  { last }
}

fn main() 
{
    let args = Arguments::parse();
    let stage = calc_stage(&args);
    compiler::run(&args.filename, stage);
}

