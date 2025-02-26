use std::cmp::max;

use crate::language::code::{LocatedNode, NodePath, Nodes};

pub struct EditorLayout {
    pub lines: Vec<EditorLine>,
}

impl EditorLayout {
    pub fn new(root: LocatedNode, nodes: &Nodes) -> Self {
        let mut nodes_from_root = Vec::new();
        let max_distance_from_root =
            collect_nodes_from_root(root, 0, &mut nodes_from_root, nodes);

        let lines = nodes_from_root
            .into_iter()
            .rev()
            .map(|node| {
                let level_of_indentation =
                    max_distance_from_root - node.distance_from_root;

                EditorLine {
                    node,
                    level_of_indentation,
                }
            })
            .collect();

        Self { lines }
    }
}

pub struct EditorLine {
    pub node: NodeInLayout,
    pub level_of_indentation: u32,
}

pub struct NodeInLayout {
    pub path: NodePath,
    pub distance_from_root: u32,
}

fn collect_nodes_from_root(
    node: LocatedNode,
    distance_from_root: u32,
    nodes_from_root: &mut Vec<NodeInLayout>,
    nodes: &Nodes,
) -> u32 {
    nodes_from_root.push(NodeInLayout {
        path: node.path,
        distance_from_root,
    });

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
