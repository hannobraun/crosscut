use crate::language::code::{Children, Nodes, SyntaxNode};

pub enum TypedNode {
    Expression,
    Pattern,
    Other,
}

impl TypedNode {
    pub fn from_syntax_node(syntax_node: &SyntaxNode) -> Self {
        match syntax_node {
            SyntaxNode::AddNode | SyntaxNode::Test { .. } => Self::Other,

            SyntaxNode::Apply { .. }
            | SyntaxNode::Empty
            | SyntaxNode::Function { .. }
            | SyntaxNode::Identifier { .. }
            | SyntaxNode::Number { .. }
            | SyntaxNode::Recursion
            | SyntaxNode::Tuple { .. } => Self::Expression,

            SyntaxNode::Binding { .. } => Self::Pattern,
        }
    }
}

pub struct Function;

impl Function {
    pub fn to_syntax_node(&self, nodes: &mut Nodes) -> SyntaxNode {
        let parameter = nodes.insert(SyntaxNode::Binding {
            name: "_".to_string(),
        });
        let body = Children::new([nodes.insert(SyntaxNode::Empty)]);

        SyntaxNode::Function { parameter, body }
    }
}

pub struct Tuple;

impl Tuple {
    pub fn to_syntax_node(&self, nodes: &mut Nodes) -> SyntaxNode {
        let values = Children::new([]);
        let add_value = nodes.insert(SyntaxNode::AddNode);

        SyntaxNode::Tuple { values, add_value }
    }
}
