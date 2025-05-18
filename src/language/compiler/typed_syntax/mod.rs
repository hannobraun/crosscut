mod node;

pub use self::node::TypedNode;

use crate::language::code::{Child, Children, Nodes, SyntaxNode};

pub struct Apply {
    expression: SyntaxNode,
    argument: SyntaxNode,
}

impl Apply {
    #[cfg(test)]
    pub fn with_expression(mut self, expression: SyntaxNode) -> Self {
        self.expression = expression;
        self
    }

    #[cfg(test)]
    pub fn with_argument(mut self, argument: SyntaxNode) -> Self {
        self.argument = argument;
        self
    }

    pub fn into_syntax_node(self, nodes: &mut Nodes) -> SyntaxNode {
        let [expression, argument] =
            [self.expression, self.argument].map(|node| nodes.insert(node));

        SyntaxNode::Apply {
            expression: Child::new(expression, 0),
            argument: Child::new(argument, 1),
        }
    }
}

impl Default for Apply {
    fn default() -> Self {
        Self {
            expression: SyntaxNode::Empty,
            argument: SyntaxNode::Empty,
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
