//#[macro_use]
//extern crate bitflags;

use caiman::ir;

//#[macro_use]
//extern crate clap;
//use clap::App;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

//#[derive(Clap, Debug)]
//#[clap(name = "dgc")]
struct Arguments
{
	//#[clap(short, long)]
	input : String,

	//#[clap(short, long, default_value = "a.out")]
	//output : String,
}

fn main()
{
	//let arguments = Arguments::parse();
	let arguments =
	{
		/*let matches = clap_app!(dgc =>
			(version : "0.1")
			(author: "Author")
			(@arg INPUT +required "The input toml spec")
		).get_matches();
		let input = value_t!(matches.value_of("INPUT"), String).unwrap_or_else(|e| e.exit());*/
		let input = std::env::args().nth(1).expect("No input");
		Arguments {input : input}
	};

	let input_path = Path::new(& arguments.input);
	//let output_path = Path::new(& arguments.output);
	
	let mut input_file = match File::open(& input_path)
	{
		Err(why) => panic!("Couldn't open {}: {}", input_path.display(), why),
		Ok(file) => file
	};

	/*let mut output_file = match File::open(& output_path)
	{
		Err(why) => panic!("Couldn't open {}: {}", output_path.display(), why),
		Ok(file) => file
	};*/

	let mut input_string = String::new();
	match input_file.read_to_string(&mut input_string)
	{
		Err(why) => panic!("Couldn't read {}: {}", input_path.display(), why),
		Ok(_) => ()
	};

	//let definition : definition::Definition = toml::from_str(& input_string).unwrap();
	//println!("{:?}", definition);

	let program : ir::Program = toml::from_str(& input_string).unwrap();
	println!("{:?}", program);
}
