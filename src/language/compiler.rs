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
    path: &NodePath,
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
    path: &NodePath,
    child: Option<NodeHash>,
    codebase: &mut Codebase,
) -> Option<(Node, Option<CodeError>)> {
    match name {
        "fn" => {
            // This isn't quite right. Functions with an empty body are a
            // completely reasonable thing to design. They'd just do nothing,
            // returning the active value unchanged.
            //
            // And if we didn't have to handle the error case here, then this
            // code could move into `IntrinsicFunction::resolve`, where it fits
            // better.
            //
            // But we would have to handle the potential emptiness of a function
            // somehow:
            //
            // - Either `Value::Function` would have to have an optional body.
            //   I've tried this, and don't like it, because it increases
            //   complexity.
            // - Or we'd need some kind of leaf node that can fill in as an
            //   empty body. But that's a larger change, and then we have to
            //   deal with nodes that the editor shouldn't display or edit.
            //
            // Due to these problems, I'd like to leave this as-is, for now. And
            // I think this situation is only temporary anyway: At some point,
            // functions can have an arbitrary number. Zero could be a valid
            // number of branches then, and the error case here would no longer
            // be relevant.
            if codebase.node_at(path).child().is_none() {
                Some((
                    Node::Error {
                        node: name.to_string(),
                        child,
                    },
                    Some(CodeError::FunctionWithoutBody),
                ))
            } else {
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
