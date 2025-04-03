use crate::language::code::{Children, CodeError, NodeHash, NodePath};

pub struct ReplacementStrategy {
    pub next_to_replace: NodePath,
    pub next_token: String,
    pub next_children: Children,
    pub added_nodes: Vec<NodeAddedDuringReplacement>,
}

pub struct NodeAddedDuringReplacement {
    pub path_of_replaced_node: NodePath,
    pub hash_of_added_node: NodeHash,
    pub error_of_added_node: Option<CodeError>,
}
