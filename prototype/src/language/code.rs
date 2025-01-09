use std::collections::BTreeSet;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Code {
    pub fragments: Vec<Fragment>,
    pub errors: BTreeSet<usize>,
}

impl Code {
    /// # Indicate whether this is complete, meaning contains an expression
    ///
    /// The presence of errors has no significance for the return value of this
    /// function. Its purpose, rather, is to indicate whether the addition of
    /// more fragments would also result in the addition of _more_ errors.
    ///
    /// ## Implementation Note
    ///
    /// That the presence of errors has no significance, as documented above, is
    /// not completely true. That is because of a hack that eases the transition
    /// to a more functional evaluation model.
    ///
    /// Eventually, this method would move to a type representing something like
    /// a branch, or branch body. That it is defined on `Code` is only a
    /// consequence of the current state of development.
    pub fn is_complete(&self) -> bool {
        self.fragments
            .iter()
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
