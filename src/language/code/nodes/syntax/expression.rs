use std::fmt;

use crate::language::{
    code::{Children, NodeHash, SiblingIndex},
    packages::{FunctionId, Packages},
};

use super::Function;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable)]
pub enum Expression {
    /// # The application of a function
    Apply {
        /// # The function that is being applied
        function: NodeHash<Expression>,

        /// # The argument that the function is applied to
        argument: NodeHash<Expression>,
    },

    /// # An empty node
    ///
    /// Empty nodes are placeholders, while the user is editing the code. They
    /// have no effect and evaluate to the empty tuple.
    Empty,

    /// # A function literal
    ///
    /// Evaluates to a function value.
    Function { function: Function },

    /// # A number literal
    Number {
        /// # The value of the number this literal evaluates to
        ///
        /// ## Implementation Note
        ///
        /// At this point, number literals always evaluate to signed, 32-bit
        /// integers, so that's the type of this field. In the future, once we
        /// support more number types, and more ways of specifying literals
        /// except as decimal numbers, this needs to become more sophisticated.
        value: i32,
    },

    /// # The application of a provided function
    ///
    /// Evaluating this note applies a provided function to the active value.
    /// Provided functions are functions that are provided (as the name
    /// suggests) by an entity outside of the language: either the runtime,
    /// which provides intrinsic functions; or the host, which provides host
    /// functions.
    ///
    /// ## Implementation Note
    ///
    /// Under the hood, this is handled in the following way:
    ///
    /// - It triggers an effect.
    /// - This effect is processed by the runtime or host (the provider).
    /// - The provider determined which function is being called, via the ID.
    /// - The provider checks that the correct argument has been provided.
    /// - The provider evaluates the function and returns the value.
    ///
    /// If we replaced the concept of intrinsic function with a special
    /// `trigger` expression that triggers a provided value as an effect, that
    /// would be better in at least the following ways:
    ///
    /// - The provider would just need to check which value was triggered.
    ///   - This would remove the opportunity to provide an invalid argument.
    ///   - It would also simplify the handling on the provider side.
    /// - It would mesh well with the later introduction of algebraic effects,
    ///   which would require such a feature anyway.
    /// - Since a `trigger` expression would be required anyway, this would
    ///   remove the redundant concept of provided functions.
    ///
    /// From an API perspective, this `trigger` expression could be wrapped into
    /// a Crosscut function, preserving the same API.
    ///
    /// As of this writing, this is not possible. The language doesn't support
    /// powerful enough values yet.
    ProvidedFunction {
        /// # The ID of the provided function
        id: FunctionId,
    },

    /// # The recursive application of the current function
    ///
    /// Evaluating the node recursively applies the current function to the
    /// active value.
    Recursion,

    /// # A tuple literal
    ///
    /// A literal that evaluates to a composite data type, a tuple.
    ///
    /// ## Implementation Note
    ///
    /// Tuples only exist in the language as a placeholder. I (@hannobraun)
    /// expect to replace them with record types at some point.
    Tuple {
        /// # The nodes that determine the values of the tuple literal
        ///
        /// A tuple literal can have an arbitrary number of children, each of
        /// which evaluates to one of the values in the tuple value.
        values: Children,
    },

    /// # The result of a build error
    Error {
        /// # The original token that couldn't be compiled correctly
        node: String,

        /// # The children of this node
        children: Children,
    },
}

impl Expression {
    pub fn has_child_at(
        &self,
        child: &NodeHash<Expression>,
        sibling_index: &SiblingIndex,
    ) -> bool {
        match self {
            Self::Apply {
                function: child_a,
                argument: child_b,
            }
            | Self::Function {
                function:
                    Function {
                        parameter: child_a,
                        body: child_b,
                    },
            } => {
                let [index_a, index_b] =
                    [0, 1].map(|index| SiblingIndex { index });

                child == child_a && sibling_index == &index_a
                    || child == child_b && sibling_index == &index_b
            }

            Self::Empty
            | Self::Number { value: _ }
            | Self::ProvidedFunction { .. }
            | Self::Recursion => false,

            Self::Tuple { values: children } => {
                children.contains_at(child.raw(), sibling_index)
            }

            Self::Error { children, .. } => {
                children.contains_at(child.raw(), sibling_index)
            }
        }
    }

    pub fn has_no_children(&self) -> bool {
        match self {
            Self::Apply {
                function: NodeHash { .. },
                argument: NodeHash { .. },
            }
            | Self::Function {
                function:
                    Function {
                        parameter: NodeHash { .. },
                        body: NodeHash { .. },
                    },
            } => false,

            Self::Empty
            | Self::Number { value: _ }
            | Self::ProvidedFunction { .. }
            | Self::Recursion => true,

            Self::Tuple { values: children } | Self::Error { children, .. } => {
                children.is_empty()
            }
        }
    }

    pub fn has_single_child(&self) -> Option<&NodeHash<Expression>> {
        match self {
            Self::Apply { .. }
            | Self::Empty
            | Self::Number { value: _ }
            | Self::Function {
                function:
                    Function {
                        parameter: NodeHash { .. },
                        body: NodeHash { .. },
                    },
            }
            | Self::ProvidedFunction { .. }
            | Self::Recursion => None,

            Self::Tuple { values: children } | Self::Error { children, .. } => {
                children.is_single_child()
            }
        }
    }

    pub fn to_children(&self) -> Children {
        match self {
            Self::Apply {
                function: a,
                argument: b,
            }
            | Self::Function {
                function:
                    Function {
                        parameter: a,
                        body: b,
                    },
            } => Children::new([*a, *b]),

            Self::Empty
            | Self::Number { value: _ }
            | Self::ProvidedFunction { .. }
            | Self::Recursion => Children::new([]),

            Self::Tuple { values: children } | Self::Error { children, .. } => {
                children.clone()
            }
        }
    }

    pub fn to_token(&self, packages: &Packages) -> String {
        self.display(packages).to_string()
    }

    pub fn display<'r>(&'r self, packages: &'r Packages) -> NodeDisplay<'r> {
        NodeDisplay {
            node: self,
            packages,
        }
    }
}

pub struct NodeDisplay<'r> {
    node: &'r Expression,
    packages: &'r Packages,
}

impl fmt::Display for NodeDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.node {
            Expression::Apply { .. } => {
                write!(f, "apply")
            }
            Expression::Empty => {
                write!(f, "")
            }
            Expression::Function { .. } => {
                write!(f, "fn")
            }
            Expression::Number { value } => {
                write!(f, "{value}")
            }
            Expression::ProvidedFunction { id, .. } => {
                let name = self.packages.function_name_by_id(id);
                write!(f, "{name}")
            }
            Expression::Recursion => {
                write!(f, "self")
            }
            Expression::Tuple { .. } => {
                write!(f, "tuple")
            }
            Expression::Error { node, .. } => {
                write!(f, "{node}")
            }
        }
    }
}
