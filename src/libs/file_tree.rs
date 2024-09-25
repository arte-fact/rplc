use std::collections::HashMap;

use super::terminal::print_at;

#[derive(Default)]
pub struct Tree {
    pub children: HashMap<String, Tree>,
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
    for (name, child) in &tree.children {
        let name = if child.children.is_empty() {
            name.to_string()
        } else {
            format!("{}{}", name, "/")
        };
        result.push(format!("{}{}", "  ".repeat(depth), name));
        result.extend(display_tree(child, depth + 1));
    }
    result
}

pub fn print_tree(x: usize, y: usize, tree: &Tree) -> Result<(), std::io::Error> {
    let lines = display_tree(tree, 0); // "Ctrl-↑/Crtl-↓ to select");
    for (index, line) in lines.iter().enumerate() {
        print_at(x as u16, (y + index) as u16, line)?;
    }
    Ok(())
}

pub fn display_files_tree(x: usize, y: usize, paths: Vec<String>) -> Result<(), std::io::Error> {
    let tree = tree_from_pah_vec(paths);
    print_tree(y, x, &tree)
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
