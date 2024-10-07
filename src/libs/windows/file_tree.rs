use std::path::PathBuf;

use crossterm::style::Stylize;
use crate::libs::file_tree::Tree;
use crate::libs::store::tree::{get_selected, get_tree, store_selected};
use crate::libs::terminal::screen_height;
use crate::libs::ui::window::{create_and_store_window, WindowAttr};

pub async fn select_file_by(delta: isize) -> Result<(), std::io::Error> {
    let selected_file = get_selected().await;
    let tree = get_tree().await;

    if selected_file.is_none() || tree.is_none() {
        return Ok(());
    }

    let selected_file = selected_file.unwrap();
    let files = tree.unwrap().files();
    
    let current_index = files.iter().position(|x| x == &selected_file).unwrap();
    let next_index = (current_index as isize + delta) as usize;

    if next_index < files.len() {
        let next_file = &files[next_index];
        store_selected(Some(next_file.clone())).await;
    }

    Ok(())
}

pub fn display_tree(tree: &Tree, depth: usize, width: usize, selected: &Option<PathBuf>) -> Vec<String> {
    let mut result = vec![];
    let mut children = tree.children.clone();
    let mut children = children.iter_mut().collect::<Vec<_>>();
    children.sort_by(|(_, a), (_, b)| b.children.keys().len().cmp(&a.children.keys().len()));

    for (name, child) in children {
        let mut name = name.clone();
        if !child.children.is_empty() {
            name = format!("{}{}", name, "/")
        };
        let mut line: String = format!("{}{}", "  ".repeat(depth), name).chars().take(width).collect();
        let selected = selected.clone();
        if Some(child.path.clone()) == selected {
            line = line.on_black().to_string();
        }
        result.push(line);
        result.extend(display_tree(child, depth + 1, width, &selected));
    }
    result
}

pub async fn file_tree_window() -> Result<(), std::io::Error> {
    let tree = if let Some(tree) = get_tree().await {
        tree
    } else {
        return Ok(());
    };

    let top = 4;
    let left = 0;
    let width = 40;
    let height = screen_height() as usize - top - 1;

    let selected = get_selected().await;
    let lines = display_tree(&tree, 0, width - 2, &selected);

    create_and_store_window(
        "file-tree".to_string(),
        vec![
            WindowAttr::Top(top),
            WindowAttr::Left(left),
            WindowAttr::Width(width),
            WindowAttr::Height(Some(height)),
            WindowAttr::Title(Some("Files".to_string())),
            WindowAttr::Content(lines),
            WindowAttr::Scrollable(false),
            WindowAttr::Scroll(0),
            WindowAttr::Decorated(true),
            WindowAttr::Highlight(None),
        ],
    ).await?.draw()?;
    Ok(())
}
