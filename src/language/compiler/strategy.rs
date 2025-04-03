use crate::language::code::{Children, CodeError, NodeHash, NodePath};

use super::token::Token;

pub struct ReplacementStrategy {
    pub next_to_replace: NodePath,
    pub next_token: String,
    pub next_children: Children,
    pub added_nodes: Vec<NodeAddedDuringReplacement>,
}

impl ReplacementStrategy {
    pub fn next_action(&self) -> Token {
        Token {
            text: &self.next_token,
            parent: self.next_to_replace.parent(),
            sibling_index: self.next_to_replace.sibling_index(),
            children: self.next_children.clone(),
        }
    }
}

pub struct NodeAddedDuringReplacement {
    pub replaced: NodePath,
    pub added: NodeHash,
    pub maybe_error: Option<CodeError>,
}
