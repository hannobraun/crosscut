use crate::language::{
    code::{Children, CodeError, Errors, Literal, NodeHash, Nodes, SyntaxNode},
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
    } else if let Some(literal) = resolve_literal(token, nodes) {
        match literal {
            Literal::Function => {
                let [parameter, body] = [nodes.insert(SyntaxNode::Empty); 2];

                (SyntaxNode::Function { parameter, body }, None)
            }
            Literal::Integer { value } => (SyntaxNode::Number { value }, None),
            Literal::Tuple => {
                let values = Children::new([]);

                (SyntaxNode::Tuple { values }, None)
            }
        }
    } else {
        match resolve_function(token, packages, nodes) {
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

fn resolve_function(
    name: &str,
    packages: &Packages,
    _: &mut Nodes,
) -> Option<SyntaxNode> {
    let provided_function = packages.resolve_function(name);

    provided_function.map(|id| SyntaxNode::ProvidedFunction { id })
}

fn resolve_literal(name: &str, _: &mut Nodes) -> Option<Literal> {
    if let Ok(value) = name.parse() {
        Some(Literal::Integer { value })
    } else {
        match name {
            "fn" => Some(Literal::Function),
            "tuple" => Some(Literal::Tuple),
            _ => None,
        }
    }
}
