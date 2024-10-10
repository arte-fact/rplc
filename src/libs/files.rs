use std::path::PathBuf;
use tokio::fs::read_to_string;
use crate::libs::ignore::match_glob_and_ignore;

use super::split_query::QuerySplit;
use super::store::files::{clear_files, store_file};

pub async fn list_glob_files(glob_pattern: &str) -> Result<Vec<PathBuf>, std::io::Error> {
    match_glob_and_ignore(glob_pattern.to_string())
}

pub async fn store_glob_files(query: &QuerySplit) -> Result<Vec<PathBuf>, std::io::Error> {
    clear_files().await;
    let glob = match &query.glob {
        Some(glob) => glob,
        None => return Ok(vec![]),
    };
    match list_glob_files(glob).await {
        Ok(files) => {
            for file in files.iter() {
                if !file.is_file() {
                    continue;
                }
                let file_name = file.to_str().unwrap_or("Could not read file name");
                let content = match read_to_string(file).await {
                    Ok(content) => content,
                    Err(_) => continue,
                };
                store_file(file_name, content).await;
            }
            Ok(files)
        }
        Err(e) => Err(e),
    }
}
