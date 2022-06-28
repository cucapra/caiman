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

    match parse_file(String::from(args.filename))
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

