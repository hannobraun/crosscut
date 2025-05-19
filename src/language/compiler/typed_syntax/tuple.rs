use crate::language::code::{ChildrenOwned, Nodes, SyntaxNode};

#[derive(Default)]
pub struct Tuple {}

impl Tuple {
    pub fn into_syntax_node(self, nodes: &mut Nodes) -> SyntaxNode {
        let values = [].map(|node| nodes.insert(node));
        let add_value = nodes.insert(SyntaxNode::AddNode);

        SyntaxNode::Tuple {
            values: ChildrenOwned::new(values),
            add_value,
        }
    }
}
