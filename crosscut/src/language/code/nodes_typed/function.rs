use crate::{
    language::code::{NodeAsUniform, NodeHash, Nodes, SyntaxNode},
    util::form::{Form, Owned},
};

use super::{Binding, TypedChild};

#[derive(Debug)]
pub struct Function<T: Form> {
    pub parameter: T::Form<Binding>,

    /// # The body of the function
    ///
    /// Refers to the body node by hash, regardless of what form the other
    /// fields have. This is necessary, so function values can be constructed
    /// from this at runtime.
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
    pub fn empty(nodes: &mut Nodes) -> Self {
        Self {
            parameter: SyntaxNode::Binding {
                name: "".to_string(),
            },
            body: SyntaxNode::Body {
                children: Vec::new(),
                add: nodes.insert(SyntaxNode::Add),
            },
        }
    }

    pub fn into_syntax_node(self, nodes: &mut Nodes) -> SyntaxNode {
        let parameter = nodes.insert(self.parameter);
        let body = nodes.insert(self.body);

        SyntaxNode::Function { parameter, body }
    }
}
