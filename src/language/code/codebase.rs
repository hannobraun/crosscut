use std::collections::BTreeMap;

use super::{
    Changes, CodeError, LocatedNode, Node, NodeHash, NodePath, Nodes,
    SyntaxTree,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Codebase {
    root: NodeHash,
    empty: NodeHash,
    nodes: Nodes,
    changes: Changes,
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
            empty: root,
            nodes,
            changes: Changes::new(),
            errors: BTreeMap::new(),
        }
    }

    pub fn root(&self) -> LocatedNode {
        LocatedNode {
            node: self.nodes.get(&self.root),
            path: NodePath { hash: self.root },
        }
    }

    /// # Iterate over notes in the current version, from entry to root
    pub fn leaf_to_root(&self) -> impl Iterator<Item = LocatedNode> {
        SyntaxTree::from_root(self.root).leaf_to_root(&self.nodes)
    }

    pub fn leaf(&self) -> NodePath {
        let hash = SyntaxTree::from_root(self.root).find_leaf(&self.nodes);
        NodePath { hash }
    }

    pub fn child_of(&self, path: &NodePath) -> Option<NodePath> {
        let hash = *self.node_at(path).child()?;
        Some(NodePath { hash })
    }

    pub fn parent_of(&self, path: &NodePath) -> Option<NodePath> {
        SyntaxTree::from_root(self.root).find_parent_of(&path.hash, &self.nodes)
    }

    pub fn node_at(&self, path: &NodePath) -> &Node {
        self.nodes.get(path.hash())
    }

    pub fn latest_version_of(&self, path: NodePath) -> NodePath {
        self.changes.latest_version_of(path)
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
            node.child(),
            Some(child.hash()),
            "Inserting a node as the parent of another, but other node is not \
            the new parent's child.",
        );

        let hash = self.nodes.insert(node);

        if let Some(parent) = self.parent_of(&child) {
            let mut updated_parent = self.nodes.get(parent.hash()).clone();
            *updated_parent.child_mut() = Some(hash);

            self.replace_node(&parent, updated_parent);
        } else {
            self.root = hash;
        }

        NodePath { hash }
    }

    pub fn remove_node(&mut self, to_remove: &NodePath) {
        let node_to_remove = self.nodes.get(to_remove.hash());

        if let Some(parent) = self.parent_of(to_remove) {
            let mut updated_parent = self.nodes.get(parent.hash()).clone();
            *updated_parent.child_mut() = node_to_remove.child().copied();

            self.replace_node(&parent, updated_parent);
        } else {
            self.root = node_to_remove.child().copied().unwrap_or(self.empty);
        }
    }

    pub fn replace_node(
        &mut self,
        to_replace: &NodePath,
        replacement: Node,
    ) -> NodePath {
        let change_set = self.changes.new_change_set();

        let mut next_to_replace = *to_replace;
        let mut next_replacement = replacement;

        let mut previous_replacement;
        let mut initial_replacement = None;

        loop {
            let hash = self.nodes.insert(next_replacement);
            change_set.add(next_to_replace, NodePath { hash });

            initial_replacement = initial_replacement.or(Some(hash));
            previous_replacement = hash;

            if let Some(parent) = SyntaxTree::from_root(self.root)
                .find_parent_of(&next_to_replace.hash, &self.nodes)
            {
                next_to_replace = parent;

                next_replacement =
                    self.nodes.get(next_to_replace.hash()).clone();
                *next_replacement.child_mut() = Some(previous_replacement);

                continue;
            } else {
                break;
            };
        }

        self.root = previous_replacement;

        if let Some(hash) = initial_replacement {
            NodePath { hash }
        } else {
            unreachable!(
                "The loop above is executed at least once. The variable must \
                have been set."
            );
        }
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
    use crate::language::code::{Node, NodeKind};

    use super::Codebase;

    #[test]
    fn insert_node_after_update_root() {
        let mut codebase = Codebase::new();

        let a = codebase.leaf();
        let b = codebase.insert_as_parent_of(a, Node::empty(Some(*a.hash())));

        assert_eq!(
            codebase
                .leaf_to_root()
                .map(|located_node| located_node.path)
                .collect::<Vec<_>>(),
            vec![a, b],
        );
    }

    #[test]
    fn remove_node_should_update_root_node() {
        let mut codebase = Codebase::new();

        let a = codebase.replace_node(
            &codebase.leaf(),
            Node {
                kind: NodeKind::Error {
                    node: String::from("a"),
                },
                child: None,
            },
        );
        let b = codebase.insert_as_parent_of(
            a,
            Node {
                kind: NodeKind::Error {
                    node: String::from("b"),
                },
                child: Some(*a.hash()),
            },
        );

        assert_eq!(codebase.root().path, b);

        codebase.remove_node(&b);
        assert_eq!(codebase.root().path, a);

        codebase.remove_node(&a);
        assert_eq!(codebase.root().node, &Node::empty(None));
    }
}
