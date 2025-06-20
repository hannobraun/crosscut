use super::{
    Body, Changes, LocatedNode, NewChangeSet, NodeHash, NodePath, Nodes,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Codebase {
    root: Root,
    nodes: Nodes,
    changes: Changes,
}

impl Codebase {
    pub fn new() -> Self {
        let mut nodes = Nodes::default();

        let root = {
            let node = Body::default().into_syntax_node(&mut nodes);
            let hash = nodes.insert(node);
            Root { hash }
        };

        Self {
            root,
            nodes,
            changes: Changes::default(),
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

    pub fn latest_version_of<'r>(&'r self, path: &'r NodePath) -> &'r NodePath {
        self.changes.latest_version_of(path)
    }

    pub fn make_change<R>(
        &mut self,
        f: impl FnOnce(&mut NewChangeSet) -> R,
    ) -> R {
        let mut new_change_set =
            self.changes.new_change_set(self.root.hash, &mut self.nodes);
        let value = f(&mut new_change_set);

        let root_was_replaced =
            new_change_set.change_set().was_replaced(&self.root.path());

        if let Some(new_root) = root_was_replaced {
            self.root.hash = *new_root.hash();
        }

        value
    }
}

impl Default for Codebase {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Root {
    hash: NodeHash,
}

impl Root {
    fn path(&self) -> NodePath {
        NodePath::for_root(self.hash)
    }
}

#[cfg(test)]
mod tests {
    use crate::language::{code::NodePath, tests::infra::identifier};

    use super::Codebase;

    #[test]
    fn replace_root_node() {
        // When replacing the root node, the replacement should become the new
        // root node.

        let mut codebase = Codebase::new();

        let root = codebase.make_change(|change_set| {
            let a =
                NodePath::for_root(change_set.nodes.insert(identifier("a")));
            change_set.replace(&change_set.root_before_change(), &a);

            a
        });

        assert_eq!(codebase.root().path, root);
    }
}
