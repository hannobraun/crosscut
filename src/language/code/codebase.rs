use std::collections::BTreeMap;

use crate::language::code::NodeKind;

use super::{
    Changes, Children, CodeError, LocatedNode, Node, NodeHash, NodePath, Nodes,
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

    pub fn children_of(&self, path: &NodePath) -> &Children {
        self.node_at(path).children()
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
        updated_parent.children_mut().add([hash]);

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
        new_parent: Node,
    ) -> NodePath {
        assert!(
            new_parent.children().contains(child.hash()),
            "Inserting a node as the parent of another, but other node is not \
            the new parent's child.",
        );

        self.replace_node(child, new_parent)
    }

    pub fn remove_node(&mut self, to_remove: &NodePath) {
        let node_to_remove = self.nodes.get(to_remove.hash());

        if let Some(parent) = self.parent_of(to_remove) {
            // The node we're removing has a parent. We need to update the
            // children of that node.

            let mut updated_parent = self.nodes.get(parent.hash()).clone();

            updated_parent.children_mut().replace(
                to_remove.hash(),
                node_to_remove.children().iter().copied(),
            );

            self.replace_node(&parent, updated_parent);
        } else {
            // The node we're removing has no parent, which means it is the root
            // node. We need to update that then.

            assert_eq!(
                &self.root,
                to_remove.hash(),
                "Removing a node that has no parent, but it's not the root \
                node.",
            );

            if node_to_remove.children().has_none() {
                // The root node we're removing has no children, but we still
                // need a new root node.

                self.root = self.empty;
            } else if let Some(child) =
                node_to_remove.children().has_one().copied()
            {
                // The root node we're removing has exactly one child, which can
                // become the new root node.

                self.root = child;
            } else {
                // The root node we're removing has multiple children, but we
                // still need a single root node afterwards.

                let mut root = self.nodes.get(&self.empty).clone();
                root.children_mut()
                    .add(node_to_remove.children().iter().copied());

                self.root = self.nodes.insert(root);
            }
        }
    }

    pub fn replace_node(
        &mut self,
        to_replace: &NodePath,
        replacement: Node,
    ) -> NodePath {
        let mut change_set = self.changes.new_change_set(&mut self.nodes);

        let mut next_to_replace = *to_replace;
        let mut next_replacement = replacement;

        let mut previous_replacement;
        let mut initial_replacement = None;

        loop {
            let hash = change_set.replace(next_to_replace, next_replacement);

            initial_replacement = initial_replacement.or(Some(hash));
            previous_replacement = hash;

            if let Some(parent) = SyntaxTree::from_root(self.root)
                .find_parent_of(&next_to_replace.hash, change_set.nodes())
            {
                next_replacement =
                    change_set.nodes().get(parent.hash()).clone();
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
    fn replace_root_node() {
        // When replacing the root node, the replacement should become the new
        // root node.

        let mut codebase = Codebase::new();

        let root = codebase.replace_node(
            &codebase.root().path,
            Node::new(NodeKind::Empty, []),
        );

        assert_eq!(codebase.root().path, root);
    }

    #[test]
    fn remove_root_node_with_single_child() {
        // When removing a root node that has a single child, that child should
        // become the new root node.

        let [a, b, ..] = test_nodes();
        let mut codebase = Codebase::new();

        let a = codebase.replace_node(&codebase.root().path, Node::new(a, []));
        let b = codebase.insert_node_as_parent(&a, Node::new(b, [*a.hash()]));
        assert_eq!(codebase.root().path, b);

        codebase.remove_node(&codebase.root().path);
        assert_eq!(codebase.root().path, a);
    }

    #[test]
    fn remove_root_node_with_no_child() {
        // When removing a root node that has no child, an empty node should be
        // left in its place.

        let [a, ..] = test_nodes();
        let mut codebase = Codebase::new();

        let a = codebase.replace_node(&codebase.root().path, Node::new(a, []));
        assert_eq!(codebase.root().path, a);

        codebase.remove_node(&a);
        assert_eq!(codebase.root().node, &Node::new(NodeKind::Empty, []));
    }

    #[test]
    fn remove_root_node_with_multiple_children() {
        // When removing a root node that has multiple children, there still
        // needs to be one root node after. An empty node can be created for
        // this.

        let [a, b, c] = test_nodes();
        let mut codebase = Codebase::new();

        let a = codebase
            .insert_node_as_child(&codebase.root().path, Node::new(a, []));
        let b = codebase
            .insert_node_as_child(&codebase.root().path, Node::new(b, []));
        let c = codebase.replace_node(
            &codebase.root().path,
            Node::new(c, [a.hash, b.hash]),
        );

        codebase.remove_node(&c);

        assert_eq!(
            codebase.root().node,
            &Node::new(NodeKind::Empty, [a.hash, b.hash]),
        );
    }

    fn test_nodes() -> [NodeKind; 3] {
        ["a", "b", "c"].map(|node| NodeKind::Error {
            node: String::from(node),
        })
    }
}
