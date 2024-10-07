use std::path::PathBuf;
use tokio::fs::read_to_string;
use super::split_query::QuerySplit;
use super::store::files::{clear_files, store_file};
use glob::glob;


async fn read_gitingore() -> Option<Vec<String>> {
    if let Ok(content) = read_to_string(".gitignore").await {
        let lines: Vec<String> = content.lines().map(|x| x.to_string()).collect();
        return Some(lines);
    } else {
        return None;
    }
}

pub async fn list_glob_files(glob_pattern: &str) -> Result<Vec<PathBuf>, std::io::Error> {
    match glob(glob_pattern) {
        Err(e) => {
            println!("Could not list files: {}", e);
            Ok(vec![])
        }
        Ok(files) => {
            let mut files: Vec<PathBuf> = files.filter_map(|x| x.ok()).collect();
            if let Some(gitignore) = read_gitingore().await {
                files = files
                    .into_iter()
                    .filter(|file| {
                        let file_name = file.to_str().unwrap_or("Could not read file name");
                        if file_name.contains(".git") {
                            return false;
                        }
                        !gitignore.iter().any(|line| {
                            line.starts_with("#") || ("/".to_string() + file_name).contains(line)
                        })
                    })
                    .collect();
            }
            files.sort_by(|a, b| a.is_dir().cmp(&b.is_dir()));
            Ok(files)
        }
    }
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
                    Err(e) => file_name.to_string() + &format!(": {}", e),
                };
                store_file(file_name, content).await;
            }
            Ok(files)
        }
        Err(e) => Err(e),
    }
}
