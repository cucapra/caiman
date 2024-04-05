#![warn(clippy::pedantic, clippy::nursery)]
#![warn(clippy::all, clippy::correctness)]
extern crate clap;
mod error;
mod lower;
mod normalize;
mod parse;
mod typing;

use clap::Parser;
use lower::lower;

#[derive(Parser)]
#[clap(version)]
#[allow(clippy::struct_excessive_bools)]
struct Arguments {
    /// The file to compile.
    filename: String,

    /// When this flag is enabled, the compiler will only parse the file and
    /// print the AST.
    #[clap(long)]
    parse: bool,

    /// When this flag is enabled, the compiler will only lower the AST and
    /// print the lowered caiman assembly AST.
    #[clap(long)]
    lower: bool,

    /// When this flag is enabled, the compiler will only explicate the lowered
    /// AST and print the result
    #[clap(long, alias = "ir")]
    explicate_only: bool,

    /// When this flag is enabled, the compiler will typecheck the data types of
    /// the program and print the result. This DOES NOT typecheck the quotients
    /// or flows nor does it check for spec violations.
    #[clap(long, alias = "ty")]
    typecheck: bool,

    /// When this flag is enabled, the compiler will normalize the AST and print
    /// the result.
    #[clap(long, alias = "norm")]
    normalize: bool,

    /// When this parameter is set, outputs the compiled code to the given file.
    #[clap(long, short, takes_value = true)]
    output: Option<String>,

    /// When this flag is enabled, the compiler will not print any output
    /// except for errors.
    #[clap(long, short)]
    quiet: bool,
}

fn main() -> Result<(), error::Error> {
    let args = Arguments::parse();
    compile_new_lang(args)
}

fn compile_new_lang(args: Arguments) -> Result<(), error::Error> {
    let ast = match args.filename.as_str() {
        "-" => parse::parse_read(std::io::stdin(), "stdin").map_err(|e| error::Error {
            error: e,
            filename: "stdin".to_string(),
        })?,
        filename => parse::parse_file(filename)?,
    };
    if args.parse {
        if !args.quiet {
            println!("{ast:#?}");
        }
        return Ok(());
    }
    let ast = normalize::normalize_ast(ast).map_err(|e| error::Error {
        error: e,
        filename: args.filename.clone(),
    })?;
    if args.normalize {
        if !args.quiet {
            println!("{ast:#?}");
        }
        return Ok(());
    }
    let ctx = typing::Context::new(&ast).map_err(|e| error::Error {
        error: e,
        filename: args.filename.clone(),
    })?;
    if args.typecheck {
        if !args.quiet {
            println!("Data types valid");
        }
        return Ok(());
    }
    let lowered = lower(ast, &ctx).map_err(|e| error::Error {
        error: e,
        filename: args.filename.clone(),
    })?;
    if args.lower {
        if !args.quiet {
            println!("{lowered:#?}");
        }
        return Ok(());
    }
    caiman::explicate_and_execute(args.output, lowered, args.explicate_only);
    Ok(())
}
