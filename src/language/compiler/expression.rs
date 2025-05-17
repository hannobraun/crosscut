use crate::language::code::{NodeHash, Nodes, SyntaxNode};

use super::{Function, Tuple, typed_nodes::Apply};

pub fn compile(token: &str, nodes: &mut Nodes) -> NodeHash {
    let node = if token.is_empty() {
        SyntaxNode::Empty
    } else if let Some(node) = resolve_keyword(token, nodes) {
        node
    } else if let Some(node) = resolve_literal(token, nodes) {
        node
    } else {
        SyntaxNode::Identifier {
            name: token.to_string(),
        }
    };

    nodes.insert(node)
}

fn resolve_keyword(name: &str, nodes: &mut Nodes) -> Option<SyntaxNode> {
    match name {
        "apply" => Some(Apply::new().to_syntax_node(nodes)),
        "self" => Some(SyntaxNode::Recursion),
        _ => None,
    }
}

fn resolve_literal(name: &str, nodes: &mut Nodes) -> Option<SyntaxNode> {
    if let Ok(value) = name.parse() {
        Some(SyntaxNode::Number { value })
    } else {
        match name {
            "fn" => Some(Function.to_syntax_node(nodes)),
            "tuple" => Some(Tuple.to_syntax_node(nodes)),
            _ => None,
        }
    }
}
