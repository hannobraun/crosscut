use super::{
    Changes, Errors, LocatedNode, NewChangeSet, Node, NodeHash, NodePath, Nodes,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Codebase {
    root: Root,
    empty: NodeHash<Node>,
    nodes: Nodes,
    changes: Changes,
    errors: Errors,
}

impl Codebase {
    pub fn new() -> Self {
        let mut nodes = Nodes::new();
        let empty = nodes.insert(Node::Empty);

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

            if root.has_no_children() {
                // The root node we're removing has no children, but we still
                // need a new root node.

                self.root.hash = self.empty;
            } else if let Some(child) = root.has_single_child().copied() {
                // The root node we're removing has exactly one child, which can
                // become the new root node.

                self.root.hash = child;
            } else {
                // The root node we're removing has multiple children, but we
                // still need a single root node afterwards.
                //
                // Since we're just conjuring up a new node, there are no
                // contents we could put in there. And since an empty node with
                // multiple children is always an error, that's what we're
                // creating here.

                let new_root = Node::Error {
                    node: "".to_string(),
                    children: root.to_children(),
                };

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
    hash: NodeHash<Node>,
}

impl Root {
    fn path(&self) -> NodePath {
        NodePath::for_root(self.hash)
    }
}

#[cfg(test)]
mod tests {
    use crate::language::{
        code::{Children, Node, NodePath},
        tests::infra::node,
    };

    use super::Codebase;

    #[test]
    fn replace_root_node() {
        // When replacing the root node, the replacement should become the new
        // root node.

        let mut codebase = Codebase::new();

        let old_root = codebase.root().path;
        let new_root = codebase.make_change(|change_set| {
            let a = NodePath::for_root(
                change_set.nodes_mut().insert(node("a", [])),
            );
            change_set.replace(&old_root, &a);

            a
        });

        assert_eq!(codebase.root().path, new_root);
    }

    #[test]
    fn remove_root_node_with_single_child() {
        // When removing a root node that has a single child, that child should
        // become the new root node.

        let mut codebase = Codebase::new();

        let root = codebase.root().path;
        let a = codebase.make_change(|change_set| {
            let a = change_set.nodes_mut().insert(node("a", []));
            let b = change_set.nodes_mut().insert(node("b", [a]));

            change_set.replace(&root, &NodePath::for_root(b));

            a
        });

        let root = codebase.root().path;
        codebase.make_change(|change_set| {
            change_set.remove(&root);
        });
        assert_eq!(codebase.root().path.hash(), &a);
    }

    #[test]
    fn remove_root_node_with_no_child() {
        // When removing a root node that has no child, an empty node should be
        // left in its place.

        let mut codebase = Codebase::new();

        let root = codebase.root().path;
        let a = codebase.make_change(|change_set| {
            let a = NodePath::for_root(
                change_set.nodes_mut().insert(node("a", [])),
            );
            change_set.replace(&root, &a);

            a
        });
        assert_eq!(codebase.root().path, a);

        codebase.make_change(|change_set| {
            change_set.remove(&a);
        });
        assert_eq!(codebase.root().node, &Node::Empty);
    }

    #[test]
    fn remove_root_node_with_multiple_children() {
        // When removing a root node that has multiple children, there still
        // needs to be one root node after. An empty node can be created for
        // this.

        let mut codebase = Codebase::new();

        let root = codebase.root().path;
        let (a, b, c) = codebase.make_change(|change_set| {
            let a = change_set.nodes_mut().insert(node("a", []));
            let b = change_set.nodes_mut().insert(node("b", []));
            let c = change_set.nodes_mut().insert(node("c", [a, b]));

            let c = NodePath::for_root(c);
            change_set.replace(&root, &c);

            (a, b, c)
        });

        codebase.make_change(|change_set| {
            change_set.remove(&c);
        });
        assert_eq!(
            codebase.root().node,
            &Node::Error {
                node: "".to_string(),
                children: Children::new([a, b]),
            },
        );
    }
}
