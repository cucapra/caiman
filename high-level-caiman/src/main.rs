extern crate clap;
use clap::Parser;
use hlc::{error, parse};

#[derive(Parser)]
#[clap(version)]
struct Arguments {
    filename: String,

    #[clap(short)]
    value_language_only: bool,
    #[clap(short)]
    scheduling_language_only: bool,

    #[clap(long)]
    parse: bool,
    #[clap(long)]
    typeelab: bool,
    #[clap(long)]
    vil: bool,

    #[clap(long, takes_value = true)]
    output: Option<String>,

    // By default it just prints, for now
    #[clap(long)]
    run: bool,
}

fn main() -> Result<(), error::Error> {
    let args = Arguments::parse();
    compile_new_lang(args)
}

fn compile_new_lang(args: Arguments) -> Result<(), error::Error> {
    let ast = parse::parse_file(&args.filename)?;
    if args.parse {
        println!("{:#?}", ast);
    } else {
        unimplemented!("Only parse is implemented for now")
    }
    Ok(())
}

// fn explicate_and_execute(output: Option<String>, program: asm::Program) {
//     let version = &program.version;
//     assert_eq!((version.major, version.minor, version.detailed), (0, 0, 2));

//     let definition = caiman::assembly::lowering_pass::lower(program);
//     caiman::ir::validation::validate_program(&definition.program);
//     let mut codegen = caiman::rust_wgpu_backend::codegen::CodeGen::new(&definition.program);
//     codegen.set_print_codgen_debug_info(true);
//     let output_string = codegen.generate();
//     match output {
//         None => println!("{}", output_string),
//         Some(path_str) => {
//             // Copied from caiman/src/main.rs
//             let path = Path::new(&path_str);
//             let prefix = path.parent().unwrap();
//             std::fs::create_dir_all(prefix).unwrap();
//             std::fs::write(path, output_string).unwrap();
//         }
//     }
// }
