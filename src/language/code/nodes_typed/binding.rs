use crate::language::code::{NodeHash, Nodes, SyntaxNode};

pub struct Binding {
    pub name: String,
}

impl Binding {
    pub fn new(hash: &NodeHash, nodes: &Nodes) -> Self {
        let SyntaxNode::Binding { name } = nodes.get(hash) else {
            panic!("Expected node to be a binding.");
        };

        Self { name: name.clone() }
    }
}
