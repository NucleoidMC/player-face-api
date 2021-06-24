use std::future::Future;
use std::hash::Hash;

use lru_cache::LruCache;
use tokio::sync::Mutex;

pub trait Key: Eq + Hash + Clone {}

pub trait Value: Send + Sync + Clone {}

impl<T: Eq + Hash + Clone> Key for T {}

impl<T: Send + Sync + Clone> Value for T {}

pub struct Cache<K: Key, V: Value> {
    inner: Mutex<LruCache<K, V>>,
}

impl<K: Key, V: Value> Cache<K, V> {
    pub fn new(capacity: usize) -> Cache<K, V> {
        Cache {
            inner: Mutex::new(LruCache::new(capacity))
        }
    }

    pub async fn clear(&self) {
        self.inner.lock().await.clear();
    }

    pub async fn try_get<'a, F, Fut, E>(&'a self, key: K, load: F) -> Result<V, E>
        where F: FnOnce(K) -> Fut,
              Fut: Future<Output = Result<V, E>> + 'a,
    {
        let mut cache = self.inner.lock().await;

        if let Some(value) = cache.get_mut(&key) {
            return Ok(value.clone());
        }

        let value = load(key.clone()).await?;
        cache.insert(key.clone(), value.clone());

        Ok(value)
    }
}
