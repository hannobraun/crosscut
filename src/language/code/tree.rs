use super::{NodeHash, NodePath, Nodes};

pub struct SyntaxTree {
    pub root: NodeHash,
}

impl SyntaxTree {
    pub fn from_root(root: NodeHash) -> Self {
        Self { root }
    }

    pub fn find_parent_of(
        self,
        child: &NodePath,
        nodes: &Nodes,
    ) -> Option<NodePath> {
        let mut to_search = Vec::new();
        to_search.push(self.root);

        while let Some(hash) = to_search.pop() {
            let node = nodes.get(&hash);

            if node.children().contains(&child.hash) {
                return Some(NodePath { hash });
            }

            to_search.extend(node.children());
        }

        None
    }
}
