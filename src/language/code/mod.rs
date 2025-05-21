mod changes;
mod codebase;
mod nodes_uniform;
mod typed_syntax;
mod types;

pub use self::{
    changes::{Changes, NewChangeSet},
    codebase::Codebase,
    nodes_uniform::{
        ChildIndex, LocatedNode, NodeByHash, NodeHash, NodePath, Nodes,
        SyntaxNode,
    },
    typed_syntax::{Apply, Function, Tuple, TypedNode},
    types::{Type, display_tuple},
};

#[cfg(test)]
pub use self::typed_syntax::Expressions;

#[cfg(test)]
mod tests;
