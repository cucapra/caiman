extern crate clap;

use clap::Parser;
use caiman_frontend::parse_file;

#[derive(Parser)]
#[clap(version)]
struct Arguments 
{
    filename: String,
}

fn main() 
{
    let args = Arguments::parse();

    let ast = parse_file(String::from(args.filename));
    for decl in ast.iter() 
    {
        println!("{:?}", decl);
    }
}

