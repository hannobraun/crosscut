use super::{
    code::{
        CodeError, Codebase, Expression, IntrinsicFunction, Literal, Node,
        NodeHash, NodePath,
    },
    packages::Package,
};

pub fn compile_and_replace(
    token: &str,
    path: &mut NodePath,
    package: &Package,
    codebase: &mut Codebase,
) {
    let (node, maybe_error) = compile_token(token, path, package, codebase);

    *path = codebase.replace_node(path, node);
    if let Some(error) = maybe_error {
        codebase.insert_error(*path, error);
    }
}

fn compile_token(
    token: &str,
    path: &mut NodePath,
    package: &Package,
    codebase: &mut Codebase,
) -> (Node, Option<CodeError>) {
    let node = codebase.node_at(path);
    let child = node.child().copied();

    let (node, maybe_error) = if token.is_empty() {
        (Node::Empty { child }, None)
    } else if let Some((node, maybe_err)) =
        resolve_keyword(token, path, child, codebase)
    {
        (node, maybe_err)
    } else {
        match resolve_function(token, package) {
            Ok(expression) => (Node::Expression { expression, child }, None),
            Err(candidates) => (
                Node::Error {
                    node: token.to_string(),
                    child,
                },
                Some(CodeError::UnresolvedIdentifier { candidates }),
            ),
        }
    };

    (node, maybe_error)
}

fn resolve_keyword(
    name: &str,
    path: &mut NodePath,
    child: Option<NodeHash>,
    codebase: &mut Codebase,
) -> Option<(Node, Option<CodeError>)> {
    match name {
        "fn" => {
            let child = if codebase.node_at(path).child().is_none() {
                let child = codebase
                    .insert_as_child_of(path, Node::Empty { child: None });
                *path = codebase.latest_version_of(*path);

                Some(*child.hash())
            } else {
                child
            };

            Some((
                Node::Expression {
                    expression: Expression::IntrinsicFunction {
                        intrinsic: IntrinsicFunction::Literal {
                            literal: Literal::Function,
                        },
                    },
                    child,
                },
                None,
            ))
        }
        "self" => Some((Node::Recursion { child }, None)),
        _ => None,
    }
}

fn resolve_function(
    name: &str,
    package: &Package,
) -> Result<Expression, Vec<Expression>> {
    let host_function = package.resolve_function(name);
    let intrinsic_function = IntrinsicFunction::resolve(name);

    match (host_function, intrinsic_function) {
        (Some(id), None) => Ok(Expression::HostFunction { id }),
        (None, Some(intrinsic)) => {
            Ok(Expression::IntrinsicFunction { intrinsic })
        }
        (None, None) => {
            let candidates = Vec::new();
            Err(candidates)
        }
        (Some(id), Some(intrinsic)) => {
            let candidates = vec![
                Expression::HostFunction { id },
                Expression::IntrinsicFunction { intrinsic },
            ];
            Err(candidates)
        }
    }
}
