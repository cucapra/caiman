extern crate clap;
use clap::{Arg, App, SubCommand};

use caiman::frontend;
use caiman::pretty_print;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

struct Arguments
{
	input_path : String,
	output_path : Option<String>,
	explicate_only : bool,
	print_codegen_debug_info : bool,
    pretty_print : bool,
}

fn compile(input_file : &mut File, output_file : &mut File, options_opt : Option<caiman::frontend::CompileOptions>)
{

	let mut input_string = String::new();
	match input_file.read_to_string(&mut input_string)
	{
		Err(why) => panic!("Couldn't read file: {}", why),
		Ok(_) => ()
	};

	let result : Result<String, caiman::frontend::CompileError> = caiman::frontend::compile_ron_definition(& input_string, options_opt);
	match result
	{
		Err(why) => panic!("Parse error: {}", why),
		Ok(output_string) =>
		{
			write!(output_file, "{}", output_string);
		}
	}
}

fn explicate(input_file : &mut File, output_file : &mut File)
{

	let mut input_string = String::new();
	match input_file.read_to_string(&mut input_string)
	{
		Err(why) => panic!("Couldn't read file: {}", why),
		Ok(_) => ()
	};

	let result : Result<String, caiman::frontend::CompileError> = caiman::frontend::explicate_ron_definition(& input_string, None);
	match result
	{
		Err(why) => panic!("Parse error: {}", why),
		Ok(output_string) =>
		{
			write!(output_file, "{}", output_string);
		}
	}
}

fn pretty_print(input_file : &mut File) {
    caiman::pretty_print::print_file(input_file);
}

fn main()
{
	let arguments =
	{
		let matches =
			App::new("Caiman Compiler")
			.version("0.0.1")
			.arg
			(
				Arg::with_name("input")
					.short("i")
					.long("input")
					.value_name("path.ron")
					.help("Path to input spec (ron)")
					.takes_value(true)
			)
			.arg
			(
				Arg::with_name("output")
					.short("o")
					.long("output")
					.value_name("path.rs")
					.help("Path to output code (rust)")
					.takes_value(true)
			)
			.arg
			(
				Arg::with_name("explicate_only")
					.short("x")
					.long("explicate_only")
					.help("Only run schedule explication")
					.takes_value(false)
			)
            .arg
            (
                Arg::with_name("pretty_print")
                    .long("pretty")
                    .help("Print a readable format of input file to stdout")
                    .takes_value(false)
            )
			.arg
			(
				Arg::with_name("print_codegen_debug_info")
					.long("print_codegen_debug_info")
					.help("Print Codegen Debug Info")
					.takes_value(false)
			)
			.get_matches();
		let input_match = matches.value_of("input");
		if input_match.is_none()
		{
			panic!("Must have input path");
		}
		let output_path = match matches.value_of("output")
		{
			Some(path) => Some(path.to_string()),
			None => None
		};
		let explicate_only = matches.is_present("explicate_only");
		let print_codegen_debug_info = matches.is_present("print_codegen_debug_info");
		let pretty_print = matches.is_present("pretty_print");
		Arguments {
            input_path : input_match.unwrap().to_string(), 
            output_path, 
            explicate_only, 
            print_codegen_debug_info, 
            pretty_print,
        }
	};

	let input_path = Path::new(& arguments.input_path);
    let output_path = match & arguments.output_path
    {
        Some(output_path) => output_path.clone(),
        None => String::from("a.out")
    };

	let mut input_file = match File::open(& input_path)
	{
		Err(why) => panic!("Couldn't open {}: {}", input_path.display(), why),
		Ok(file) => file
	};
    let mut output_file = File::create(output_path).unwrap();

    if arguments.explicate_only
    {
        explicate(&mut input_file, &mut output_file);
    }
    else if arguments.pretty_print
    {
        pretty_print(&mut input_file);
    }
    else
    {
        let options = caiman::frontend::CompileOptions{print_codegen_debug_info : arguments.print_codegen_debug_info};
        compile(&mut input_file, &mut output_file, Some(options));
    }

}
