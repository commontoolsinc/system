use ranked_prolly_tree::{
    BasicEncoder, EphemeralStorage, NodeStorage, Result, SyncMemoryStore, TrackingStore, Tree,
};
use std::collections::BTreeMap;

fn bytes(s: &str) -> Vec<u8> {
    String::from(s).into_bytes()
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test;
#[cfg(target_arch = "wasm32")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_dedicated_worker);

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
async fn basic_set_and_get() -> Result<()> {
    let storage = EphemeralStorage::default();
    let mut tree = Tree::from(storage.clone());

    tree.set(bytes("foo1"), bytes("bar1")).await?;
    tree.set(bytes("foo2"), bytes("bar2")).await?;
    tree.set(bytes("foo3"), bytes("bar3")).await?;

    assert_eq!(tree.get(&bytes("bar")).await?, None);
    assert_eq!(tree.get(&bytes("foo1")).await?, Some(bytes("bar1")));
    assert_eq!(tree.get(&bytes("foo2")).await?, Some(bytes("bar2")));
    assert_eq!(tree.get(&bytes("foo3")).await?, Some(bytes("bar3")));

    let mut inverse_tree = Tree::<32, _>::new(storage);
    inverse_tree.set(bytes("foo3"), bytes("bar3")).await?;
    inverse_tree.set(bytes("foo2"), bytes("bar2")).await?;
    inverse_tree.set(bytes("foo1"), bytes("bar1")).await?;

    assert_eq!(
        tree.hash(),
        inverse_tree.hash(),
        "alternate insertion order results in same hash"
    );
    Ok(())
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
async fn create_tree_from_set() -> Result<()> {
    let iter_storage = EphemeralStorage::default();
    let set_storage = EphemeralStorage::default();
    let mut iter_tree = Tree::from(iter_storage);
    let mut set = BTreeMap::default();
    for i in 0..=255 {
        let key = vec![i];
        let value = vec![255 - i];
        set.insert(key.clone(), value.clone());
        iter_tree.set(key, value).await?;
    }
    let set_tree = Tree::<64, _>::from_set(set, set_storage).await?;

    for i in 0..=255 {
        let key = vec![i];
        let value = vec![255 - i];
        assert_eq!(set_tree.get(&key).await?, Some(value.clone()));
        assert_eq!(iter_tree.get(&key).await?, Some(value));
    }

    assert!(iter_tree.hash().is_some());
    assert_eq!(
        iter_tree.hash(),
        set_tree.hash(),
        "arrives at same root hash"
    );
    Ok(())
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
async fn larger_random_tree() -> Result<()> {
    use rand::{thread_rng, Rng};

    fn random() -> Vec<u8> {
        let mut buffer = [0u8; 32];
        thread_rng().fill(&mut buffer[..]);
        buffer.to_vec()
    }

    let mut ledger = vec![];
    let storage = EphemeralStorage::default();
    let mut tree = Tree::from(storage);
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

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
async fn restores_tree_from_hash() -> Result<()> {
    let storage = NodeStorage::new(BasicEncoder::default(), SyncMemoryStore::default());
    let mut tree = Tree::from(storage.clone());

    tree.set(bytes("foo1"), bytes("bar1")).await?;
    tree.set(bytes("foo2"), bytes("bar2")).await?;
    tree.set(bytes("foo3"), bytes("bar3")).await?;

    let root_hash = tree.hash().unwrap().to_owned();

    let tree = Tree::<32, _>::from_hash(&root_hash, storage).await?;
    assert_eq!(tree.get(&bytes("foo1")).await?, Some(bytes("bar1")));
    assert_eq!(tree.get(&bytes("foo2")).await?, Some(bytes("bar2")));
    assert_eq!(tree.get(&bytes("foo3")).await?, Some(bytes("bar3")));

    Ok(())
}

#[cfg(feature = "lru")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
async fn lru_store_caches() -> Result<()> {
    use ranked_prolly_tree::LruStore;
    let inner = SyncMemoryStore::default();
    let root_hash = {
        let storage = NodeStorage::new(BasicEncoder::default(), inner.clone());
        let mut set = BTreeMap::default();
        for i in 0..1024u32 {
            let key = i.to_be_bytes().to_vec();
            let value = <[u8; 32] as From<blake3::Hash>>::from(blake3::hash(&key)).to_vec();
            set.insert(key, value);
        }
        let tree = Tree::<32, _>::from_set(set, storage.clone()).await?;
        tree.hash().unwrap().to_owned()
    };

    let tracking = TrackingStore::new(inner);
    let lru = LruStore::new(tracking.clone(), 10)?;
    let mut tree =
        Tree::<32, _>::from_hash(&root_hash, NodeStorage::new(BasicEncoder::default(), lru))
            .await?;
    assert_eq!(tracking.writes()?, 0);
    assert_eq!(tracking.reads()?, 1); // read root hash

    let key = 1023u32.to_be_bytes().to_vec();
    let _ = tree.get(&key).await?;
    assert_eq!(tracking.writes()?, 0);
    assert_eq!(tracking.reads()?, 3);

    let _ = tree.get(&key).await?;
    assert_eq!(tracking.writes()?, 0);
    assert_eq!(tracking.reads()?, 3); // reads cached

    let _ = tree.set(key.to_vec(), vec![1]).await?;
    assert_eq!(tracking.writes()?, 3); // 3 writes on insertion
    assert_eq!(tracking.reads()?, 3); // reads cached

    Ok(())
}
