#![allow(warnings)]
#[macro_use]
mod operations;
mod id_generator;
mod collections;
mod ir;
mod ir_builders;
mod shadergen;
pub mod frontend;
mod rust_wgpu_backend;
mod dataflow;
mod value;
pub mod optimizations;
mod node_usage_analysis;