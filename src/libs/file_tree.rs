use std::collections::BTreeMap;
use std::path::PathBuf;

use crossterm::style::Stylize;

#[derive(Default, Clone, Debug)]
pub struct Tree {
    pub children: BTreeMap<String, Tree>,
    pub path: PathBuf,
}

impl Tree {
    // pub fn from_string_vec(paths: &Vec<String>) -> Tree {
    //     let mut tree = Tree::default();
    //     for path in paths {
    //         let mut current = &mut tree;
    //         for component in path.split('/') {
    //             let name = component.to_string();
    //             current = current
    //                 .children
    //                 .entry(name)
    //                 .or_insert_with(|| Tree::default());
    //         }
    //     }
    //     tree
    // }
    pub fn from_path_vec(paths: &Vec<PathBuf>) -> Tree {
        let mut tree = Tree::default();
        for path in paths {
            let mut current = &mut tree;
            current.path = path.clone();
            for component in path.to_str().unwrap_or("").split('/') {
                let name = component.to_string();
                current = current.children.entry(name).or_insert_with(|| Tree {
                    children: BTreeMap::new(),
                    path: path.clone(),
                });
            }
        }
        tree
    }

    pub fn sorted_paths(&self) -> Vec<PathBuf> {
        let mut result = vec![];
        let mut children = self.children.clone();
        let mut children = children.iter_mut().collect::<Vec<_>>();
        children.sort_by(|(_, a), (_, b)| b.children.keys().len().cmp(&a.children.keys().len()));
        for (_, child) in children {
            if child.children.is_empty() {
                result.push(self.path.clone());
            } else {
                let child_paths = child.sorted_paths();
                result.extend(child_paths);
            }
        }
        result
    }

    pub fn files(&self) -> Vec<PathBuf> {
        let mut result = vec![];
        let mut children = self.children.clone();
        let mut children = children.iter_mut().collect::<Vec<_>>();
        children.sort_by(|(_, a), (_, b)| b.children.keys().len().cmp(&a.children.keys().len()));
        for (_, child) in children {
            if child.children.is_empty() {
                result.push(child.path.clone());
            } else {
                let child_files = child.files();
                result.extend(child_files);
            }
        }
        result
    }

    pub fn first_file_path(&self) -> Option<String> {
        let mut children = self.children.clone();
        let mut children = children.iter_mut().collect::<Vec<_>>();
        children.sort_by(|(_, a), (_, b)| b.children.keys().len().cmp(&a.children.keys().len()));
        for (name, child) in children {
            if child.children.is_empty() {
                return Some(name.clone());
            } else {
                let child_path = child.first_file_path();
                if child_path.is_some() {
                    let full_path = format!("{}/{}", name, child_path.unwrap());
                    return Some(full_path);
                }
            }
        }
        None
    }
}

pub fn display_tree(
    tree: &Tree,
    depth: usize,
    width: usize,
    selected: &Option<String>,
) -> Vec<String> {
    let mut result = vec![];
    let mut children = tree.children.clone();
    let mut children = children.iter_mut().collect::<Vec<_>>();
    children.sort_by(|(_, a), (_, b)| b.children.keys().len().cmp(&a.children.keys().len()));
    for (name, child) in children {
        let name = if let Some(selected) = selected {
            if name == selected.split('/').last().unwrap() {
                name.clone().on_black().to_string()
            } else {
                name.clone()
            }
        } else {
            name.clone()
        };
        let name = if child.children.is_empty() {
            name.clone()
        } else {
            format!("{}{}", name, "/")
        };
        result.push(
            format!("{}{}", "  ".repeat(depth), name)
                .chars()
                .take(width)
                .collect(),
        );
        result.extend(display_tree(child, depth + 1, width, selected));
    }
    result
}
