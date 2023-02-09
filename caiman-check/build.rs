use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use caiman::frontend::CompileMode;

fn compile(input_file : &mut File, output_file : &mut File)
{

    let mut input_string = String::new();
    match input_file.read_to_string(&mut input_string)
    {
        Err(why) => panic!("Couldn't read file: {}", why),
        Ok(_) => ()
    };

    let result : Result<String, caiman::frontend::CompileError> = caiman::frontend::compile_caiman(& input_string,
       caiman::frontend::CompileOptions{print_codegen_debug_info : true, compile_mode : CompileMode::Assembly});

    match result
    {
        Err(why) => panic!("Parse error: {}", why),
        Ok(output_string) =>
            {
                write!(output_file, "{}", output_string).expect("Invalid file");
            }
    }
}

fn main()
{
    println!("cargo:rerun-if-changed=src/test.cair");
    println!("cargo:rerun-if-changed=../src/");

    let input_path = Path::new(& "src/test.cair");
    let mut input_file = match File::open(& input_path)
    {
        Err(why) => panic!("Couldn't open {}: {}", input_path.display(), why),
        Ok(file) => file
    };

    let mut output_file = File::create("src/output.out").unwrap();
    compile(&mut input_file, &mut output_file);
}