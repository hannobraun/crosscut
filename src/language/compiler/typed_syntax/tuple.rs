use crate::language::code::{ChildrenOwned, Nodes, SyntaxNode};

#[derive(Default)]
pub struct Tuple {}

impl Tuple {
    pub fn into_syntax_node(self, nodes: &mut Nodes) -> SyntaxNode {
        let values = []
            .into_iter()
            .map(|node| nodes.insert(node))
            .collect::<Vec<_>>();
        let add_value = nodes.insert(SyntaxNode::AddNode);

        SyntaxNode::Tuple {
            values: ChildrenOwned::new(values),
            add_value,
        }
    }
}
