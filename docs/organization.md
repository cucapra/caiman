# Organization

There are a few projects within caiman (to be described eventually):
- (top level) caimanc
- caiman-ref
- caiman-rt
- caiman-spec
- caiman-test

## caimanc
This is the caiman compiler in both library and command line application forms. `src/frontend.rs` is the main entry point for everything. `src/main.rs` is a thin layer over the frontend module when used as a command line application.

The frontend module provides two functions: `compile_ron_definition`, for compiling the provided IR to rust code, and `explicate_ron_definition`, for just running explication on the provided IR and outputting the explicit IR. Both receive input as strings and output as strings. Except for `src/main.rs`, caimanc does no file access.

Both run the explicator, but the former also invokes the `generate` method of the `CodeGen` struct from the codegen module in `src/rust_wgpu_backend/codegen.rs`.

### caiman/rust_wgpu_backend/codegen
This module is responsible for converting explicit IR to a string of rust code. It calls out to (the confusingly named) `CodeGenerator` struct defined in `src/rust_wgpu_backend/code_generator.rs`.

### caiman/rust_wgpu_backend/code_generator
This module handles the mechanics of building rust/wgpu gadgets and outputting a string of rust code. It is supposed to be (mostly) independent of the provided IR so that code generation doesn't need to modify the IR first. It also isolates `codegen` from rust as a syntactic/textual concept.

### caiman/rust_wgpu_backend/code_writer
This is a boring utility for writing code to a string, with some helper methods for common patterns.

## caiman-ref

## caiman-rt

This is a thin runtime for caiman that contains boilerplate data structures used by code generated from the caiman compiler.

## caiman-spec

## caiman-test

This is a test suite for caiman that also serves as a demonstration of how to build an application that invokes the caiman compiler and uses a caiman pipeline.

caiman-test invokes the library form of caimanc (via the frontend module) from its `build.rs` during the build phase. This is is the only place where files get read/written in the compilation of a caiman pipeline.
