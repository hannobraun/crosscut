use std::fmt;

use crate::language::{
    code::{Children, NodeHash, SiblingIndex},
    compiler::Apply,
};

use super::ChildOwned;

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
    /// # A node used for adding a child to its parent
    ///
    /// This is used for editing, so the user has a node to navigate to when
    /// they want to add a child.
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
    /// top of the existing infrastructure for addressing nodes.
    AddNode,

    /// # The application of an expression to an argument
    Apply {
        /// # The expression that is being applied to the argument
        expression: ChildOwned,

        /// # The argument that the expression is being applied to
        argument: ChildOwned,
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
        body: Children,
    },

    /// # An identifier
    Identifier {
        /// # The name of the identifier
        name: String,
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
                expression,
                argument,
            } => {
                let apply = Apply {
                    expression: *expression.hash(),
                    argument: *argument.hash(),
                };
                apply.expression().is(child, sibling_index)
                    || apply.argument().is(child, sibling_index)
            }

            Self::AddNode
            | Self::Binding { .. }
            | Self::Empty
            | Self::Identifier { .. }
            | Self::Number { value: _ }
            | Self::Recursion => false,

            Self::Function { parameter, body } => {
                child == parameter && sibling_index.index == 0
                    || body.contains_at(child, sibling_index, 1)
            }

            Self::Tuple { values, add_value } => {
                values.contains_at(child, sibling_index, 0)
                    || add_value == child
                        && sibling_index.index == values.inner.len()
            }

            Self::Test { children, .. } => {
                children.contains_at(child, sibling_index, 0)
            }
        }
    }

    pub fn children(&self) -> Vec<NodeHash> {
        match self {
            Self::Apply {
                expression,
                argument,
            } => vec![*expression.hash(), *argument.hash()],

            Self::AddNode
            | Self::Binding { .. }
            | Self::Empty
            | Self::Identifier { .. }
            | Self::Number { value: _ }
            | Self::Recursion => vec![],

            Self::Function { parameter, body } => {
                let mut children = vec![*parameter];
                children.extend(body.inner.iter().copied());
                children
            }

            Self::Tuple { values, add_value } => {
                let mut children = values.inner.clone();
                children.push(*add_value);
                children
            }

            Self::Test { children, .. } => children.inner.clone(),
        }
    }

    pub fn to_token(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for SyntaxNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SyntaxNode::AddNode => {
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
            SyntaxNode::Identifier { name, .. } => {
                write!(f, "{name}")
            }
            SyntaxNode::Number { value } => {
                write!(f, "{value}")
            }
            SyntaxNode::Recursion => {
                write!(f, "self")
            }
            SyntaxNode::Tuple { .. } => {
                write!(f, "tuple")
            }
            SyntaxNode::Test { name, .. } => {
                write!(f, "{name}")
            }
        }
    }
}
