use std::path::PathBuf;
use tokio::fs::read_to_string;
use super::state::{clear_files, store_file};
use glob::glob;


pub fn list_glob_files(glob_pattern: &str) -> Result<Vec<PathBuf>, std::io::Error> {
    match glob(glob_pattern) {
        Err(e) => {
            println!("Could not list files: {}", e);
            Ok(vec![])
        }
        Ok(files) => {
            let files: Vec<PathBuf> = files.filter_map(|x| x.ok()).collect();
            Ok(files)
        }
    }
}

pub async fn store_glob_files(glob_pattern: &str) -> Result<(), std::io::Error> {
    clear_files().await;
    match glob(glob_pattern) {
        Err(e) => {
            println!("Could not list files: {}", e);
            return Ok(());
        }
        Ok(files) => {
            let files: Vec<PathBuf> = files.filter_map(|x| x.ok()).collect();
            for file in files.iter() {
                if !file.is_file() {
                    continue;
                }
                let file_name = file.to_str().unwrap_or("Could not read file name");
                let content = match read_to_string(file).await {
                    Ok(content) => content,
                    Err(e) => file_name.to_string() + &format!(": {}", e),
                };
                store_file(file_name.to_string(), content).await;
            }
        }
    }
    Ok(())
}
