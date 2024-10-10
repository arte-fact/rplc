use crate::libs::store::files::get_file;
use crate::libs::store::tree::get_selected;
use crate::libs::terminal::{screen_height, screen_width};
use crate::libs::ui::window::{create_and_store_window, WindowAttr};


pub async fn code_window() -> Result<(), std::io::Error> {
    let path = if let Some(path) = get_selected().await {
        path
    } else {
        return Ok(());
    };

    let path = path.to_str().unwrap_or_else(|| "").to_string();

    let left = 40;
    let top = 4;
    let height = screen_height() - top - 1;
    let width = screen_width() - left;

    let content = get_file(&path).await.unwrap_or_else(|| "Unable to read file.".to_string());
    let file_ext = match path.split('.').last() {
        Some(ext) => Some(ext.to_string()),
        None => None,
    };

    create_and_store_window(
        "result".to_string(),
        vec![
            WindowAttr::Title(Some(path.to_string())),
            WindowAttr::Content(content.lines().map(|x| x.to_string()).collect()),
            WindowAttr::Footer(None),
            WindowAttr::Top(top as usize),
            WindowAttr::Left(left as usize),
            WindowAttr::Width(width as usize),
            WindowAttr::Height(Some(height as usize)),
            WindowAttr::Decorated(true),
            WindowAttr::Scrollable(true),
            WindowAttr::Scroll(0),
            WindowAttr::Highlight(file_ext),
            WindowAttr::DecorationStyle(Default::default()),
        ],
    )
    .await?
    .draw()
}
