use std::fs;

use crate::node::Node;
use rayon::iter::ParallelBridge;
use rayon::prelude::ParallelIterator;
use std::path::PathBuf;

use std::sync::atomic;
use std::sync::atomic::AtomicBool;

use std::collections::HashSet;

use crate::node::build_node;

pub fn walk_it(
    dirs: HashSet<PathBuf>,
    ignore_directories: HashSet<PathBuf>,
    use_apparent_size: bool,
    by_filecount: bool,
    show_hidden: bool,
) -> (Vec<Node>, bool) {
    let permissions_flag = AtomicBool::new(false);

    let top_level_nodes: Vec<_> = dirs
        .into_iter()
        .filter_map(|d| {
            let n = walk(
                d,
                &permissions_flag,
                &ignore_directories,
                use_apparent_size,
                by_filecount,
                show_hidden,
            );
            match n {
                Some(n) => {
                    let mut inodes: HashSet<(u64, u64)> = HashSet::new();
                    clean_inodes(n, &mut inodes, use_apparent_size)
                }
                None => None,
            }
        })
        .collect();
    (top_level_nodes, permissions_flag.into_inner())
}

// Remove files which have the same inode, we don't want to double count them.
fn clean_inodes(
    x: Node,
    inodes: &mut HashSet<(u64, u64)>,
    use_apparent_size: bool,
) -> Option<Node> {
    if use_apparent_size {
        if let Some(id) = x.inode_device {
            if inodes.contains(&id) {
                return None;
            }
            inodes.insert(id);
        }
    }

    let new_children: Vec<_> = x
        .children
        .into_iter()
        .filter_map(|c| clean_inodes(c, inodes, use_apparent_size))
        .collect();

    return Some(Node {
        name: x.name,
        size: x.size + new_children.iter().map(|c| c.size).sum::<u64>(),
        children: new_children,
        inode_device: x.inode_device,
    });
}

fn walk(
    dir: PathBuf,
    permissions_flag: &AtomicBool,
    ignore_directories: &HashSet<PathBuf>,
    use_apparent_size: bool,
    by_filecount: bool,
    show_hidden: bool,
) -> Option<Node> {
    let mut children = vec![];

    if let Ok(entries) = fs::read_dir(dir.clone()) {
        children = entries
            .into_iter()
            .par_bridge()
            .filter_map(|entry| {
                if let Ok(ref entry) = entry {
                    // uncommenting the below line gives simpler code but
                    // rayon doesn't parallelise as well giving a 3X performance drop
                    // hence we unravel the recursion a bit

                    // return walk(entry.path(), permissions_flag, ignore_directories, use_apparent_size, by_filecount, show_hidden);

                    // A clunky if rewrite this
                    if (!entry.file_name().to_str().unwrap_or("").starts_with('.') || show_hidden)
                        && !ignore_directories.contains(&entry.path())
                    {
                        if let Ok(data) = entry.file_type() {
                            if data.is_symlink() {
                                // return None;
                                return build_node(
                                    entry.path(),
                                    vec![],
                                    use_apparent_size,
                                    by_filecount,
                                );
                            }
                            if data.is_dir() {
                                return walk(
                                    entry.path(),
                                    permissions_flag,
                                    ignore_directories,
                                    use_apparent_size,
                                    by_filecount,
                                    show_hidden,
                                );
                            }
                            return build_node(
                                entry.path(),
                                vec![],
                                use_apparent_size,
                                by_filecount,
                            );
                        }
                    }
                } else {
                    permissions_flag.store(true, atomic::Ordering::Relaxed);
                }
                None
            })
            .collect();
    } else {
        permissions_flag.store(true, atomic::Ordering::Relaxed);
    }
    build_node(dir, children, use_apparent_size, by_filecount)
}
