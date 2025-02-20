use super::{LocatedNode, NodeHash, NodePath, Nodes};

pub struct SyntaxTree {
    pub root: NodeHash,
}

impl SyntaxTree {
    pub fn from_root(root: NodeHash) -> Self {
        Self { root }
    }

    pub fn find_leaf(self, nodes: &Nodes) -> NodeHash {
        let mut possible_leaf = self.root;

        while let Some(child) = nodes.get(&possible_leaf).child().copied() {
            possible_leaf = child;
        }

        possible_leaf
    }

    pub fn find_parent_of(
        self,
        child: &NodeHash,
        nodes: &Nodes,
    ) -> Option<NodePath> {
        let parent_of_node = |located_node: LocatedNode| {
            (located_node.node.child() == Some(child))
                .then_some(located_node.path)
        };

        let mut leaf_to_root = self.leaf_to_root(nodes);

        let maybe_parent = leaf_to_root.by_ref().find_map(parent_of_node);
        assert!(
            leaf_to_root.find_map(parent_of_node).is_none(),
            "A node should never have multiple parents within a syntax tree.",
        );

        maybe_parent
    }

    pub fn leaf_to_root(
        self,
        nodes: &Nodes,
    ) -> impl Iterator<Item = LocatedNode> {
        let mut hashes = Vec::new();
        let mut current_node = self.root;

        loop {
            let child = nodes.get(&current_node).child().copied();
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
}
