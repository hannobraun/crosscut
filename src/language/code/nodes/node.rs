use std::fmt;

use crate::language::{
    code::{Children, NodeHash, SiblingIndex},
    packages::FunctionId,
};

/// # Structured but untyped representation of a syntax node
///
/// This representation is structured, in the sense that for each type of node
/// the number and role of children is known. But it is untyped, in the sense
/// that all children are a generic `SyntaxNode` again, instead of a more
/// specific type that restricts the child to a valid value.
///
/// ## Implementation Note
///
/// In principle, a typed representation would be preferable, but this would
/// make dealing with syntax nodes in a uniform way much more difficult,
/// significantly increasing the complexity of code that needs to do so.
/// Specifically, there are two reasons for that:
///
/// - The need to deal with multiple types of nodes in some places. For example,
///   storage might require dealing with multiple maps, one per type of node.
/// - The need to make some types that use nodes generic, which in turn requires
///   advanced infrastructure for abstracting over those types.
///
/// Experience has shown that, pending further insights that might this more
/// tenable, a typed representation is not desirable as a base layer, for these
/// reasons.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable)]
pub enum SyntaxNode {
    /// # A node used for adding a value to a list
    ///
    /// ## Implementation Note
    ///
    /// Having this child type of node is a bit weird, and probably not
    /// desirable in the long term. It is only relevant to the editor, and
    /// nothing else.
    ///
    /// As the syntax grows more complex, and what's shown in the editor keeps
    /// diverging from the underlying syntax tree, it might not make sense to
    /// still have this node.
    ///
    /// But so far, this point hasn't been reached. As of this writing, this
    /// is the only such "editor-only" node. And while that is the case,
    /// managing it here makes more sense. It allows the node to piggyback on
    /// top of the existing infrastructure for other nodes.
    AddValue,

    /// # The application of an expression to an argument
    Apply {
        /// # The expression that is being applied to the argument
        expression: NodeHash,

        /// # The argument that the expression is being applied to
        argument: NodeHash,
    },

    /// # Assigns a name to a value
    Binding {
        /// # The name that this binding assigns to the value
        name: String,
    },

    /// # An empty node
    ///
    /// Empty nodes are placeholders, while the user is editing the code. They
    /// have no effect and evaluate to the empty tuple.
    Empty,

    /// # A function literal
    ///
    /// Evaluates to a function value.
    Function {
        /// # The parameter of the function
        parameter: NodeHash,

        /// # The root node of the function's body
        body: NodeHash,
    },

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

        /// # The name of the provided function
        name: String,
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

        /// # A node that can be edited to add values to the tuple
        ///
        /// This is used as a destination for the editor to navigate to, which
        /// it can edit to add a value.
        ///
        /// From the perspective of the syntax tree, this child stays static.
        /// When the user tries to edit it, the editor actually creates a new
        /// child that is then edited, and this one stays as it is.
        add_value: NodeHash,
    },

    /// # An unresolved identifier
    UnresolvedIdentifier {
        /// # The identifier that could not be resolved
        identifier: String,
    },

    /// # An expression that can be used for testing
    ///
    /// It has a name, making it possible to clearly identify it within a test
    /// scenario; and has an arbitrary number of children, making it suitable
    /// for editing tests.
    #[cfg_attr(not(test), allow(dead_code))]
    Test {
        /// # The name of the test expression
        name: String,

        /// # The children of the test expression
        children: Children,
    },
}

impl SyntaxNode {
    pub fn has_child_at(
        &self,
        child: &NodeHash,
        sibling_index: &SiblingIndex,
    ) -> bool {
        match self {
            Self::Apply {
                expression: child_a,
                argument: child_b,
            }
            | Self::Function {
                parameter: child_a,
                body: child_b,
            } => {
                let [index_a, index_b] =
                    [0, 1].map(|index| SiblingIndex { index });

                child == child_a && sibling_index == &index_a
                    || child == child_b && sibling_index == &index_b
            }

            Self::AddValue
            | Self::Binding { .. }
            | Self::Empty
            | Self::Number { value: _ }
            | Self::ProvidedFunction { .. }
            | Self::Recursion
            | Self::UnresolvedIdentifier { .. } => false,

            Self::Tuple { values, add_value } => {
                values.contains_at(child, sibling_index)
                    || add_value == child
                        && sibling_index.index == values.inner.len()
            }

            Self::Test { children, .. } => {
                children.contains_at(child, sibling_index)
            }
        }
    }

    pub fn children(&self) -> Vec<NodeHash> {
        match self {
            Self::Apply {
                expression: a,
                argument: b,
            }
            | Self::Function {
                parameter: a,
                body: b,
            } => vec![*a, *b],

            Self::AddValue
            | Self::Binding { .. }
            | Self::Empty
            | Self::Number { value: _ }
            | Self::ProvidedFunction { .. }
            | Self::Recursion
            | Self::UnresolvedIdentifier { .. } => vec![],

            Self::Tuple { values, add_value } => {
                let mut children = values.inner.clone();
                children.push(*add_value);
                children
            }

            Self::Test { children, .. } => children.inner.clone(),
        }
    }

    pub fn inputs(&self) -> Vec<NodeHash> {
        match self {
            Self::Apply {
                expression: a,
                argument: b,
            } => vec![*a, *b],

            Self::AddValue
            | Self::Binding { .. }
            | Self::Empty
            | Self::Function { .. }
            | Self::Number { value: _ }
            | Self::ProvidedFunction { .. }
            | Self::Recursion
            | Self::UnresolvedIdentifier { .. } => vec![],

            Self::Tuple { values: inputs, .. }
            | Self::Test {
                children: inputs, ..
            } => inputs.inner.clone(),
        }
    }

    pub fn to_token(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for SyntaxNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SyntaxNode::AddValue => {
                write!(f, "")
            }
            SyntaxNode::Apply { .. } => {
                write!(f, "apply")
            }
            SyntaxNode::Binding { name } => {
                write!(f, "{name}")
            }
            SyntaxNode::Empty => {
                write!(f, "")
            }
            SyntaxNode::Function { .. } => {
                write!(f, "fn")
            }
            SyntaxNode::Number { value } => {
                write!(f, "{value}")
            }
            SyntaxNode::ProvidedFunction { id: _, name } => {
                write!(f, "{name}")
            }
            SyntaxNode::Recursion => {
                write!(f, "self")
            }
            SyntaxNode::Tuple { .. } => {
                write!(f, "tuple")
            }
            SyntaxNode::UnresolvedIdentifier { identifier, .. } => {
                write!(f, "{identifier}")
            }
            SyntaxNode::Test { name, .. } => {
                write!(f, "{name}")
            }
        }
    }
}
