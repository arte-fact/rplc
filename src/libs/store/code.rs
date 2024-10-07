use std::collections::BTreeMap;
use std::sync::Arc;

use tokio::sync::Mutex;

lazy_static! {
    static ref CODE: Arc<Mutex<BTreeMap<String, Vec<String>>>> = Arc::new(Mutex::new(BTreeMap::new()));
}

pub async fn store_code(key: String, value: Vec<String>) {
    let mut query_store = CODE.lock().await;
    query_store.insert(key, value);
}

pub async fn get_code(key: &str) -> Option<Vec<String>> {
    let query = CODE.lock().await;
    query.get(key).cloned()
}

