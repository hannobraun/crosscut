mod changes;
mod codebase;
mod errors;
mod intrinsics;
mod nodes;
mod types;

pub use self::{
    changes::{Changes, NewChangeSet},
    codebase::Codebase,
    errors::{CandidateForResolution, CodeError, Errors, Literal},
    intrinsics::IntrinsicFunction,
    nodes::{
        ChildOfExpression, Children, Expression, LocatedNode, NodeHash,
        NodePath, Nodes, SiblingIndex,
    },
    types::{Type, display_tuple},
};

#[cfg(test)]
pub use self::nodes::Borrowed;

#[cfg(test)]
mod tests;
