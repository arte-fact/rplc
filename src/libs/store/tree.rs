use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::libs::file_tree::Tree;

lazy_static! {
    static ref TREES: Arc<Mutex<Option<Tree>>> = Arc::new(Mutex::new(None));
    static ref SELECTED: Arc<Mutex<Option<PathBuf>>> = Arc::new(Mutex::new(None));
}

pub async fn store_tree(t: &Tree) {
    let mut tree = TREES.lock().await;
    *tree = Some(t.clone());
}

pub async fn get_tree() -> Option<Tree> {
    let trees = TREES.lock().await;
    trees.clone()
}

pub async fn store_selected(selected: Option<PathBuf>) {
    let mut selected_store = SELECTED.lock().await;
    *selected_store = selected;
}

pub async fn get_selected() -> Option<PathBuf> {
    let selected = SELECTED.lock().await;
    selected.clone()
}
