use crate::language::code::NodePath;

use super::{Effect, Value};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeState {
    Started,
    Running { path: Option<NodePath> },
    Effect { effect: Effect, path: NodePath },
    Finished { output: Value },
    Error { path: NodePath },
}

impl RuntimeState {
    #[cfg(test)]
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running { .. })
    }

    pub fn is_effect(&self) -> bool {
        matches!(self, Self::Effect { .. })
    }

    #[cfg(test)]
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }

    pub fn path(&self) -> Option<&NodePath> {
        match self {
            Self::Started => None,
            Self::Running { path, .. } => path.as_ref(),
            Self::Effect { path, .. } => Some(path),
            Self::Error { path } => Some(path),
            Self::Finished { .. } => None,
        }
    }
}
