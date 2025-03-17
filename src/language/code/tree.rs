use super::{NodePath, Nodes};

pub struct SyntaxTree {
    pub root: NodePath,
}

impl SyntaxTree {
    pub fn from_root(root: NodePath) -> Self {
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
            let node = nodes.get(&hash.hash);

            if node.children().contains(&child.hash) {
                return Some(hash);
            }

            to_search.extend(
                node.children()
                    .iter()
                    .copied()
                    .map(|hash| NodePath { hash }),
            );
        }

        None
    }
}
