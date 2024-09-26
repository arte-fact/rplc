use std::collections::BTreeMap;

use super::terminal::screen_height;
use super::ui::window::{create_and_store_window, WindowAttr};

#[derive(Default, Clone)]
pub struct Tree {
    pub children: BTreeMap<String, Tree>,
}

pub fn tree_from_pah_vec(paths: Vec<String>) -> Tree {
    let mut tree = Tree::default();
    for path in paths {
        let mut current = &mut tree;
        for component in path.split('/') {
            let name = component.to_string();
            current = current
                .children
                .entry(name)
                .or_insert_with(|| Tree::default());
        }
    }
    tree
}

pub fn display_tree(tree: &Tree, depth: usize) -> Vec<String> {
    let mut result = vec![];
    let mut children = tree.children.clone();
    let mut children = children.iter_mut().collect::<Vec<_>>();
    children.sort_by(|(_, a), (_, b)| b.children.keys().len().cmp(&a.children.keys().len()));
    for (name, child) in children {
        let name = if child.children.is_empty() {
            name.clone()
        } else {
            format!("{}{}", name, "/")
        };
        result.push(format!("{}{}", "  ".repeat(depth), name));
        result.extend(display_tree(child, depth + 1));
    }
    result
}

pub async fn print_tree(top: usize, left: usize, tree: &Tree) -> Result<(), std::io::Error> {
    let lines = display_tree(tree, 0); // "Ctrl-↑/Crtl-↓ to select");
    create_and_store_window(
        "file-tree".to_string(),
        vec![
            WindowAttr::Top(top),
            WindowAttr::Left(left),
            WindowAttr::Width(40),
            WindowAttr::Height(Some(screen_height() as usize - top)),
            WindowAttr::Title("Files".to_string()),
            WindowAttr::Content(lines),
            WindowAttr::Scrollable(false),
            WindowAttr::Scroll(0),
            WindowAttr::Decorated(false),
            WindowAttr::Highlight(None),
        ],
    ).await?.draw()?;
    Ok(())
}

pub async fn display_files_tree(top: usize, left: usize, paths: Vec<String>) -> Result<(), std::io::Error> {
    let tree = tree_from_pah_vec(paths);
    print_tree(top, left, &tree).await
}

#[test]
fn test_tree_from_path_vec() {
    let paths = vec![
        "src/file_tree.rs".to_string(),
        "src/syntax_highlight.rs".to_string(),
        "src/terminal.rs".to_string(),
        "src/state.rs".to_string(),
        "src/mod.rs".to_string(),
    ];
    let tree = tree_from_pah_vec(paths);
    let expected = r#"src/
  file_tree.rs
  mod.rs
  state.rs
  syntax_highlight.rs
  terminal.rs"#;
    let result = display_tree(&tree, 0).join("\n");
    assert_eq!(result, expected);
}
