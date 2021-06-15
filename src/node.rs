use crate::platform::get_metadata;

use std::cmp::Ordering;
use std::path::PathBuf;

#[derive(Debug, Eq, Clone)]
pub struct DisplayNode {
    pub name: PathBuf, //todo: consider moving to a string?
    pub size: u64,
    pub children: Vec<DisplayNode>,
}

#[derive(Debug, Eq, Clone)]
pub struct Node {
    pub name: PathBuf,
    pub size: u64,
    pub children: Vec<Node>,
    pub inode_device: Option<(u64, u64)>,
}

pub fn build_node(
    dir: PathBuf,
    children: Vec<Node>,
    use_apparent_size: bool,
    by_filecount: bool,
) -> Option<Node> {
    match get_metadata(&dir, use_apparent_size) {
        Some(data) => {
            let (size, inode_device) = if by_filecount { (1, data.1) } else { data };
            Some(Node {
                name: dir,
                size,
                children,
                inode_device,
            })
        }
        None => None,
    }
}

impl Ord for DisplayNode {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.size == other.size {
            self.name.cmp(&other.name)
        } else {
            self.size.cmp(&other.size)
        }
    }
}

impl PartialOrd for DisplayNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for DisplayNode {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.size == other.size && self.children == other.children
    }
}

impl DisplayNode {
    pub fn num_siblings(&self) -> u64 {
        self.children.len() as u64
    }

    pub fn get_children_from_node(&self, is_reversed: bool) -> impl Iterator<Item = DisplayNode> {
        if is_reversed {
            let children: Vec<DisplayNode> = self.children.clone().into_iter().rev().collect();
            children.into_iter()
        } else {
            self.children.clone().into_iter()
        }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.size == other.size && self.children == other.children
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.size == other.size {
            self.name.cmp(&other.name)
        } else {
            self.size.cmp(&other.size)
        }
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
