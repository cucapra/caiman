use anyhow::Context;
use caiman::frontend;
use clap::{App, Arg};
use std::{fs::File, io::Read, io::Write};

fn main() -> Result<(), anyhow::Error> {
    let default_max_iter_str = frontend::TransformConfig::DEFAULT_MAX_PASSES.to_string();
    let matches = App::new("Caiman Compiler")
        .version("0.0.1")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("path.ron")
                .help("Path to input spec (ron)")
                .takes_value(true)
                .number_of_values(1),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("path.rs")
                .help("Path to output")
                .takes_value(true)
                .allow_hyphen_values(true)
                .number_of_values(1),
        )
        .arg(
            Arg::with_name("action")
                .short("a")
                .long("action")
                .help("Which action to take")
                .number_of_values(1)
                .default_value("compile")
                .possible_values(&["optimize", "explicate", "compile"]),
        )
        .arg(
            Arg::with_name("max_passes")
                .long("max-passes")
                .help("The max number of transformation passes")
                .value_name("count")
                .number_of_values(1)
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
                .takes_value(false)
                .number_of_values(1),
        )
        .get_matches();

    let action = match matches.value_of("action") {
        Some("optimize") => frontend::Action::Optimize,
        Some("explicate") => frontend::Action::Explicate,
        _ => frontend::Action::Compile,
    };

    let transform_config = {
        let max_passes = matches
            .value_of("max_passes")
            .unwrap_or(&default_max_iter_str)
            .parse()
            .context("invalid number of transformation passes")?;
        let mut transform_config = frontend::TransformConfig::new(max_passes);
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

    let output = frontend::compile(&options, &input)?;

    let maybe_path = matches.value_of("output");
    if maybe_path == Some("-") {
        std::io::stdout().write(output.as_bytes()).unwrap();
    } else {
        let output_path = maybe_path.unwrap_or("a.out");
        let mut output_file = File::create(output_path).context("couldn't open output file")?;
        output_file
            .write(output.as_bytes())
            .context("couldn't write output file")?;
    }

    Ok(())
}
