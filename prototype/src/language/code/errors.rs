use super::FragmentError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CodeError {
    Fragment { err: FragmentError },
    MissingArgument,
}
