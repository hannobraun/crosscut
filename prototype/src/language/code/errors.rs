#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CodeError {
    MissingArgument,
    UnexpectedToken,
    UnresolvedIdentifier,
}
