use futures_util::TryStreamExt;
use ranked_prolly_tree::{EphemeralStorage, Result, Tree};
use std::collections::BTreeMap;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test;
#[cfg(target_arch = "wasm32")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_dedicated_worker);

async fn create_test_tree<const P: u8>(
    size: u32,
) -> Result<Tree<P, EphemeralStorage<Vec<u8>, Vec<u8>>, Vec<u8>>> {
    let storage = EphemeralStorage::default();
    let mut set = BTreeMap::default();
    for i in 0..size {
        let key = i.to_be_bytes().to_vec();
        let value = <[u8; 32] as From<blake3::Hash>>::from(blake3::hash(&key)).to_vec();
        set.insert(key, value);
    }
    Tree::<P, _>::from_set(set, storage).await
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
async fn gets_full_range() -> Result<()> {
    let tree = create_test_tree::<32>(1024).await?;
    let stream = tree.stream().await;
    tokio::pin!(stream);
    let mut i = 0u32;
    while let Some(_) = stream.try_next().await? {
        i += 1;
    }
    assert_eq!(i, 1024, "full range yields all nodes");
    Ok(())
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
async fn stream_range_on_empty_trees() -> Result<()> {
    let storage = EphemeralStorage::default();
    let empty = Tree::from(storage);

    let stream = empty.stream_range(..).await;
    tokio::pin!(stream);
    assert!(stream.try_next().await?.is_none());

    let stream = empty.stream().await;
    tokio::pin!(stream);
    assert!(stream.try_next().await?.is_none());

    Ok(())
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
async fn gets_range() -> Result<()> {
    let tree = create_test_tree::<32>(1024).await?;

    const OFFSET: u32 = 2;
    const MAX: u32 = 10;

    let start = OFFSET.to_be_bytes().to_vec();
    let end = MAX.to_be_bytes().to_vec();
    let stream = tree.stream_range(start..end).await;
    tokio::pin!(stream);
    let mut i = 0u32;
    while let Some(entry) = stream.try_next().await? {
        assert_eq!(entry.key, (i + OFFSET).to_be_bytes().to_vec());
        assert_eq!(
            entry.value,
            <[u8; 32] as From<blake3::Hash>>::from(blake3::hash(&entry.key)).to_vec()
        );
        i += 1;
    }
    assert_eq!(i, MAX - OFFSET);
    Ok(())
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
async fn request_out_of_range() -> Result<()> {
    let tree = create_test_tree::<32>(1024).await?;
    let start = 1_000_000u32.to_be_bytes().to_vec();
    let stream = tree.stream_range(start..).await;
    tokio::pin!(stream);
    assert!(
        stream.try_next().await?.is_none(),
        "start range out of tree range yields no items"
    );

    let storage = EphemeralStorage::default();
    let mut tree = Tree::<32, _>::new(storage);
    tree.set(10u32.to_be_bytes().to_vec(), vec![1]).await?;
    let start = 0u32.to_be_bytes().to_vec();
    let end = 5u32.to_be_bytes().to_vec();
    let stream = tree.stream_range(start..end).await;
    tokio::pin!(stream);
    assert!(
        stream.try_next().await?.is_none(),
        "end range out of tree range yields no items"
    );

    Ok(())
}
