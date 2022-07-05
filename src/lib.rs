#![allow(warnings)]
#[macro_use]
mod operations;
mod id_generator;
mod arena;
mod ir;
mod ir_builders;
mod shadergen;
pub mod frontend;
mod rust_wgpu_backend;
mod dataflow;
mod transformations;
mod convert;
mod node_usage_analysis;