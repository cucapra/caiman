#![allow(warnings)]

extern crate core;

#[macro_use]
mod operations;
mod id_generator;
pub mod ir;
pub mod stable_vec;
pub mod assembly;
//mod ir_builders;
pub mod frontend;
pub mod rust_wgpu_backend;
mod scheduling_state;
mod shadergen;
mod type_system;
//pub mod assembly_context;
