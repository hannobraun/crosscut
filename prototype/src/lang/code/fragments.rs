use std::{collections::BTreeMap, fmt};

use base64::{prelude::BASE64_STANDARD_NO_PAD, Engine};

use super::{Body, Expression};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Nodes {
    inner: BTreeMap<NodeId, Node>,
}

impl Nodes {
    pub fn get(&self, id: &NodeId) -> &Node {
        let Some(node) = self.inner.get(id) else {
            panic!(
                "Fragment with ID `{id:?}` not found. This should never \
                happen, unless you are mixing and matching data structures \
                from different instances of `Code`."
            );
        };
        node
    }

    pub fn insert(&mut self, node: Node) -> NodeId {
        let id = NodeId::generate_for(&node);
        self.inner.insert(id, node);
        id
    }
}

// TASK: Update documentation.
/// # The ID of a node
///
/// Node IDs are based on hashes. This means that nodes that are different from
/// another should result in different hashes. Hash collisions, meaning the same
/// IDs for different nodes, should be exceedingly unlikely.
///
/// A consequence of this, is that equal node end up with the same ID, even if
/// they are located in different parts of the syntax tree. In some situation,
/// this is not a problem. Equal nodes can often be treated the same. Where the
/// location _is_ important, you should use [`Location`] instead.
///
/// [`Location`]: super::Location
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable)]
pub struct NodeId {
    hash: [u8; 32],
}
impl NodeId {
    pub fn generate_for(node: &Node) -> Self {
        let hash = udigest::hash::<blake3::Hasher>(node).into();
        Self { hash }
    }
}

impl fmt::Debug for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", BASE64_STANDARD_NO_PAD.encode(self.hash))?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub struct Node {
    pub kind: FragmentKind,
    pub body: Body,
}

impl Node {
    #[cfg(test)]
    pub fn id(&self) -> NodeId {
        NodeId::generate_for(self)
    }

    /// # Returns the body of the fragment, if this kind can have a valid one
    ///
    /// Fragments that can have a valid body, are all fragments that allow
    /// nesting. That includes function calls, for example, whose argument is
    /// nested within the function call fragment's body.
    pub fn valid_body(&self) -> Option<&Body> {
        match self.kind {
            FragmentKind::Root
            | FragmentKind::Expression {
                expression: Expression::FunctionCall { .. },
            } => Some(&self.body),
            _ => None,
        }
    }

    #[cfg(test)]
    pub fn with_child(mut self, child: NodeId) -> Self {
        self.body.push_id(child);
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum FragmentKind {
    Root,
    Empty,
    Expression { expression: Expression },
    Error { err: FragmentError },
}

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum FragmentError {
    IntegerOverflow { value: String },
    MultiResolvedIdentifier { name: String },
    UnresolvedIdentifier { name: String },
}
