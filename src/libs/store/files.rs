use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;

lazy_static! {
    static ref FILES: Arc<Mutex<FileHashMap>> = Arc::new(Mutex::new(HashMap::new()));
}

pub type FileHashMap = HashMap<String, String>;

pub async fn store_file(key: &str, value: String) {
    let mut files = FILES.lock().await;
    files.insert(key.to_string(), value);
}

pub async fn get_file(key: &str) -> Option<String> {
    let files = FILES.lock().await;
    files.get(key).cloned()
}

pub async fn get_files() -> FileHashMap {
    let files = FILES.lock().await;
    files.clone()
}

pub async fn clear_files() {
    let mut files = FILES.lock().await;
    files.clear();
}
