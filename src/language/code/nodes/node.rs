use std::fmt;

use crate::language::packages::{FunctionId, Packages};

use super::{Children, NodeHash, SiblingIndex};

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Node {
    /// # An empty node
    ///
    /// Empty nodes are placeholders, while the user is editing the code. They
    /// have no effect. They can have up to one child, and evaluate to their
    /// input.
    Empty {
        /// # The child of the empty node, if any
        ///
        /// Since empty nodes are placeholders, they can have any type of child,
        /// or none at all. If they have a child, they evaluate to its output.
        child: Option<NodeHash>,
    },

    /// # A function literal
    ///
    /// Evaluates to a function value.
    LiteralFunction {
        /// # The parameter of the function
        parameter: NodeHash,

        /// # The root node of the function's body
        body: NodeHash,
    },

    /// # A number literal
    ///
    /// As of this writing, there is only one number type supported in the
    /// language (signed, 32-bit integer), so this literal always evaluates to
    /// that. At a future point, it may be able to evaluate to different types
    /// of number value, depending on context.
    ///
    /// Since a number literal takes no input and carries all the information it
    /// needs to evaluate within itself, nodes of this type do not have any
    /// children.
    LiteralNumber {
        /// # The value of the number this literal evaluates to
        ///
        /// ## Implementation Note
        ///
        /// At this point, number literals always evaluate to signed, 32-bit
        /// integers anyway, so that's the type of this field. In the future,
        /// once we support more number types, and more ways of specifying
        /// literals except as decimal numbers, this needs to become more
        /// sophisticated.
        value: i32,
    },

    /// # A tuple literal
    ///
    /// A literal that evaluates to a composite data type, a tuple.
    ///
    /// ## Implementation Note
    ///
    /// Tuples only exist in the language as a placeholder. I (@hannobraun)
    /// expect to replace them with record types at some point.
    LiteralTuple {
        /// # The nodes that determine the values of the tuple literal
        ///
        /// A tuple literal can have an arbitrary number of children, each of
        /// which evaluates to one of the values in the tuple value.
        values: Children,
    },

    /// # The application of a provided function
    ///
    /// Evaluating this note applies a provided function to the active value.
    /// Provided functions are functions that are provided (as the name
    /// suggests) by an entity outside of the language: either the runtime,
    /// which provides intrinsic functions; or the host, which provides host
    /// functions.
    ProvidedFunction {
        /// # The ID of the provided function
        id: FunctionId,

        /// # The child of the node, if any
        ///
        /// If the provided function node has a child, that child's output is
        /// taken as the input of the provided function.
        argument: Option<NodeHash>,
    },

    /// # The recursive application of the current function
    ///
    /// Evaluating the node recursively applies the current function to the
    /// active value.
    Recursion {
        /// # The child of the node, if any
        ///
        /// If the recursion node has a child, that child's output is taken as
        /// the input of the applied function.
        argument: Option<NodeHash>,
    },

    /// # The result of a build error
    Error {
        /// # The original token that couldn't be compiled correctly
        node: String,

        /// # The children of this node
        children: Children,
    },
}

impl Node {
    pub fn has_child_at(
        &self,
        child: &NodeHash,
        sibling_index: &SiblingIndex,
    ) -> bool {
        match self {
            Self::Empty { child: c }
            | Self::ProvidedFunction { argument: c, .. }
            | Self::Recursion { argument: c } => {
                let child_index = SiblingIndex { index: 0 };
                c.as_ref() == Some(child) && sibling_index == &child_index
            }
            Self::LiteralNumber { value: _ } => false,
            Self::LiteralFunction { parameter, body } => {
                let [parameter_index, body_index] =
                    [0, 1].map(|index| SiblingIndex { index });

                parameter == child && sibling_index == &parameter_index
                    || body == child && sibling_index == &body_index
            }
            Self::LiteralTuple { values: children }
            | Self::Error { children, .. } => {
                children.contains_at(child, sibling_index)
            }
        }
    }

    pub fn has_no_children(&self) -> bool {
        match self {
            Self::Empty { child }
            | Self::ProvidedFunction {
                argument: child, ..
            }
            | Self::Recursion { argument: child } => child.is_none(),
            Self::LiteralNumber { value: _ } => true,
            Self::LiteralFunction {
                parameter: NodeHash { .. },
                body: NodeHash { .. },
            } => false,
            Self::LiteralTuple { values: children }
            | Self::Error { children, .. } => children.is_empty(),
        }
    }

    pub fn has_single_child(&self) -> Option<&NodeHash> {
        match self {
            Self::Empty { child }
            | Self::ProvidedFunction {
                argument: child, ..
            }
            | Self::Recursion { argument: child } => child.as_ref(),
            Self::LiteralNumber { value: _ } => None,
            Self::LiteralFunction {
                parameter: NodeHash { .. },
                body: NodeHash { .. },
            } => None,
            Self::LiteralTuple { values: children }
            | Self::Error { children, .. } => children.is_single_child(),
        }
    }

    pub fn to_children(&self) -> Children {
        match self {
            Self::Empty { child }
            | Self::ProvidedFunction {
                argument: child, ..
            }
            | Self::Recursion { argument: child } => Children::new(*child),
            Self::LiteralNumber { value: _ } => Children::new([]),
            Self::LiteralFunction { parameter, body } => {
                Children::new([*parameter, *body])
            }
            Self::LiteralTuple { values: children }
            | Self::Error { children, .. } => children.clone(),
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
    node: &'r Node,
    packages: &'r Packages,
}

impl fmt::Display for NodeDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.node {
            Node::Empty { .. } => {
                write!(f, "")
            }
            Node::LiteralFunction { .. } => {
                write!(f, "fn")
            }
            Node::LiteralNumber { value } => {
                write!(f, "{value}")
            }
            Node::LiteralTuple { .. } => {
                write!(f, "tuple")
            }
            Node::ProvidedFunction { id, .. } => {
                let name = self.packages.function_name_by_id(id);
                write!(f, "{name}")
            }
            Node::Recursion { .. } => {
                write!(f, "self")
            }
            Node::Error { node, .. } => {
                write!(f, "{node}")
            }
        }
    }
}
