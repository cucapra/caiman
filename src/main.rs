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
		let input_path = std::env::args().nth(1).expect("No input");
		let output_path = None;
		Arguments {input_path, output_path}
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
