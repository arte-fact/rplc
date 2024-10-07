use std::sync::Arc;

use tokio::sync::Mutex;

use crate::libs::split_query::QuerySplit;

lazy_static! {
    static ref QUERY: Arc<Mutex<QuerySplit>> = Arc::new(Mutex::new(QuerySplit::default()));
}

pub async fn store_query(query: &QuerySplit) {
    let mut query_store = QUERY.lock().await;
    *query_store = query.clone();
}

pub async fn get_query() -> QuerySplit {
    let query = QUERY.lock().await;
    query.clone()
}
