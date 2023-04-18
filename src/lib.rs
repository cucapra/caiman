#![allow(warnings)]

extern crate core;

#[macro_use]
mod operations;
mod id_generator;
mod stable_vec;
mod ir;
//mod assembly;
//mod ir_builders;
mod shadergen;
pub mod frontend;
mod rust_wgpu_backend;
mod scheduling_state;
mod type_system;
//#[macro_use]
//mod assembly_ast;
