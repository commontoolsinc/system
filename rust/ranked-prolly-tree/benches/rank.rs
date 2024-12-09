#![cfg(not(target_arch = "wasm32"))]

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use rand::{thread_rng, Rng};
use ranked_prolly_tree::compute_rank;

fn random() -> Vec<u8> {
    let mut buffer = [0u8; 32];
    thread_rng().fill(&mut buffer[..]);
    buffer.to_vec()
}

pub fn run_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("rank");
    for factor in [16, 32, 64, 128u32] {
        group.bench_with_input(BenchmarkId::from_parameter(factor), &factor, |b, factor| {
            b.iter_batched(
                || random(),
                |hash| compute_rank(&hash, *factor),
                BatchSize::SmallInput,
            )
        });
    }
}
criterion_group!(benches, run_benchmark);
criterion_main!(benches);
