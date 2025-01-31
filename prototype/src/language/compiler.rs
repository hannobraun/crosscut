use super::{
    code::{
        CodeError, Codebase, Expression, IntrinsicFunction, Location, Node,
    },
    host::Host,
    runtime::Value,
};

pub fn compile(
    token: &str,
    location: &Location,
    host: &Host,
    codebase: &mut Codebase,
) {
    codebase.clear_error(location);

    let node = if token.is_empty() {
        Node::Empty
    } else if let Ok(value) = token.parse() {
        Node::Expression {
            expression: Expression::IntrinsicFunction {
                function: IntrinsicFunction::Literal {
                    value: Value::Integer { value },
                },
            },
        }
    } else {
        // The token is an identifier.

        let host_function = host.function_id_by_name(token);
        let intrinsic_function = IntrinsicFunction::resolve(token);

        match (host_function, intrinsic_function) {
            (Some(id), None) => Node::Expression {
                expression: Expression::HostFunction { id },
            },
            (None, Some(function)) => Node::Expression {
                expression: Expression::IntrinsicFunction { function },
            },
            (None, None) => {
                let candidates = Vec::new();
                emit_unresolved_identifier_error(
                    token, *location, candidates, codebase,
                )
            }
            (Some(id), Some(function)) => {
                let candidates = vec![
                    Expression::HostFunction { id },
                    Expression::IntrinsicFunction { function },
                ];
                emit_unresolved_identifier_error(
                    token, *location, candidates, codebase,
                )
            }
        }
    };

    codebase.replace_node(location, node);
}

fn emit_unresolved_identifier_error(
    token: &str,
    location: Location,
    candidates: Vec<Expression>,
    codebase: &mut Codebase,
) -> Node {
    codebase
        .insert_error(location, CodeError::UnresolvedIdentifier { candidates });

    Node::UnresolvedIdentifier {
        name: token.to_string(),
    }
}
