use anyhow::Context;
use caiman::frontend;
use clap::{App, Arg};
use std::fs::File;
use std::io::prelude::*;

fn main() -> Result<(), anyhow::Error> {
    let default_max_iter_str = frontend::TransformConfig::DEFAULT_MAX_ITERATIONS.to_string();
    let matches = App::new("Caiman Compiler")
        .version("0.0.1")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("path.ron")
                .help("Path to input spec (ron)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("path.rs")
                .help("Path to output")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("action")
                .short("a")
                .long("action")
                .help("Which action to take")
                .default_value("compile")
                .possible_values(&["optimize", "explicate", "compile"]),
        )
        .arg(
            Arg::with_name("max_iterations")
                .long("max-iterations")
                .help("The max number of transformation iterations")
                .value_name("count")
                .default_value(&default_max_iter_str),
        )
        .arg(
            Arg::with_name("transformations")
                .short("t")
                .long("transformations")
                .value_name("transform")
                .help("Which transformations to apply")
                .use_delimiter(true)
                .multiple(true)
                .default_value("basic-cse"),
        )
        .arg(
            Arg::with_name("print_codegen_debug_info")
                .long("print_codegen_debug_info")
                .help("Print Codegen Debug Info")
                .takes_value(false),
        )
        .get_matches();

    let action = match matches.value_of("action") {
        Some("optimize") => frontend::Action::Optimize,
        Some("explicate") => frontend::Action::Explicate,
        _ => frontend::Action::Compile,
    };

    let transform_config = {
        let max_iterations = matches
            .value_of("max_iterations")
            .unwrap_or(&default_max_iter_str)
            .parse()
            .context("invalid number of transformation iterations")?;
        let mut transform_config = frontend::TransformConfig::new(max_iterations);
        if let Some(transforms) = matches.values_of("transformations") {
            for transform in transforms {
                transform_config.add_transform(transform)?;
            }
        }
        transform_config
    };

    let print_codegen_debug_info = matches.is_present("print_codegen_debug_info");

    let options = frontend::Options {
        action,
        transform_config,
        print_codegen_debug_info,
    };

    let input = {
        let path = matches.value_of("input").context("must have input path")?;
        let mut file = File::open(path).context("couldn't open input file")?;
        let mut input = String::new();
        file.read_to_string(&mut input)
            .context("couldn't read input file")?;
        input
    };

    let mut output_file = {
        let output_path = matches.value_of("output").unwrap_or("a.out");
        File::open(output_path).context("couldn't open output file")?
    };

    let output = frontend::compile(&options, &input)?;
    output_file
        .write(output.as_bytes())
        .context("couldn't write output file")?;

    Ok(())
}
