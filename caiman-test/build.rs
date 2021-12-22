use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

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
	println!("cargo:rerun-if-changed=src/pipelines.ron");

	let input_path = Path::new(& "src/pipelines.ron");
	let mut input_file = match File::open(& input_path)
	{
		Err(why) => panic!("Couldn't open {}: {}", input_path.display(), why),
		Ok(file) => file
	};

	let out_dir = std::env::var("OUT_DIR").unwrap();
	let generated_path = format!("{}/generated", out_dir);
	std::fs::create_dir(&generated_path);
	let mut output_file = File::create(format!("{}/generated/pipelines.txt", out_dir)).unwrap();
	compile(&mut input_file, &mut output_file);
}
