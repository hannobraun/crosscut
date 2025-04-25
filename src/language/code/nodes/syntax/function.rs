use crate::language::code::NodeHash;

use super::Expression;

/// # A function
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable)]
pub struct Function {
    /// # The parameter of the function
    pub parameter: NodeHash<Expression>,

    /// # The root node of the function's body
    pub body: NodeHash<Expression>,
}
