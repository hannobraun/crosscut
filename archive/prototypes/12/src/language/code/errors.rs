#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CodeError {
    IntegerOverflow,
    MissingArgument,
    MultiResolvedIdentifier,
    UnexpectedToken,
    UnresolvedIdentifier,
}
