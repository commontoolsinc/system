#![cfg(target_arch = "wasm32")]

use rand::{thread_rng, Rng};
use ranked_prolly_tree::{BincodeEncoder, NodeStorage, Result, Tree};

fn random() -> Vec<u8> {
    let mut buffer = [0u8; 32];
    thread_rng().fill(&mut buffer[..]);
    buffer.to_vec()
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test;
#[cfg(target_arch = "wasm32")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_dedicated_worker);

#[wasm_bindgen_test]
async fn indexed_db_storage() -> Result<()> {
    use ranked_prolly_tree::IndexedDbStore;
    let storage = NodeStorage::new(
        BincodeEncoder::default(),
        IndexedDbStore::new("db_name", "store_name").await?,
    );
    let mut tree = Tree::<32, _>::new(storage.clone());

    let mut ledger = vec![];
    for _ in 1..1024 {
        let key_value = (random(), random());
        ledger.push(key_value.clone());
        tree.set(key_value.0, key_value.1).await?;
    }

    for entry in ledger {
        assert_eq!(tree.get(&entry.0).await?, Some(entry.1));
    }
    Ok(())
}
