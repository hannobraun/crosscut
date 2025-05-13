use crate::language::{
    code::{Children, CodeError, Errors, NodeHash, Nodes, SyntaxNode},
    packages::Packages,
};

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
    } else {
        match resolve_function(token, packages) {
            Some(node) => (node, None),
            None => (
                SyntaxNode::UnresolvedIdentifier {
                    identifier: token.to_string(),
                },
                Some(CodeError::UnresolvedIdentifier),
            ),
        }
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
            "fn" => {
                let [parameter, body] = [nodes.insert(SyntaxNode::Empty); 2];
                Some(SyntaxNode::Function { parameter, body })
            }
            "tuple" => {
                let values = Children::new([]);
                Some(SyntaxNode::Tuple { values })
            }
            _ => None,
        }
    }
}

fn resolve_function(name: &str, packages: &Packages) -> Option<SyntaxNode> {
    packages
        .resolve_function(name)
        .map(|id| SyntaxNode::ProvidedFunction { id })
}
