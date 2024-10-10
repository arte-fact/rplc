use super::decorate_file_content::happend_changes_in_file;
use super::state::get_file;
use super::syntax_highlight::highlight_line;


pub async fn display_changes_in_file(
    search: Option<String>,
    substitute: Option<String>,
    path: &str,
) -> Result<(Vec<String>, usize), std::io::Error> {
    let content = get_file(path).await.unwrap_or("No content.".to_string());
    let extension = path.split('.').last().unwrap_or("txt");
    let content = content
        .lines()
        .map(|x| highlight_line(x, extension).unwrap_or(x.to_string()))
        .collect();

    let query = match search {
        Some(query) => query,
        None => return Ok((content, 0)),
    };

    let substitute = match substitute {
        Some(substitute) => substitute,
        None => query.clone(),
    };

    let (result, changes): _ = happend_changes_in_file(content, &query, &substitute);

    Ok((result, changes))
}

pub async fn replace_in_file(
    query: String,
    substitute: String,
    path: &str,
) -> Result<(), std::io::Error> {
    let content = match get_file(path).await {
        Some(content) => content,
        None => {
            return Ok(());
        }
    };
    if !content.contains(&query) {
        return Ok(());
    }
    let new_content = content.replace(&query, &substitute);

    std::fs::write(path, new_content)?;
    Ok(())
}
