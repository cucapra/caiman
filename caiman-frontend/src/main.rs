extern crate clap;

use clap::Parser;
use caiman_frontend::ir_version;
use caiman_frontend::value_language;

#[derive(Parser)]
#[clap(version)]
struct Arguments 
{
    filename: String,

    #[clap(short)]
    ir_version: bool,
    #[clap(short)]
    value_language: bool,
}

fn main() 
{
    let args = Arguments::parse();

    if args.ir_version 
    {
        match ir_version::parse_file(String::from(args.filename.clone()))
        {
            Ok(ast) => {
                for decl in ast.iter() 
                {
                    println!("{:?}", decl);
                }
            },
            Err(why) => {
                println!("Parser error: {}", why)
            },
        }
    }
    if args.value_language
    {
        match value_language::parse_file(String::from(args.filename).clone())
        {
            Ok(ast) => {
                for statement in ast.iter() 
                {
                    println!("{:?}", statement);
                }
            },
            Err(why) => {
                println!("Parser error: {}", why)
            },
        }
    }
}

