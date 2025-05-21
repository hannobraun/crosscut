use std::fmt;

use crate::{
    language::code::NodeHash,
    util::form::{Ref, RefMut},
};

use super::Children;

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
    /// This approach to adding children is not desirable, for at least two
    /// reasons:
    ///
    /// 1. Having a node here that has no meaning to that part of the code, and
    ///    is _only_ used by the editor, is a bit weird. At the very least, it
    ///    might be better to have some kind of "virtual" node that only the
    ///    editor deals with.
    /// 2. From a UX perspective, this "add child" node is weird. It might be
    ///    more intuitive, if navigating away from a node using "space" or
    ///    "enter" would create a new child or sibling where possible, and just
    ///    navigate to the next node where not.
    ///
    /// But both of those approaches are more complicated than the current one,
    /// which has turned out to be a quick win. Specifically, I found there to
    /// be the following hurdles:
    ///
    /// 1. A virtual editor-only node would not benefit from the infrastructure
    ///    that we already have for identifying nodes, like `NodePath`. Maybe we
    ///    need a solution for this anyway, as the syntax grows more complex and
    ///    what's shown in the editor diverges from the structure of the syntax
    ///    tree. But so far, this hasn't been necessary.
    /// 2. Adding new children or siblings or just navigating to the next node,
    ///    depending on the context, might be the better solution, but it's not
    ///    trivial to implement. There are many edge cases to consider to get it
    ///    to a good enough state.
    ///
    /// Neither solution seems worth paying the price for right now, so this
    /// weird node is what we got for the time being.
    AddNode,

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
    ///
    /// ## Implementation Note
    ///
    /// Having a dedicated type of syntax node just to serve as a placeholder is
    /// undesirable. Just using an empty tuple (or later on, an empty record)
    /// instead would be better. Or in some cases, like apply nodes, maybe the
    /// identity function as the placeholder for the function.
    ///
    /// But right now, there are multiple problems with this:
    ///
    /// - The syntax for tuples is quite cumbersome. Once records are a thing,
    ///   and their syntax is fully implemented, I expect the empty record to be
    ///   quite compact (`{}`), but right now, this is not the case.
    /// - To make this work well, the editor would need to distinguish between
    ///   nodes the user has explicitly written, and those that were generated
    ///   as placeholders. Otherwise, the user would have to manually delete
    ///   these placeholders to type something new.
    ///
    /// These problems are definitely not insurmountable, but solving them would
    /// require resources that are, for the time being, better spent elsewhere.
    /// So for now, having a dedicated node as the placeholder, seems like a
    /// practical solution.
    Empty,

    /// # A function literal
    ///
    /// Evaluates to a function value.
    Function {
        /// # The parameter of the function
        parameter: NodeHash,

        /// # The root node of the function's body
        body: Vec<NodeHash>,
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
        values: Vec<NodeHash>,

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
}

impl SyntaxNode {
    pub fn children(&self) -> Children<Ref> {
        let mut hashes = Vec::new();

        match self {
            Self::Apply {
                expression,
                argument,
            } => hashes.extend([expression, argument]),

            Self::AddNode
            | Self::Binding { .. }
            | Self::Empty
            | Self::Identifier { .. }
            | Self::Number { value: _ }
            | Self::Recursion => {}

            Self::Function { parameter, body } => {
                hashes.push(parameter);
                hashes.extend(body);
            }

            Self::Tuple { values, add_value } => {
                hashes.extend(values);
                hashes.push(add_value);
            }
        }

        Children { hashes }
    }

    pub fn children_mut(&mut self) -> Children<RefMut> {
        let mut hashes = Vec::new();

        match self {
            Self::Apply {
                expression,
                argument,
            } => hashes.extend([expression, argument]),

            Self::AddNode
            | Self::Binding { .. }
            | Self::Empty
            | Self::Identifier { .. }
            | Self::Number { value: _ }
            | Self::Recursion => {}

            Self::Function { parameter, body } => {
                hashes.push(parameter);
                hashes.extend(body);
            }

            Self::Tuple { values, add_value } => {
                hashes.extend(values);
                hashes.push(add_value);
            }
        }

        Children { hashes }
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
        }
    }
}
