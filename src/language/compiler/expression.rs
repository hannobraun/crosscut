use crate::language::{
    code::{CodeError, Errors, NodeHash, Nodes, SyntaxNode},
    packages::Packages,
};

use super::{Function, Tuple};

pub fn compile(
    token: &str,
    nodes: &mut Nodes,
    errors: &mut Errors,
    packages: &Packages,
) -> NodeHash {
    let (node, maybe_error) = if token.is_empty() {
        (SyntaxNode::Empty, None)
    } else if let Some(node) = resolve_keyword(token, nodes) {
        (node, None)
    } else if let Some(node) = resolve_literal(token, nodes) {
        (node, None)
    } else if let Some(node) = resolve_function(token, packages) {
        (node, None)
    } else {
        (
            SyntaxNode::UnresolvedIdentifier {
                identifier: token.to_string(),
            },
            Some(CodeError::UnresolvedIdentifier),
        )
    };

    let hash = nodes.insert(node);
    if let Some(error) = maybe_error {
        errors.insert(hash, error);
    }

    hash
}

fn resolve_keyword(name: &str, nodes: &mut Nodes) -> Option<SyntaxNode> {
    match name {
        "apply" => {
            let [expression, argument] = [nodes.insert(SyntaxNode::Empty); 2];
            Some(SyntaxNode::Apply {
                expression,
                argument,
            })
        }
        "self" => Some(SyntaxNode::Recursion),
        _ => None,
    }
}

fn resolve_literal(name: &str, nodes: &mut Nodes) -> Option<SyntaxNode> {
    if let Ok(value) = name.parse() {
        Some(SyntaxNode::Number { value })
    } else {
        match name {
            "fn" => Some(Function.to_node(nodes)),
            "tuple" => Some(Tuple.to_node(nodes)),
            _ => None,
        }
    }
}

fn resolve_function(name: &str, packages: &Packages) -> Option<SyntaxNode> {
    packages
        .resolve_function(name)
        .map(|id| SyntaxNode::ProvidedFunction {
            id,
            name: name.to_string(),
        })
}
