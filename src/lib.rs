#![allow(warnings)]

extern crate core;

#[macro_use]
mod operations;
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
