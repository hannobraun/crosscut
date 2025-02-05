use super::{
    nodes::{NodeHash, Nodes},
    LocatedNode, NodePath,
};

pub struct SyntaxTree {
    pub root: NodeHash,
}

impl SyntaxTree {
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
}
