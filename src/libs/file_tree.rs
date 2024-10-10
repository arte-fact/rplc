use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Default, Clone, Debug)]
pub struct Tree {
    pub children: BTreeMap<String, Tree>,
    pub path: PathBuf,
}

impl Tree {
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
}
