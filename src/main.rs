extern crate clap;
use clap::{Arg, App, SubCommand};

use caiman::frontend;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

struct Arguments
{
	input_path : String,
	output_path : Option<String>
}

fn compile(input_file : &mut File, output_file : &mut File)
{

	let mut input_string = String::new();
	match input_file.read_to_string(&mut input_string)
	{
		Err(why) => panic!("Couldn't read file: {}", why),
		Ok(_) => ()
	};

	let result : Result<String, caiman::frontend::CompileError> = caiman::frontend::compile_ron_definition(& input_string, None);
	match result
	{
		Err(why) => panic!("Parse error: {}", why),
		Ok(output_string) =>
		{
			write!(output_file, "{}", output_string);
		}
	}
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
		Arguments {input_path : input_match.unwrap().to_string(), output_path}
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
	compile(&mut input_file, &mut output_file);
}
