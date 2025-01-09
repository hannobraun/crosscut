use std::collections::{BTreeMap, BTreeSet};

use super::Hash;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Code {
    pub fragments: BTreeMap<Hash, Fragment>,
    pub root: Vec<Fragment>,
    pub errors: BTreeSet<usize>,
}

impl Code {
    pub fn root(&self) -> impl Iterator<Item = &Fragment> {
        self.root.iter()
    }

    /// # Indicate whether this is complete, meaning contains an expression
    ///
    /// The presence of errors has no significance for the return value of this
    /// function. Its purpose, rather, is to indicate whether the addition of
    /// more fragments would also result in the addition of _more_ errors.
    ///
    /// ## Implementation Note
    ///
    /// Eventually, this method would move to a type representing something like
    /// a branch, or branch body. That it is defined on `Code` is only a
    /// consequence of the current state of development.
    pub fn is_complete(&self) -> bool {
        self.root()
            .any(|fragment| matches!(fragment, Fragment::Expression { .. }))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Fragment {
    Expression { expression: Expression },
    UnexpectedToken { token: Token },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    FunctionCall { target: usize },
    LiteralValue { value: f64 },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Identifier { name: String },
    LiteralNumber { value: f64 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HostFunction {
    pub id: usize,
}
