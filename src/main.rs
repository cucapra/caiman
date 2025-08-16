extern crate clap;

use clap::Arg;

use caiman::frontend;
use caiman::frontend::{CompileData, CompileMode, CompileOptions};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Tries to run `rustfmt` on the given path.
fn format(p: &Path) {
    let _ = Command::new("rustfmt")
        .arg("-q")
        .arg("--")
        .arg(p.as_os_str())
        .status();
}

struct Arguments {
    input: PathBuf,
    output: Option<PathBuf>,
    explicate_only: bool,
    print_codegen_debug_info: bool,
}
impl Arguments {
    fn from_cmdline() -> Self {
        let matches = clap::Command::new("Caiman Compiler")
            .version("0.0.1")
            .arg(
                Arg::new("input")
                    .short('i')
                    .long("input")
                    .value_name("path.cair")
                    .help("Path to input assembly (caimanir)")
                    .num_args(1),
            )
            .arg(
                Arg::new("output")
                    .short('o')
                    .long("output")
                    .value_name("path.rs")
                    .help("Path to output code (rust)")
                    .num_args(1),
            )
            .arg(
                Arg::new("explicate_only")
                    .short('x')
                    .long("explicate_only")
                    .help("Only run schedule explication")
                    .num_args(0)
            )
            .arg(
                Arg::new("print_codegen_debug_info")
                    .long("print_codegen_debug_info")
                    .help("Print Codegen Debug Info")
                    .num_args(0),
            )
            .get_matches();
        let input = matches
            .get_one::<String>("input")
            .expect("Must have input path")
            .into();
        let output = matches.get_one::<String>("output").map(PathBuf::from);
        let explicate_only = matches.contains_id("explicate_only");
        let print_codegen_debug_info = matches.contains_id("print_codegen_debug_info");
        Arguments {
            input,
            output,
            explicate_only,
            print_codegen_debug_info,
        }
    }
}

fn main() {
    let args = Arguments::from_cmdline();
    let compile_mode = match args
        .input
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap()
    {
        "cair" => CompileMode::Assembly,
        "ron" => CompileMode::RON,
        _ => panic!("Unsupported file extension for {:?}", args.input),
    };

    let input_string = std::fs::read_to_string(&args.input).expect("couldn't read input");
    let compile_info = CompileData {
        path: match args.input.parent() {
            None => "".to_string(),
            Some(s) => s.to_str().unwrap().to_string(),
        },
        input_string,
    };
    let options = CompileOptions {
        print_codegen_debug_info: args.print_codegen_debug_info,
        compile_mode,
    };

    let result = if args.explicate_only {
        frontend::explicate_caiman(compile_info, options)
    } else {
        frontend::compile_caiman(compile_info, options)
    };

    let output_string = result.expect("compiler error");
    match args.output.as_ref() {
        Some(path) => {
            // https://stackoverflow.com/a/59046435/5031773
            let prefix = path.parent().unwrap();
            std::fs::create_dir_all(prefix).unwrap();
            std::fs::write(path, output_string).unwrap();
            if !args.explicate_only {
                format(path);
            }
        }
        None => {
            print!("{output_string}");
        }
    }
}
