use super::{
    nodes::{NodeHash, Nodes},
    LocatedNode, NodePath,
};

pub struct SyntaxTree {
    pub root: NodeHash,
}

impl SyntaxTree {
    pub fn from_root(root: NodeHash) -> Self {
        Self { root }
    }

    pub fn leaf_to_root(
        self,
        nodes: &Nodes,
    ) -> impl Iterator<Item = LocatedNode> {
        let mut hashes = Vec::new();
        let mut current_node = self.root;

        loop {
            let child = nodes.get(&current_node).child;
            hashes.push(current_node);

            if let Some(child) = child {
                current_node = child;
            } else {
                break;
            }
        }

        hashes.into_iter().rev().map(|hash| LocatedNode {
            node: nodes.get(&hash),
            path: NodePath { hash },
        })
    }

    pub fn find_parent_of(
        self,
        node: &NodeHash,
        nodes: &Nodes,
    ) -> Option<NodePath> {
        let has_node_as_child = |located_node: &LocatedNode| {
            located_node.node.child.as_ref() == Some(node)
        };

        let mut leaf_to_root = self.leaf_to_root(nodes);

        leaf_to_root
            .by_ref()
            .find(has_node_as_child)
            .map(|located_node| located_node.path)
    }
}
