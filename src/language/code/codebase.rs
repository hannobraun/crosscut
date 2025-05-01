use super::{
    Changes, Errors, Expression, LocatedNode, NewChangeSet, NodeHash, NodePath,
    Nodes,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Codebase {
    root: Root,
    empty: NodeHash<Expression>,
    nodes: Nodes,
    changes: Changes,
    errors: Errors,
}

impl Codebase {
    pub fn new() -> Self {
        let mut nodes = Nodes::default();
        let empty = nodes.insert(Expression::Empty);

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

    pub fn errors(&self) -> &Errors {
        &self.errors
    }

    pub fn make_change<R>(
        &mut self,
        f: impl FnOnce(&mut NewChangeSet) -> R,
    ) -> R {
        let mut new_change_set = self.changes.new_change_set(
            self.root.hash,
            &mut self.nodes,
            &mut self.errors,
        );
        let value = f(&mut new_change_set);

        let root_was_replaced =
            new_change_set.change_set().was_replaced(&self.root.path());

        if let Some(new_root) = root_was_replaced {
            self.root.hash = *new_root.hash();
        }

        value
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Root {
    hash: NodeHash<Expression>,
}

impl Root {
    fn path(&self) -> NodePath {
        NodePath::for_root(self.hash)
    }
}

#[cfg(test)]
mod tests {
    use crate::language::{code::NodePath, tests::infra::error};

    use super::Codebase;

    #[test]
    fn replace_root_node() {
        // When replacing the root node, the replacement should become the new
        // root node.

        let mut codebase = Codebase::new();

        let root = codebase.make_change(|change_set| {
            let a = NodePath::for_root(change_set.nodes.insert(error("a", [])));
            change_set.replace(&change_set.root_before_change(), &a);

            a
        });

        assert_eq!(codebase.root().path, root);
    }
}
