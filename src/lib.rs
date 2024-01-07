#![allow(warnings)]

use frontend::Definition;

extern crate core;

#[macro_use]
mod operations;
pub mod assembly;
mod id_generator;
pub mod ir;
pub mod stable_vec;
//mod ir_builders;
pub mod frontend;
mod rust_wgpu_backend;
mod scheduling_state;
mod shadergen;
mod type_system;

// TODO (stephen): unified CLI
pub fn explicate_and_execute(
    output: Option<String>,
    program: assembly::ast::Program,
    explicate_only: bool,
) {
    let version = &program.version;
    assert_eq!((version.major, version.minor, version.detailed), (0, 0, 2));

    let definition = assembly::lowering_pass::lower(program);
    if explicate_only {
        println!("{:#?}", definition);
        return;
    }
    match crate::type_system::check_program(&definition.program) {
        Ok(_) => (),
        Err(error) => panic!("Type checking failed:\n{}", error),
    }
    let mut codegen = rust_wgpu_backend::codegen::CodeGen::new(&definition.program);
    codegen.set_print_codgen_debug_info(true);
    let output_string = codegen.generate();
    match output {
        None => println!("{}", output_string),
        Some(path_str) => {
            // Copied from caiman/src/main.rs (by Mia)
            // Copied from Mia (by Stephen)
            let path = std::path::Path::new(&path_str);
            let prefix = path.parent().unwrap();
            std::fs::create_dir_all(prefix).unwrap();
            std::fs::write(path, output_string).unwrap();
        }
    }
}
