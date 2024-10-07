use std::collections::BTreeMap;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::libs::ui::window::Window;

lazy_static! {
    static ref WINDOWS: Arc<Mutex<BTreeMap<String, Window>>> = Arc::new(Mutex::new(BTreeMap::new()));
}

pub async fn store_window(key: String, value: Window) {
    let mut query_store = WINDOWS.lock().await;
    query_store.insert(key, value);
}

pub async fn get_window(key: &str) -> Option<Window> {
    let query = WINDOWS.lock().await;
    query.get(key).cloned()
}


