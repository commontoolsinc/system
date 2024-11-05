#![cfg(not(target_arch = "wasm32"))]

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};
use rand::{thread_rng, Rng};
use ranked_prolly_tree::{EphemeralStorage, Result, Tree};
use std::collections::BTreeMap;

const TREE_SIZES: [usize; 3] = [10_000, 100_000, 1_000_000];

async fn setup_tree<const P: u8>(count: u32) -> Result<(Tree<P, EphemeralStorage>, Vec<Vec<u8>>)> {
    let mut set = BTreeMap::default();
    for _ in 0..count {
        set.insert(random(), random());
    }
    let storage = EphemeralStorage::default();
    let tree = Tree::from_set(set.clone(), storage).await?;
    let keys = set.into_keys().collect();
    Ok((tree, keys))
}

fn random_get<T: Clone>(input: &Vec<T>) -> Option<T> {
    input
        .get(thread_rng().gen_range(0..input.len()) as usize)
        .map(|v| v.to_owned())
}

fn random() -> Vec<u8> {
    let mut buffer = [0u8; 32];
    thread_rng().fill(&mut buffer[..]);
    buffer.to_vec()
}

pub fn run_benchmark(c: &mut Criterion) {
    let async_runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    {
        let mut group_get = c.benchmark_group("get");
        for size in TREE_SIZES {
            let (tree, keys) =
                async_runtime.block_on(async { setup_tree::<32>(size as u32).await.unwrap() });
            group_get.throughput(Throughput::Elements(size as u64));
            group_get.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
                b.to_async(&async_runtime).iter_batched(
                    || (tree.clone(), random_get(&keys).unwrap()),
                    |(tree, key)| async move { tree.get(&key).await.unwrap().unwrap() },
                    BatchSize::SmallInput,
                )
            });
        }
    }
    {
        let mut group_insert = c.benchmark_group("insert");
        for size in TREE_SIZES {
            let (tree, _) =
                async_runtime.block_on(async { setup_tree::<32>(size as u32).await.unwrap() });
            group_insert.throughput(Throughput::Elements(size as u64));
            group_insert.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
                b.to_async(&async_runtime).iter_batched(
                    || (tree.clone(), random()),
                    |(mut tree, key)| async move { tree.set(key, vec![123u8]).await.unwrap() },
                    BatchSize::SmallInput,
                )
            });
        }
    }
    {
        let mut group_update = c.benchmark_group("update");
        for size in TREE_SIZES {
            let (tree, keys) =
                async_runtime.block_on(async { setup_tree::<32>(size as u32).await.unwrap() });
            group_update.throughput(Throughput::Elements(size as u64));
            group_update.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
                b.to_async(&async_runtime).iter_batched(
                    || (tree.clone(), random_get(&keys).unwrap()),
                    |(mut tree, key)| async move { tree.set(key, vec![123u8]).await.unwrap() },
                    BatchSize::SmallInput,
                )
            });
        }
    }
}

criterion_group!(benches, run_benchmark);
criterion_main!(benches);
