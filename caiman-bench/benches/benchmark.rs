use criterion::{black_box, criterion_group, criterion_main, Criterion};
mod basic_bench;
mod matmul_bench;
use basic_bench::{wgpu_basic_bench_1};
use matmul_bench::{wgpu_matmul_bench_1};

criterion_group!(benches, wgpu_basic_bench_1, wgpu_matmul_bench_1);
criterion_main!(benches);
