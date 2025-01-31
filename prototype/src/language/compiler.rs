use super::{
    code::{Codebase, Expression, IntrinsicFunction, Node},
    host::Host,
    runtime::Value,
};

pub fn compile(token: &str, host: &Host, _: &mut Codebase) -> Node {
    if token.is_empty() {
        Node::Empty
    } else if let Ok(value) = token.parse() {
        Node::Expression {
            expression: Expression::IntrinsicFunction {
                function: IntrinsicFunction::Literal {
                    value: Value::Integer { value },
                },
            },
        }
    } else if let Some(id) = host.function_id_by_name(token) {
        Node::Expression {
            expression: Expression::HostFunction { id },
        }
    } else if token == "identity" {
        Node::Expression {
            expression: Expression::IntrinsicFunction {
                function: IntrinsicFunction::Identity,
            },
        }
    } else {
        Node::UnresolvedIdentifier {
            name: token.to_string(),
        }
    }
}
