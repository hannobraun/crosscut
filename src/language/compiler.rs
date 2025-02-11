use super::{
    code::{
        CodeError, Codebase, Expression, IntrinsicFunction, Literal, Node,
        NodeKind, NodePath,
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
    codebase: &Codebase,
) -> (Node, Option<CodeError>) {
    let (kind, maybe_error) = if token.is_empty() {
        (NodeKind::Empty, None)
    } else if let Some((node, maybe_err)) =
        resolve_keyword(token, path, codebase)
    {
        (node, maybe_err)
    } else {
        match resolve_function(token, package) {
            Ok(expression) => (NodeKind::Expression { expression }, None),
            Err(candidates) => (
                NodeKind::Error {
                    node: token.to_string(),
                },
                Some(CodeError::UnresolvedIdentifier { candidates }),
            ),
        }
    };

    let node = Node {
        child: codebase.child_of(path).map(|path| *path.hash()),
        kind,
    };

    (node, maybe_error)
}

fn resolve_keyword(
    name: &str,
    path: &NodePath,
    codebase: &Codebase,
) -> Option<(NodeKind, Option<CodeError>)> {
    match name {
        "fn" => match codebase.node_at(path).child {
            Some(_) => Some((
                NodeKind::Expression {
                    expression: Expression::IntrinsicFunction {
                        function: IntrinsicFunction::Literal {
                            literal: Literal::Function,
                        },
                    },
                },
                None,
            )),
            None => Some((
                NodeKind::Error {
                    node: name.to_string(),
                },
                Some(CodeError::FunctionWithoutBody),
            )),
        },
        "self" => Some((NodeKind::Recursion, None)),
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
        (None, Some(function)) => {
            Ok(Expression::IntrinsicFunction { function })
        }
        (None, None) => {
            let candidates = Vec::new();
            Err(candidates)
        }
        (Some(id), Some(function)) => {
            let candidates = vec![
                Expression::HostFunction { id },
                Expression::IntrinsicFunction { function },
            ];
            Err(candidates)
        }
    }
}
