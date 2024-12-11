#![cfg(not(target_arch = "wasm32"))]

use rand::{thread_rng, Rng};
use ranked_prolly_tree::{BincodeEncoder, NodeStorage, Result, Tree};

fn random() -> Vec<u8> {
    let mut buffer = [0u8; 32];
    thread_rng().fill(&mut buffer[..]);
    buffer.to_vec()
}

#[tokio::test]
async fn file_system_storage() -> Result<()> {
    use ranked_prolly_tree::FileSystemStore;
    let root_dir = tempfile::TempDir::new()?;
    let storage = NodeStorage::new(
        BincodeEncoder::default(),
        FileSystemStore::new(root_dir.path()).await?,
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
