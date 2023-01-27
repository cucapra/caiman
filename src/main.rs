extern crate clap;

use std::env;
use clap::{Arg, App, SubCommand};

use caiman::frontend;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

struct Arguments
{
	input_path : String,
	output_path : Option<String>,
	explicate_only : bool,
	print_codegen_debug_info : bool
}

fn output_result(result : Result<String, caiman::frontend::CompileError>, output_file : &mut Option<File>) {
	match result
	{
		Err(why) => panic!("Parse error: {}", why),
		Ok(output_string) =>
			{
				match output_file {
					Some(out_file) => { write!(out_file, "{}", output_string); },
					None => { print!("{}", output_string); }
				}
			}
	}
}

fn compile(input_file : &mut File, output_file : &mut Option<File>,
		   options_opt : Option<caiman::frontend::CompileOptions>)
{

	let mut input_string = String::new();
	match input_file.read_to_string(&mut input_string)
	{
		Err(why) => panic!("Couldn't read file: {}", why),
		Ok(_) => ()
	};

	let result = caiman::frontend::compile_caiman
		(& input_string, options_opt, true);
	output_result(result, output_file);
}

fn explicate(input_file : &mut File, output_file : &mut Option<File>)
{

	let mut input_string = String::new();
	match input_file.read_to_string(&mut input_string)
	{
		Err(why) => panic!("Couldn't read file: {}", why),
		Ok(_) => ()
	};

	let result : Result<String, caiman::frontend::CompileError> =
		caiman::frontend::explicate_caiman(& input_string, None, true);
	output_result(result, output_file);
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
					.value_name("path.caimanir")
					.help("Path to input assembly (caimanir)")
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
		Arguments {input_path : input_match.unwrap().to_string(), output_path, explicate_only, print_codegen_debug_info}
	};

	let input_path = Path::new(& arguments.input_path);
	let mut output_file = match & arguments.output_path
	{
		Some(output_path) => Some(File::create(output_path).unwrap()),
		None => None
	};
	
	let mut input_file = match File::open(& input_path)
	{
		Err(why) => panic!("Couldn't open {}: {}", input_path.display(), why),
		Ok(file) => file
	};

	if arguments.explicate_only
	{
		explicate(&mut input_file, &mut output_file);
	}
	else
	{
		let options = caiman::frontend::CompileOptions{print_codegen_debug_info : arguments.print_codegen_debug_info};
		compile(&mut input_file, &mut output_file, Some(options));
	}
}
