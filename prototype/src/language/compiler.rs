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
        let intrinsic_function = match token {
            "identity" => Some(IntrinsicFunction::Identity),
            _ => None,
        };

        if let Some(id) = host_function {
            Node::Expression {
                expression: Expression::HostFunction { id },
            }
        } else if let Some(function) = intrinsic_function {
            Node::Expression {
                expression: Expression::IntrinsicFunction { function },
            }
        } else {
            codebase.insert_error(*location, CodeError::UnresolvedIdentifier);

            Node::UnresolvedIdentifier {
                name: token.to_string(),
            }
        }
    };

    codebase.replace_node(location, node);
}
