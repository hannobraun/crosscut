use std::collections::BTreeMap;

use super::{
    nodes::{NodeHash, Nodes},
    CodeError, LocatedNode, Node, NodePath,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Codebase {
    root: NodeHash,
    nodes: Nodes,
    errors: BTreeMap<NodePath, CodeError>,
}

impl Codebase {
    pub fn new() -> Self {
        let mut nodes = Nodes::new();

        let root = {
            let child = None;
            let node = Node::empty(child);

            nodes.insert(node)
        };

        Self {
            root,
            nodes,
            errors: BTreeMap::new(),
        }
    }

    /// # Iterate over notes in the current version, from entry to root
    pub fn nodes_to_root(&self) -> impl Iterator<Item = LocatedNode> {
        let mut hashes = Vec::new();
        let mut current_node = self.root;

        loop {
            let child = self.nodes.get(&current_node).child;
            hashes.push(current_node);

            if let Some(child) = child {
                current_node = child;
            } else {
                break;
            }
        }

        hashes.into_iter().rev().map(|hash| LocatedNode {
            node: self.nodes.get(&hash),
            path: NodePath { hash },
        })
    }

    pub fn entry(&self) -> NodePath {
        let mut possible_entry = self.root;

        while let Some(child) = self.nodes.get(&possible_entry).child {
            possible_entry = child;
        }

        NodePath {
            hash: possible_entry,
        }
    }

    pub fn child_of(&self, path: &NodePath) -> Option<NodePath> {
        let hash = self.node_at(path).child?;
        Some(NodePath { hash })
    }

    pub fn parent_of(&self, path: &NodePath) -> Option<NodePath> {
        self.nodes_to_root().find_map(|located_node| {
            (located_node.node.child == Some(path.hash))
                .then_some(located_node.path)
        })
    }

    pub fn node_at(&self, path: &NodePath) -> &Node {
        self.nodes.get(path.hash())
    }

    pub fn insert_node_after(
        &mut self,
        after: NodePath,
        node: Node,
    ) -> NodePath {
        let hash = self.nodes.insert(node);

        if let Some(parent) = self.parent_of(&after) {
            let mut replacement = self.nodes.get(parent.hash()).clone();
            replacement.child = Some(hash);

            self.replace_node(&parent, replacement);
        } else {
            self.root = hash;
        }

        NodePath { hash }
    }

    pub fn replace_node(
        &mut self,
        to_replace: &NodePath,
        replacement: Node,
    ) -> NodePath {
        let mut to_replace = *to_replace;
        let mut replacement = self.nodes.insert(replacement);

        let path = NodePath { hash: replacement };

        while let Some(parent) = self.parent_of(&to_replace) {
            to_replace = parent;

            let mut parent = self.nodes.get(parent.hash()).clone();
            parent.child = Some(replacement);

            replacement = self.nodes.insert(parent);
        }

        self.root = replacement;

        path
    }

    pub fn error_at(&self, path: &NodePath) -> Option<&CodeError> {
        self.errors.get(path)
    }

    pub fn insert_error(&mut self, path: NodePath, error: CodeError) {
        self.errors.insert(path, error);
    }
}

#[cfg(test)]
mod tests {
    use crate::language::code::Node;

    use super::Codebase;

    #[test]
    fn insert_node_after_update_root() {
        let mut codebase = Codebase::new();

        let a = codebase.entry();
        let b = codebase.insert_node_after(a, Node::empty(Some(*a.hash())));

        assert_eq!(
            codebase
                .nodes_to_root()
                .map(|located_node| located_node.path)
                .collect::<Vec<_>>(),
            vec![a, b],
        );
    }
}
