//! # Benchmarks
//!
//! These benchmarks attempt to align with Okra's benchmarks: <https://github.com/canvasxyz/okra/blob/main/BENCHMARKS.md>
//! Each run tests both tree operations as well as operations on the underlying storage,
//! due to the underlying storage operations dominating execution time on the tree.
//!
//! Run Memory:
//! `cargo run --release --example benchmark`
//! Run IndexedDB:
//! `WASM_BINDGEN_TEST_TIMEOUT=300 cargo run --release --example benchmark --target wasm32-unknown-unknown`

use ct_tracing::ct_tracing;
use futures_util::TryStreamExt;
use rand::{thread_rng, Rng};
use ranked_prolly_tree::{Result, Storage, Tree};
use std::{collections::BTreeMap, io::Write};
use web_time::Instant;

#[cfg(not(target_arch = "wasm32"))]
use ranked_prolly_tree::EphemeralStorage;
#[cfg(target_arch = "wasm32")]
use ranked_prolly_tree::{BincodeEncoder, IndexedDbStore, LruStore, NodeStorage};

const BRANCHING_FACTOR: u8 = 64;
const SECONDS_TO_MILLISECONDS: f64 = 1000.0;

const TESTS: [(&'static str, u32); 3] = [
    ("1k entries", 1_000),
    ("50k entries", 50_000),
    ("1m entries", 1_000_000),
];

#[cfg(target_arch = "wasm32")]
macro_rules! render_out {
    ($($v:expr),+) => { tracing::info!($($v,)*) }
}
#[cfg(not(target_arch = "wasm32"))]
macro_rules! render_out {
    ($($v:expr),+) => { println!($($v,)*) }
}

#[ct_tracing]
async fn main_impl() -> Result<()> {
    let storage = create_storage().await?;

    for (name, tree_size) in TESTS {
        let mut writer = std::io::Cursor::new(vec![]);
        Context::exec(name, tree_size, storage.clone(), &mut writer).await?;
        render_out!("\n{}", String::from_utf8(writer.into_inner())?);
    }

    Ok(())
}

struct Context<S: Storage> {
    tree_size: u32,
    tree: Tree<BRANCHING_FACTOR, S>,
}

impl<S> Context<S>
where
    S: Storage,
{
    async fn new(tree_size: u32, storage: S) -> Result<Self> {
        let tree = Self::create_tree(tree_size, storage).await?;
        Ok(Self { tree, tree_size })
    }

    async fn exec<W: Write>(name: &str, tree_size: u32, storage: S, writer: &mut W) -> Result<()> {
        let mut printer = Printer::new();
        let mut ctx = Self::new(tree_size, storage).await?;
        ctx.get_random_entries("get random 1 entry", 100, 1, &mut printer)
            .await?;
        ctx.get_random_entries("get random 100 entries", 100, 100, &mut printer)
            .await?;
        ctx.iterate_entries(100, &mut printer).await?;
        ctx.set_random_entries("set random 1 entry", 100, 1, &mut printer)
            .await?;
        ctx.set_random_entries("set random 100 entries", 100, 100, &mut printer)
            .await?;
        ctx.set_random_entries("set random 1k entries", 10, 1_000, &mut printer)
            .await?;
        //ctx.set_random_entries("set random 50k entries", 10, 50_000, &mut printer)
        //    .await?;

        printer.print(name, writer)?;
        Ok(())
    }

    async fn create_tree(size: u32, storage: S) -> Result<Tree<BRANCHING_FACTOR, S>> {
        let mut set = BTreeMap::default();
        for i in 0..size {
            let key = i.to_be_bytes().to_vec();
            let value = <[u8; 32] as From<blake3::Hash>>::from(blake3::hash(&key)).to_vec();
            set.insert(key, value);
        }
        let tree = Tree::from_set(set, storage).await?;
        Ok(tree)
    }

    async fn iterate_entries(&self, iterations: usize, printer: &mut Printer) -> Result<()> {
        let mut runtimes = vec![];
        let mut ops = 0usize;
        for _ in 0..iterations {
            ops += self.tree_size as usize;

            let now = Instant::now();
            let stream = self.tree.stream().await;
            tokio::pin!(stream);
            let mut count = 0;
            while let Some(_entry) = stream.try_next().await? {
                count += 1;
            }
            assert_eq!(count, self.tree_size);
            runtimes.push(now.elapsed().as_secs_f64() * SECONDS_TO_MILLISECONDS);
        }
        printer.push_entry("iterate over all entries", runtimes, ops);
        Ok(())
    }

    async fn get_random_entries(
        &self,
        name: &str,
        iterations: usize,
        batch_size: usize,
        printer: &mut Printer,
    ) -> Result<()> {
        let mut runtimes = vec![];
        let mut ops = 0;
        for _ in 0..iterations {
            ops += batch_size;

            let now = Instant::now();
            for _ in 0..batch_size {
                let key = self.gen_key();
                assert!(self.tree.get(&key).await?.is_some());
            }
            runtimes.push(now.elapsed().as_secs_f64() * SECONDS_TO_MILLISECONDS);
        }
        printer.push_entry(name, runtimes, ops);
        Ok(())
    }

    async fn set_random_entries(
        &mut self,
        name: &str,
        iterations: usize,
        batch_size: usize,
        printer: &mut Printer,
    ) -> Result<()> {
        let mut runtimes = vec![];
        let mut ops = 0;
        for _ in 0..iterations {
            ops += batch_size;

            let now = Instant::now();
            for _ in 0..batch_size {
                let key = self.gen_key();
                let value = <[u8; 32] as From<blake3::Hash>>::from(blake3::hash(&key)).to_vec();
                self.tree.set(key, value).await?;
            }
            runtimes.push(now.elapsed().as_secs_f64() * SECONDS_TO_MILLISECONDS);
        }
        printer.push_entry(name, runtimes, ops);
        Ok(())
    }

    fn gen_key(&self) -> Vec<u8> {
        thread_rng()
            .gen_range(0..self.tree_size)
            .to_be_bytes()
            .to_vec()
    }
}

struct Printer {
    entries: Vec<(String, Vec<f64>, usize)>,
}

impl Printer {
    fn new() -> Self {
        Self { entries: vec![] }
    }

    fn push_entry(&mut self, name: &str, runtimes: Vec<f64>, ops: usize) {
        self.entries.push((name.to_owned(), runtimes, ops));
    }

    fn print_header<W: Write>(name: &str, writer: &mut W) -> Result<()> {
        writeln!(writer, "### {}\n", name)?;
        writeln!(
            writer,
            "| {:<30} | {:>10} | {:>10} | {:>10} | {:>10} | {:>8} | {:>10} |",
            "", "iterations", "min (ms)", "max (ms)", "avg (ms)", "std", "ops / s",
        )?;
        writeln!(
            writer,
            "| {:-<30} | {:->10} | {:->10} | {:->10} | {:->10} | {:->8} | {:->10} |",
            ":", ":", ":", ":", ":", ":", ":"
        )?;
        Ok(())
    }

    fn print_row<W: Write>(row: (String, Vec<f64>, usize), writer: &mut W) -> Result<()> {
        let (name, runtimes, ops) = row;
        let runtimes_len = runtimes.len();
        let runtimes_len_f64 = runtimes_len as f64;
        let mut sum = 0f64;
        let mut min = f64::MAX;
        let mut max = f64::MIN;
        for t in runtimes.iter() {
            let t = *t as f64;
            sum += t;
            if t < min {
                min = t;
            }
            if t > max {
                max = t;
            }
        }
        let avg = sum / runtimes_len_f64;

        let mut sum_sq = 0f64;
        for t in runtimes {
            let t = t as f64;
            let delta = t - avg;
            sum_sq += delta * delta;
        }
        let std_dev = f64::sqrt(sum_sq / runtimes_len_f64);
        let ops_per_sec = ((ops * 1_000) as f64) / sum;

        writeln!(
            writer,
            "| {:<30} | {:>10} | {:>10.4} | {:>10.4} | {:>10.4} | {:>8.4} | {:>10.0} |",
            name, runtimes_len, min, max, avg, std_dev, ops_per_sec
        )?;
        Ok(())
    }

    fn print<W: Write>(self, name: &str, writer: &mut W) -> Result<()> {
        Self::print_header(name, writer)?;
        for entry in self.entries {
            Self::print_row(entry, writer)?;
        }
        writeln!(writer, "")?;
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
async fn create_storage() -> Result<EphemeralStorage> {
    Ok(EphemeralStorage::default())
}

#[cfg(target_arch = "wasm32")]
async fn create_storage() -> Result<NodeStorage<BincodeEncoder, LruStore<IndexedDbStore>>> {
    fn gen_name() -> String {
        let bytes = thread_rng().gen_range(0..u32::MAX).to_be_bytes();
        let hash = blake3::hash(&bytes);
        ranked_prolly_tree::HashDisplay::from(<[u8; 32] as From<blake3::Hash>>::from(hash).to_vec())
            .to_string()
    }

    let store = IndexedDbStore::new(&gen_name(), &gen_name()).await?;
    let store = LruStore::new(store, 10_000)?;
    Ok(NodeStorage::new(BincodeEncoder::default(), store))
}

/*
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
trait BenchStore {
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    async fn set(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()>;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<T> BenchStore for T
where
    T: BlockStore,
{
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.get_block(key).await
    }
    async fn set(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        self.set_block(key, value).await
    }
}

struct TreeBenchStore<const P: u8, S: Storage>(Tree<P, S>);

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<const P: u8, S> BenchStore for TreeBenchStore<P, S>
where
    S: Storage,
{
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.0.get(key).await
    }
    async fn set(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        self.0.set(key, value).await
    }
}
*/

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test;
#[cfg(target_arch = "wasm32")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_dedicated_worker);
#[cfg(target_arch = "wasm32")]
pub fn main() {}
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
async fn main_js() -> Result<()> {
    main_impl().await
}
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main(flavor = "multi_thread")]
pub async fn main() -> Result<()> {
    main_impl().await
}
