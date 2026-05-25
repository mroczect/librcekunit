use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct TokenCache {
    store: RwLock<HashMap<String, String>>,
}

impl TokenCache {
    pub fn new() -> Self {
        Self {
            store: RwLock::new(HashMap::new()),
        }
    }

    pub async fn set(&self, key: &str, value: &str) {
        self.store
            .write()
            .await
            .insert(key.to_string(), value.to_string());
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        self.store.read().await.get(key).cloned()
    }

    pub async fn remove(&self, key: &str) {
        self.store.write().await.remove(key);
    }

    pub async fn clear(&self) {
        self.store.write().await.clear();
    }
}
