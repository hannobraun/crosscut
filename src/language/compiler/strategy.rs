use crate::language::code::{Children, CodeError, NodeHash, NodePath};

pub struct ReplacementStrategy {
    pub next_to_replace: NodePath,
    pub next_token: String,
    pub next_children: Children,
    pub added_nodes: Vec<NodeAddedDuringReplacement>,
}

pub type NodeAddedDuringReplacement = (NodePath, NodeHash, Option<CodeError>);
