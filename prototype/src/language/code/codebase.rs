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
    pub fn entry_to_root(&self) -> impl Iterator<Item = LocatedNode> {
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
        self.entry_to_root().find_map(|located_node| {
            (located_node.node.child == Some(path.hash))
                .then_some(located_node.path)
        })
    }

    pub fn node_at(&self, path: &NodePath) -> &Node {
        self.nodes.get(path.hash())
    }

    /// # Insert a node as the parent of another
    ///
    /// The new node takes the place of the other node in the syntax tree. The
    /// other node's parent (and its parents, recursively) are updated
    /// accordingly.
    ///
    /// ## Panics
    ///
    /// Panics, if the inserted node does not have its child node set correctly.
    ///
    /// ## Implementation Note
    ///
    /// This function is a bit weird. It explicitly inserts a node as the parent
    /// of another, but then requires the caller to set the new parent's `child`
    /// field explicitly. It could take a `NodeKind` instead, and do that
    /// itself.
    ///
    /// I've decided not to fix that for now, because I first want to see how
    /// this API (and its caller's needs) evolve, as the language expands and
    /// the syntax tree takes something akin to its final form.
    pub fn insert_as_parent_of(
        &mut self,
        child: NodePath,
        node: Node,
    ) -> NodePath {
        assert_eq!(
            node.child.as_ref(),
            Some(child.hash()),
            "Inserting a node as the parent of another, but other node is not \
            the new parent's child.",
        );

        let hash = self.nodes.insert(node);

        if let Some(parent) = self.parent_of(&child) {
            let mut updated_parent = self.nodes.get(parent.hash()).clone();
            updated_parent.child = Some(hash);

            self.replace_node(&parent, updated_parent);
        } else {
            self.root = hash;
        }

        NodePath { hash }
    }

    #[allow(unused)] // code using this function is being worked on
    pub fn remove_node(&mut self, to_remove: &NodePath) {
        let node_to_remove = self.nodes.get(to_remove.hash());

        if let Some(parent) = self.parent_of(to_remove) {
            let mut updated_parent = self.nodes.get(parent.hash()).clone();
            updated_parent.child = node_to_remove.child;

            self.replace_node(&parent, updated_parent);
        } else {
            // In principle, we'd have to update the root here. We can currently
            // get away with not doing that, as this function is never used on
            // the root node.
            //
            // I'll fix this, but I want to add a test for that first.
            todo!("Removing the root node is not supported yet.");
        }
    }

    pub fn replace_node(
        &mut self,
        to_replace: &NodePath,
        replacement: Node,
    ) -> NodePath {
        let mut next_to_replace = *to_replace;
        let mut replacement = self.nodes.insert(replacement);

        let mut path = Some(NodePath { hash: replacement });

        loop {
            let Some(parent) = self.parent_of(&next_to_replace) else {
                break;
            };
            next_to_replace = parent;

            let mut parent = self.nodes.get(next_to_replace.hash()).clone();
            parent.child = Some(replacement);

            let new_replacement = self.nodes.insert(parent);
            path = path.or(Some(NodePath {
                hash: new_replacement,
            }));
            replacement = new_replacement;
        }

        self.root = replacement;

        let Some(path) = path else {
            unreachable!("`path` is set above.");
        };

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
        let b = codebase.insert_as_parent_of(a, Node::empty(Some(*a.hash())));

        assert_eq!(
            codebase
                .entry_to_root()
                .map(|located_node| located_node.path)
                .collect::<Vec<_>>(),
            vec![a, b],
        );
    }
}
