use crate::picker::is_markdown_path;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Node {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub children: Vec<Node>,
}

pub fn build(root: &Path) -> Node {
    let name = root
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| root.to_string_lossy().into_owned());
    let mut node = Node {
        path: root.to_path_buf(),
        name,
        is_dir: true,
        children: Vec::new(),
    };
    fill(&mut node, 0, 12);
    prune(&mut node);
    node
}

fn fill(node: &mut Node, depth: usize, max_depth: usize) {
    if depth >= max_depth {
        return;
    }
    let Ok(rd) = std::fs::read_dir(&node.path) else {
        return;
    };
    let mut dirs: Vec<Node> = Vec::new();
    let mut files: Vec<Node> = Vec::new();
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        // Skip noisy build/vcs caches but allow other dot-dirs (.claude,
        // .vscode, .github, etc.) so users can browse their config.
        if name == "node_modules" || name == "target" || name == ".git" {
            continue;
        }
        let p = e.path();
        if p.is_dir() {
            let mut child = Node {
                path: p,
                name,
                is_dir: true,
                children: Vec::new(),
            };
            fill(&mut child, depth + 1, max_depth);
            dirs.push(child);
        } else if is_markdown_path(&p) {
            files.push(Node {
                path: p,
                name,
                is_dir: false,
                children: Vec::new(),
            });
        }
    }
    dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    node.children = dirs;
    node.children.extend(files);
}

/// Remove dirs with no markdown descendants.
fn prune(node: &mut Node) -> bool {
    if !node.is_dir {
        return true;
    }
    node.children.retain_mut(|c| prune(c));
    !node.children.is_empty()
}

/// Flatten tree into visible rows respecting `expanded` set. Root not shown.
pub fn flatten<'a>(root: &'a Node, expanded: &HashSet<PathBuf>) -> Vec<Row<'a>> {
    let mut out = Vec::new();
    for child in &root.children {
        push(child, 0, expanded, &mut out);
    }
    out
}

fn push<'a>(node: &'a Node, depth: usize, expanded: &HashSet<PathBuf>, out: &mut Vec<Row<'a>>) {
    out.push(Row { node, depth });
    if node.is_dir && expanded.contains(&node.path) {
        for c in &node.children {
            push(c, depth + 1, expanded, out);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Row<'a> {
    pub node: &'a Node,
    pub depth: usize,
}

/// Set containing every ancestor path (within root) needed to reveal `target`.
pub fn ancestors_of(root: &Path, target: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut cur = target.parent();
    while let Some(p) = cur {
        out.push(p.to_path_buf());
        if p == root {
            break;
        }
        cur = p.parent();
    }
    out
}
