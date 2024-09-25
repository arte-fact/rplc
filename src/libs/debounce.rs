use std::future::Future;
use std::io::Error;
use std::sync::Arc;
use tokio::sync::Mutex;


lazy_static! {
    static ref QUERY: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
}

async fn store_query(query: String) {
    let mut state = QUERY.lock().await;
    state.clear();
    state.push_str(&query);
}

async fn get_query() -> String {
    let state = QUERY.lock().await;
    state.clone()
}

pub async fn debounce<F, Fut>(query: String, func: F)
where
    F: FnOnce(String) -> Fut + Send + 'static,
    Fut: Future<Output = Result<(), Error>>,
{
    store_query(query).await;
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        let query = get_query().await;
        if (query).is_empty() || get_query().await == get_query().await {
            return;
        }
        func(query);
    });
}
