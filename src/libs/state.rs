use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

lazy_static! {
    static ref STATE: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
}

#[deprecated]
pub async fn get_file(key: &str) -> Option<String> {
    let state = STATE.lock().await;
    state.get(&format!("file_{}", key)).cloned()
}

pub async fn store_key_value(key: String, value: String) {
    let mut state = STATE.lock().await;
    state.insert(key, value);
}

pub async fn get_key_value(key: &str) -> Option<String> {
    let state = STATE.lock().await;
    state.get(key).cloned()
}
