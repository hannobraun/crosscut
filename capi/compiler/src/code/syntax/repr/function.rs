use capi_runtime::Value;

use crate::code::{IndexMap, Signature};

use super::{expression::Expression, types::SyntaxType};

/// # A function that has a name
///
/// Named functions are defined in the top-level context. Functions that are
/// defined locally within other functions do not have a name.
#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct NamedFunction {
    /// # The name of the function
    pub name: String,

    /// # The function
    pub inner: Function,
}

/// # A function
///
/// Functions can be named (see [`NamedFunction`]) or anonymous. Local functions
/// that are defined within other functions are anonymous.
#[derive(
    Clone,
    Debug,
    Default,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    serde::Deserialize,
    serde::Serialize,
    udigest::Digestable,
)]
pub struct Function {
    /// # The branches of this function
    ///
    /// A function is made up of one or more branches. When a function is
    /// called, its arguments are matched against the parameters of each branch,
    /// until one branch matches. This branch is then evaluated.
    pub branches: IndexMap<Branch>,
}

/// # A branch within a function
///
/// A function has zero or more branches. When the function is called, the
/// arguments are matched against its branches. The first branch whose
/// parameters match the arguments is executed.
#[derive(
    Clone,
    Debug,
    Default,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    serde::Deserialize,
    serde::Serialize,
    udigest::Digestable,
)]
pub struct Branch {
    /// # The parameters of the branch
    pub parameters: IndexMap<Parameter>,

    /// # The body of the branch
    pub body: IndexMap<Member>,
}

/// # A parameter
///
/// Parameters match against the arguments of a function.
#[derive(
    Clone,
    Debug,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    serde::Deserialize,
    serde::Serialize,
    udigest::Digestable,
)]
pub enum Parameter {
    /// # An identifier
    ///
    /// Identifiers match against any value. They are used to assign a local
    /// name to a value.
    Binding(Binding),

    /// # A literal
    ///
    /// Literals only match against values that are equal to their `value`
    /// field. They are used to select which branch is executed, based on the
    /// arguments of the function call.
    Literal {
        /// # The value that an argument is matched against
        value: Value,
    },
}

impl Parameter {
    /// # Convert this parameter into a binding
    ///
    /// Returns `None`, if the parameter is not a binding.
    pub fn as_binding(&self) -> Option<&Binding> {
        let Self::Binding(binding) = self else {
            return None;
        };

        Some(binding)
    }
}

/// # A binding
///
/// A binding is a value that has been bound to a name, locally within a branch.
#[derive(
    Clone,
    Debug,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    serde::Deserialize,
    serde::Serialize,
    udigest::Digestable,
)]
pub struct Binding {
    /// # The name of the binding
    pub name: String,
}

/// # A part of a branch's body
#[derive(
    Clone,
    Debug,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    serde::Deserialize,
    serde::Serialize,
    udigest::Digestable,
)]
pub enum Member {
    /// # A code comment
    Comment {
        /// # The text of the comment
        text: Vec<String>,
    },

    /// # The syntax node is an expression
    Expression {
        /// # The expression
        expression: Expression,

        /// # The optional type annotation that applies to the expression
        signature: Option<Signature<SyntaxType>>,
    },
}

impl Member {
    /// # Convert this instance of [`Member`] into an [`Expression`]
    ///
    /// Returns `None`, if the member is not an expression.
    pub fn as_expression(&self) -> Option<&Expression> {
        if let Self::Expression { expression, .. } = self {
            Some(expression)
        } else {
            None
        }
    }
}
