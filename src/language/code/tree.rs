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
        let mut to_search = Vec::new();
        to_search.push(self.root);

        while let Some(hash) = to_search.pop() {
            let node = nodes.get(&hash);

            if node.child() == Some(child) {
                return Some(NodePath { hash });
            }

            to_search.extend(node.child());
        }

        None
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
