use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

lazy_static! {
    static ref FILES: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
}

pub async fn store_file(key: String, value: String) {
    let mut state = FILES.lock().await;
    state.insert(key, value);
}

pub async fn get_file(key: &str) -> Option<String> {
    let state = FILES.lock().await;
    state.get(key).cloned()
}

pub async fn get_files_names() -> Vec<String> {
    let state = FILES.lock().await;
    state.keys().cloned().collect()
}

pub async fn clear_files() {
    let mut state = FILES.lock().await;
    state.clear();
}
