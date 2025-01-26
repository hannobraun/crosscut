use crosscut_runtime::Value;

use super::function::Function;

/// # An expression within a function
#[derive(
    Clone,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    serde::Deserialize,
    serde::Serialize,
    udigest::Digestable,
)]
pub enum Expression {
    /// # An identifier
    ///
    /// Can refer to a binding or function.
    Identifier {
        /// # The name of the identifier
        name: String,
    },

    /// # A number literal
    LiteralNumber {
        /// The number defined by this literal
        value: Value,
    },

    /// # A local function
    LocalFunction {
        /// # The local function
        function: Function,
    },
}

impl Expression {
    /// # Convert the expression into a local function
    ///
    /// Returns `None`, if this expression is not a local function.
    pub fn as_local_function(&self) -> Option<&Function> {
        let Expression::LocalFunction { function } = self else {
            return None;
        };

        Some(function)
    }
}
