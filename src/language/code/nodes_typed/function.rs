use crate::{
    language::code::{NodeAsUniform, NodeHash, Nodes, SyntaxNode},
    util::form::{Form, Owned},
};

use super::{Binding, TypedChild};

pub struct Function<T: Form> {
    pub parameter: T::Form<Binding>,

    /// # The body of the function
    ///
    /// ## Implementation Note
    ///
    /// This refers to all expressions in the function bodies by hash,
    /// regardless of what form the other fields have. This is not as type-safe
    /// as I would want these typed node types to be, but it's necessary. The
    /// hash is required to construct function values from this, at runtime.
    ///
    /// Maybe the type safety can be taken care of by an accessor method, which
    /// returns `Children`, which could get a type parameter and a conversion
    /// function into this type parameter.
    ///
    /// - Hanno Braun
    pub body: T::Form<NodeHash>,
}

impl Function<Owned> {
    pub fn new(parameter: &NodeHash, body: NodeHash, nodes: &Nodes) -> Self {
        let parameter = Binding::from_hash(parameter, nodes);
        Self { parameter, body }
    }

    pub fn body(&self) -> TypedChild {
        TypedChild::new(self.body, 1)
    }
}

impl Function<NodeAsUniform> {
    pub fn empty(_: &mut Nodes) -> Self {
        Self {
            parameter: SyntaxNode::Binding {
                name: "_".to_string(),
            },
            body: SyntaxNode::Empty,
        }
    }

    pub fn into_syntax_node(self, nodes: &mut Nodes) -> SyntaxNode {
        let parameter = nodes.insert(self.parameter);
        let body = nodes.insert(self.body);

        SyntaxNode::Function { parameter, body }
    }
}
