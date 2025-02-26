use std::cmp::max;

use crate::language::code::{LocatedNode, NodePath, Nodes};

pub struct Layout {
    pub nodes_from_root: Vec<(u32, NodePath)>,
}

impl Layout {
    pub fn new() -> Self {
        Self {
            nodes_from_root: Vec::new(),
        }
    }
}

pub fn collect_nodes_from_root(
    node: LocatedNode,
    distance_from_root: u32,
    nodes_from_root: &mut Layout,
    nodes: &Nodes,
) -> u32 {
    nodes_from_root
        .nodes_from_root
        .push((distance_from_root, node.path));

    let mut max_distance_from_root = distance_from_root;

    // By rendering leaves first, root at the end, we are essentially inverting
    // the tree, compared to how we usually think about trees. We do _not_ want
    // to invert the order of a node's children though. Otherwise, when working
    // on code that adds/removes children, our intuition won't match how we
    // think about this when manipulating children in the editor.
    for child in node.children(nodes).rev() {
        let distance_from_root = collect_nodes_from_root(
            child,
            distance_from_root + 1,
            nodes_from_root,
            nodes,
        );

        max_distance_from_root =
            max(max_distance_from_root, distance_from_root);
    }

    max_distance_from_root
}
