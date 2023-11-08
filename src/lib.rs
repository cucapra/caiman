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
