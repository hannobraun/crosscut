use super::Expression;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Node {
    Empty,
    Expression { expression: Expression },
    UnresolvedIdentifier { name: String },
}
