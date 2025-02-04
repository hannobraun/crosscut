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
        self.context
            .iter()
            .enumerate()
            .map(|(index, hash)| LocatedNode {
                node: self.nodes.get(hash),
                path: NodePath { hash: *hash, index },
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
        let hash = self
            .context
            .iter()
            .find(|hash| self.nodes.get(hash).child == Some(path.hash))?;

        let next_index = path.index + 1;
        assert!(
            next_index <= self.context.len(),
            "This is an append-only data structure. Every existing `Location` \
            must be valid, or it wouldn't have been created in the first \
            place.\n\
            \n\
            As a result, incrementing the index of an existing location must \
            result in an index that is either valid, or right next to the \
            valid range.",
        );

        if next_index < self.context.len() {
            Some(NodePath {
                hash: *hash,
                index: next_index,
            })
        } else {
            None
        }
    }

    pub fn node_at(&self, path: &NodePath) -> &Node {
        let Some(id) = self.context.get(path.index) else {
            unreachable!(
                "This is an append-only data structure. Every existing \
                `Location` must be valid, or it wouldn't have been created in \
                the first place."
            );
        };

        self.nodes.get(id)
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

        let at = NodePath {
            hash,
            index: after.index + 1,
        };
        self.context.insert(at.index, hash);
        at
    }

    pub fn replace_node(
        &mut self,
        to_replace: &NodePath,
        replacement: Node,
    ) -> NodePath {
        let hash = self.nodes.insert(replacement);
        self.context[to_replace.index] = hash;

        // All parent still point to the replaced node. Update them.
        let mut child_hash = hash;
        for hash in &mut self.context[to_replace.index + 1..] {
            let mut node = self.nodes.get(hash).clone();
            node.child = Some(child_hash);
            *hash = self.nodes.insert(node);
            child_hash = *hash;
        }

        self.root = child_hash;

        NodePath {
            hash,
            index: to_replace.index,
        }
    }

    pub fn error_at(&self, path: &NodePath) -> Option<&CodeError> {
        self.errors.get(path)
    }

    pub fn insert_error(&mut self, path: NodePath, error: CodeError) {
        self.errors.insert(path, error);
    }
}
