use std::collections::BTreeMap;

use super::{
    nodes::{NodeHash, Nodes},
    CodeError, LocatedNode, Node, NodePath,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Codebase {
    root: NodeHash,
    nodes: Nodes,
    context: Vec<NodeHash>,
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
            context: vec![root],
            errors: BTreeMap::new(),
        }
    }

    pub fn nodes(&self) -> impl Iterator<Item = LocatedNode> {
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

        hashes
            .into_iter()
            .rev()
            .enumerate()
            .map(|(index, hash)| LocatedNode {
                node: self.nodes.get(&hash),
                path: NodePath { hash, index },
            })
    }

    pub fn entry(&self) -> NodePath {
        let mut possible_entry = self.root;

        while let Some(child) = self.nodes.get(&possible_entry).child {
            possible_entry = child;
        }

        NodePath {
            hash: possible_entry,
            index: 0,
        }
    }

    pub fn child_of(&self, path: &NodePath) -> Option<NodePath> {
        let hash = self.node_at(path).child?;

        assert!(
            path.index > 0,
            "A child for the node at this path exists. We just found its hash. \
            Therefore, it can't be the first node, which means it must have an \
            index that's not zero."
        );
        let previous_index = path.index - 1;

        Some(NodePath {
            hash,
            index: previous_index,
        })
    }

    pub fn parent_of(&self, path: &NodePath) -> Option<NodePath> {
        self.nodes().find_map(|located_node| {
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

        // In principle, we would have to update all the parent nodes here, as
        // we do in `replace_node`. We can currently get away without that
        // though, due to the way this method is used in conjunction with
        // `replace_node`.
        //
        // I think it's fine for now. I expect to simplify how code is stored
        // soon enough, and I expect this function to be based on `replace_node`
        // then. Which would mean it would benefit from the updating that
        // `replace_node` already does.

        let path = NodePath {
            hash,
            index: after.index + 1,
        };
        self.context.insert(path.index, hash);
        path
    }

    pub fn replace_node(
        &mut self,
        to_replace: &NodePath,
        replacement: Node,
    ) -> NodePath {
        let mut replacement = self.nodes.insert(replacement);
        self.context[to_replace.index] = replacement;

        let path = NodePath {
            hash: replacement,
            index: to_replace.index,
        };

        // All parent still point to the replaced node. Update them.

        for hash in &mut self.context[to_replace.index + 1..] {
            let mut parent = self.nodes.get(hash).clone();
            parent.child = Some(replacement);

            replacement = self.nodes.insert(parent);
            *hash = replacement;
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
