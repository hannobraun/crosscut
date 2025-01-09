use std::collections::{BTreeMap, BTreeSet};

use super::Hash;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Code {
    pub fragments: BTreeMap<Hash, Fragment>,
    pub root: Vec<Fragment>,
    pub errors: BTreeSet<usize>,
}

impl Code {
    pub fn fragment_at(&self, index: usize) -> Option<&Fragment> {
        self.root.get(index)
    }

    pub fn root(&self) -> impl Iterator<Item = &Fragment> {
        self.root.iter()
    }

    pub fn push(&mut self, fragment: Fragment) {
        self.root.push(fragment);
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

#[derive(Clone, Debug, PartialEq, udigest::Digestable)]
pub enum Fragment {
    Expression { expression: Expression },
    UnexpectedToken { token: Token },
}

#[derive(Clone, Debug, PartialEq, udigest::Digestable)]
pub enum Expression {
    FunctionCall { target: usize },
    LiteralValue { value: u32 },
}

#[derive(Clone, Debug, PartialEq, udigest::Digestable)]
pub enum Token {
    Identifier { name: String },
    LiteralNumber { value: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HostFunction {
    pub id: usize,
}
