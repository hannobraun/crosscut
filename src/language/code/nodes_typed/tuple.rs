use crate::{
    language::code::{NodeByHash, Nodes, SyntaxNode},
    util::form::{Form, Owned},
};

use super::{Body, TypedChild};

#[derive(Debug)]
pub struct Tuple<T: Form> {
    pub values: T::Form<Body<Owned>>,
}

impl Tuple<Owned> {
    pub fn empty() -> Self {
        Self {
            values: Body::empty(),
        }
    }

    pub fn into_syntax_node(self, nodes: &mut Nodes) -> SyntaxNode {
        let values = {
            let node = self.values.into_syntax_node(nodes);
            nodes.insert(node)
        };

        SyntaxNode::Tuple { values }
    }
}

impl Tuple<NodeByHash> {
    pub fn values(&self) -> TypedChild {
        TypedChild::new(self.values, 0)
    }
}
