#![allow(warnings)]

extern crate core;

#[macro_use]
mod operations;
mod id_generator;
pub mod ir;
pub mod stable_vec;
mod assembly;
//mod ir_builders;
mod shadergen;
pub mod frontend;
mod rust_wgpu_backend;
mod scheduling_state;
mod type_system;
#[macro_use]
pub mod assembly_ast;
