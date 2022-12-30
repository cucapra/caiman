use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::io::Write;

/*
 * PLAN:
 * Have a regular lalrpop file (nodeless_parser.lalrpop)
 * Then the build script makes a NEW lalrpop file (parser.lalrpop) which is that one
 * plus some new things
 * Then lalrpop only runs on the newer one to make parser.rs
 * I'd prefer old one be called parser.lalrpop and new one be like
 * auto_parser or whatever BUT most of all i want generated file to be called parser.rs
 * which is hopefully a lalrpop option
 */

/*fn build_lalrpop() -> Result<(), Box<dyn Error>>
{
    let mut config = lalrpop::Configuration::new();
    config.generate_in_source_tree();
    config.process_current_dir()
}*/

fn write_lalrpop_parser(f: &mut fs::File)
{
    write!(
        f,
        "
        // TODO: Actually put things here!!!!! :)
        ",
    )
    .unwrap();
}

fn build_lalrpop_parser(src_dir: &Path)
{
    let nodeless_parser = src_dir.join("parser.lalrpop");
    let generated_parser = src_dir.join("generated_parser.lalrpop");
    fs::copy(&nodeless_parser, &generated_parser).unwrap();
    let mut app_file = fs::OpenOptions::new().append(true).open(generated_parser).unwrap();
    write_lalrpop_parser(&mut app_file);
}

fn build_rust_parser(src_dir: &Path) -> Result<(), Box<dyn Error>>
{
    let /*mut*/ config = lalrpop::Configuration::new();
    config.process_file(src_dir.join("generated_parser.lalrpop"))
}

fn main() 
{ 
    let src_dir = Path::new("./src/value_language/");
    build_lalrpop_parser(&src_dir);
    build_rust_parser(&src_dir).unwrap(); 
    fs::rename(src_dir.join("generated_parser.rs"), src_dir.join("parser.rs")).unwrap();
}
