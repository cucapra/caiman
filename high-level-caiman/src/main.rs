extern crate clap;
use clap::Parser;
use hlc::{error, parse};

#[derive(Parser)]
#[clap(version)]
struct Arguments {
    /// The file to compile.
    filename: String,

    /// When this flag is specified the compiler will only parse the file
    /// and print the AST.
    #[clap(long)]
    parse: bool,

    /// This parameter specifies the output file.
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
        unimplemented!("Only parse is implemented for now")
    }
    Ok(())
}
