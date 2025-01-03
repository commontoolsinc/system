use ct_storage::{CtStorage, Key, MemoryStorage, Result};
use futures_util::TryStreamExt;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test;
#[cfg(target_arch = "wasm32")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_dedicated_worker);

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
async fn it_streams_key_ranges() -> Result<()> {
    let mut storage = CtStorage::<MemoryStorage>::open_memory()?;
    let keys = [
        Key::new("alice", "calendar", "list"),
        Key::new("alice", "calendar", "index"),
        Key::new("alice", "calendar", "data"),
        Key::new("alice", "app", "index"),
        Key::new("bob", "calendar", "list"),
        Key::new("bob", "calendar", "index"),
        Key::new("bob", "app", "index"),
    ];
    for (index, key) in keys.iter().enumerate() {
        storage
            .set(key.to_owned(), index.to_le_bytes().to_vec())
            .await?;
    }

    {
        let key = Key::new("alice", "calendar", "");
        let stream = storage.get_namespace_stream(&key).await;
        tokio::pin!(stream);
        let mut count = 0;
        while let Some(entry) = stream.try_next().await? {
            assert_eq!(entry.key.entity(), key.entity());
            assert_eq!(entry.key.ns(), key.ns());
            count += 1;
        }
        assert_eq!(count, 3);
    }

    {
        let key = Key::new("alice", "", "");
        let stream = storage.get_entity_stream(&key).await;
        tokio::pin!(stream);
        let mut count = 0;
        while let Some(entry) = stream.try_next().await? {
            assert_eq!(entry.key.entity(), key.entity());
            count += 1;
        }
        assert_eq!(count, 4);
    }
    Ok(())
}
