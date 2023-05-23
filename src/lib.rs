#![allow(warnings)]

extern crate core;

#[macro_use]
mod operations;
#[cfg(feature = "assembly")]
mod assembly;
mod id_generator;
mod ir;
mod stable_vec;
//mod ir_builders;
pub mod frontend;
mod rust_wgpu_backend;
mod scheduling_state;
mod shadergen;
mod type_system;
#[macro_use]
#[cfg(feature = "assembly")]
mod assembly_ast;
