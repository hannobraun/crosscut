use crate::language::code::NodePath;

pub struct ReplacementStrategy {
    pub next_to_replace: NodePath,
    pub next_token: String,
}
