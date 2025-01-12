#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CodeError {
    IntegerOverflow,
    MissingArgument,
    UnexpectedToken,
    UnresolvedIdentifier,
}
