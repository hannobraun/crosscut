use crate::language::code::NodeKind;

use super::{
    Changes, Errors, LocatedNode, NewChangeSet, Node, NodeHash, NodePath,
    Nodes, SyntaxTree,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Codebase {
    root: Root,
    empty: NodeHash,
    nodes: Nodes,
    changes: Changes,
    errors: Errors,
}

impl Codebase {
    pub fn new() -> Self {
        let mut nodes = Nodes::new();
        let empty = nodes.insert(Node::new(NodeKind::Empty, []));

        Self {
            root: Root { hash: empty },
            empty,
            nodes,
            changes: Changes::new(),
            errors: Errors::new(),
        }
    }

    pub fn nodes(&self) -> &Nodes {
        &self.nodes
    }

    pub fn root(&self) -> LocatedNode {
        LocatedNode {
            node: self.nodes.get(&self.root.hash),
            path: self.root.path(),
        }
    }

    pub fn parent_of(&self, path: &NodePath) -> Option<LocatedNode> {
        let path = SyntaxTree::from_root(self.root.path())
            .find_parent_of(path, &self.nodes)?;

        Some(LocatedNode {
            node: self.nodes.get(path.hash()),
            path,
        })
    }

    pub fn node_at(&self, path: &NodePath) -> LocatedNode {
        LocatedNode {
            node: self.nodes.get(path.hash()),
            path: path.clone(),
        }
    }

    pub fn latest_version_of(&self, path: &NodePath) -> NodePath {
        self.changes.latest_version_of(path).clone()
    }

    pub fn errors(&self) -> &Errors {
        &self.errors
    }

    pub fn make_change<R>(
        &mut self,
        f: impl FnOnce(&mut NewChangeSet) -> R,
    ) -> R {
        self.make_change_with_errors(|change_set, _| f(change_set))
    }

    pub fn make_change_with_errors<R>(
        &mut self,
        f: impl FnOnce(&mut NewChangeSet, &mut Errors) -> R,
    ) -> R {
        let mut new_change_set = self.changes.new_change_set(&mut self.nodes);
        let value = f(&mut new_change_set, &mut self.errors);

        let root_was_removed =
            new_change_set.change_set().was_removed(&self.root.path());
        let root_was_replaced =
            new_change_set.change_set().was_replaced(&self.root.path());

        // I'm not even sure this can even happen. Maybe this should become an
        // `unreachable!`. But for now, it's probably good enough to make sure
        // that this precondition doesn't slip through the cracks somehow.
        assert!(
            !(root_was_removed && root_was_replaced.is_some()),
            "Both removing and replacing the root in the same change set is \
            not supported.",
        );

        if root_was_removed {
            let root = self.root().node;

            if root.children().has_none() {
                // The root node we're removing has no children, but we still
                // need a new root node.

                self.root.hash = self.empty;
            } else if let Some(child) = root.children().has_one().copied() {
                // The root node we're removing has exactly one child, which can
                // become the new root node.

                self.root.hash = child;
            } else {
                // The root node we're removing has multiple children, but we
                // still need a single root node afterwards.

                let mut new_root = self.nodes.get(&self.empty).clone();
                new_root.children_mut().add(root.children().iter().copied());

                self.root.hash = self.nodes.insert(new_root);
            }
        } else if let Some(new_root) = root_was_replaced {
            self.root.hash = *new_root.hash();
        }

        value
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Root {
    hash: NodeHash,
}

impl Root {
    fn path(&self) -> NodePath {
        NodePath::new(self.hash, None)
    }
}

#[cfg(test)]
mod tests {
    use crate::language::code::{Node, NodeKind, NodePath};

    use super::Codebase;

    #[test]
    fn replace_root_node() {
        // When replacing the root node, the replacement should become the new
        // root node.

        let [a, ..] = test_nodes();
        let mut codebase = Codebase::new();

        let old_root = codebase.root().path;
        let new_root = codebase.make_change(|change_set| {
            let a = NodePath::new(change_set.add(Node::new(a, [])), None);
            change_set.replace(&old_root, &a);

            a
        });

        assert_eq!(codebase.root().path, new_root);
    }

    #[test]
    fn remove_root_node_with_single_child() {
        // When removing a root node that has a single child, that child should
        // become the new root node.

        let [a, b, ..] = test_nodes();
        let mut codebase = Codebase::new();

        let root = codebase.root().path;
        let a = codebase.make_change(|change_set| {
            let a = change_set.add(Node::new(a, []));
            let b = change_set.add(Node::new(b, [a]));

            change_set.replace(&root, &NodePath::new(b, None));

            a
        });

        let root = codebase.root().path;
        codebase.make_change(|change_set| {
            change_set.remove(&root);
        });
        assert_eq!(*codebase.root().path.hash(), a);
    }

    #[test]
    fn remove_root_node_with_no_child() {
        // When removing a root node that has no child, an empty node should be
        // left in its place.

        let [a, ..] = test_nodes();
        let mut codebase = Codebase::new();

        let root = codebase.root().path;
        let a = codebase.make_change(|change_set| {
            let a = NodePath::new(change_set.add(Node::new(a, [])), None);
            change_set.replace(&root, &a);

            a
        });
        assert_eq!(codebase.root().path, a);

        codebase.make_change(|change_set| {
            change_set.remove(&a);
        });
        assert_eq!(codebase.root().node, &Node::new(NodeKind::Empty, []));
    }

    #[test]
    fn remove_root_node_with_multiple_children() {
        // When removing a root node that has multiple children, there still
        // needs to be one root node after. An empty node can be created for
        // this.

        let [a, b, c] = test_nodes();
        let mut codebase = Codebase::new();

        let root = codebase.root().path;
        let (a, b, c) = codebase.make_change(|change_set| {
            let a = change_set.add(Node::new(a, []));
            let b = change_set.add(Node::new(b, []));
            let c = change_set.add(Node::new(c, [a, b]));

            let c = NodePath::new(c, None);
            change_set.replace(&root, &c);

            (a, b, c)
        });

        codebase.make_change(|change_set| {
            change_set.remove(&c);
        });
        assert_eq!(codebase.root().node, &Node::new(NodeKind::Empty, [a, b]),);
    }

    fn test_nodes() -> [NodeKind; 3] {
        ["a", "b", "c"].map(|node| NodeKind::Error {
            node: String::from(node),
        })
    }
}
