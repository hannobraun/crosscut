use crate::language::code::{Children, NodePath};

pub struct ReplacementStrategy {
    pub next_to_replace: NodePath,
    pub next_token: String,
    pub next_children: Children,
}
