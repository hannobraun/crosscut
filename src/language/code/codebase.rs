use std::collections::BTreeMap;

use crate::language::code::NodeKind;

use super::{
    Changes, CodeError, LocatedNode, Node, NodeHash, NodePath, Nodes,
    SyntaxTree, nodes::Children,
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
        let empty = nodes.insert(Node::new(NodeKind::Empty, None));

        Self {
            root: empty,
            empty,
            nodes,
            changes: Changes::new(),
            errors: BTreeMap::new(),
        }
    }

    pub fn nodes(&self) -> &Nodes {
        &self.nodes
    }

    pub fn root(&self) -> LocatedNode {
        LocatedNode {
            node: self.nodes.get(&self.root),
            path: NodePath { hash: self.root },
        }
    }

    pub fn leaf(&self) -> NodePath {
        let hash = SyntaxTree::from_root(self.root).find_leaf(&self.nodes);
        NodePath { hash }
    }

    pub fn children_of(&self, path: &NodePath) -> Children {
        let child = self.node_at(path).child().copied();
        Children { child }
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

    pub fn insert_node_as_child(
        &mut self,
        parent: &NodePath,
        node: Node,
    ) -> NodePath {
        let hash = self.nodes.insert(node);

        let mut updated_parent = self.nodes.get(parent.hash()).clone();
        updated_parent.children_mut().add(hash);

        self.replace_node(parent, updated_parent);

        NodePath { hash }
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
    pub fn insert_node_as_parent(
        &mut self,
        child: &NodePath,
        node: Node,
    ) -> NodePath {
        assert_eq!(
            node.child(),
            Some(child.hash()),
            "Inserting a node as the parent of another, but other node is not \
            the new parent's child.",
        );

        let hash = self.nodes.insert(node);

        if let Some(parent) = self.parent_of(child) {
            let mut updated_parent = self.nodes.get(parent.hash()).clone();
            updated_parent.children_mut().replace(child.hash(), [hash]);

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

            if let Some(child) = node_to_remove.child().copied() {
                updated_parent
                    .children_mut()
                    .replace(to_remove.hash(), [child]);
            } else {
                updated_parent.children_mut().replace(to_remove.hash(), []);
            }

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
                next_replacement = self.nodes.get(parent.hash()).clone();
                next_replacement
                    .children_mut()
                    .replace(next_to_replace.hash(), [previous_replacement]);

                next_to_replace = parent;

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
        // If inserting a node after the root node, the root should be updated.

        let mut codebase = Codebase::new();

        let a = codebase.root().path;
        let b = codebase.insert_node_as_parent(
            &a,
            Node::new(NodeKind::Empty, Some(*a.hash())),
        );

        assert_eq!(codebase.root().path, b);
    }

    #[test]
    fn remove_node_should_update_root_node() {
        let mut codebase = Codebase::new();

        let a = codebase.replace_node(
            &codebase.leaf(),
            Node::new(
                NodeKind::Error {
                    node: String::from("a"),
                },
                None,
            ),
        );
        let b = codebase.insert_node_as_parent(
            &a,
            Node::new(
                NodeKind::Error {
                    node: String::from("b"),
                },
                Some(*a.hash()),
            ),
        );

        assert_eq!(codebase.root().path, b);

        codebase.remove_node(&b);
        assert_eq!(codebase.root().path, a);

        codebase.remove_node(&a);
        assert_eq!(codebase.root().node, &Node::new(NodeKind::Empty, None));
    }
}
