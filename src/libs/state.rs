use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

lazy_static! {
    static ref STATE: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
}

pub async fn store_file(key: String, value: String) {
    let mut state = STATE.lock().await;
    state.insert(format!("file_{}", key), value);
}

pub async fn get_file(key: &str) -> Option<String> {
    let state = STATE.lock().await;
    state.get(&format!("file_{}", key)).cloned()
}

pub async fn get_files_names() -> Vec<String> {
    let state = STATE.lock().await;
    state.keys()
        .filter(|x| x.starts_with("file_"))
        .cloned()
        .map(|x| x.replace("file_", ""))
        .collect()
}

pub async fn get_files_paths() -> Vec<PathBuf> {
    let state = STATE.lock().await;
    state.values().map(PathBuf::from).collect()
}

pub async fn clear_files() {
    let mut state = STATE.lock().await;
    state.clear();
}

pub async fn store_key_value(key: String, value: String) {
    let mut state = STATE.lock().await;
    state.insert(key, value);
}

pub async fn get_key_value(key: &str) -> Option<String> {
    let state = STATE.lock().await;
    state.get(key).cloned()
}
