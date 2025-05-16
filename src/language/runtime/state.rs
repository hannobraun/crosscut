use crate::language::code::NodePath;

use super::{Effect, Value};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeState {
    Started,
    Running,
    Effect { effect: Effect, path: NodePath },
    Finished { output: Value },
}

impl RuntimeState {
    #[cfg(test)]
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running { .. })
    }

    pub fn is_effect(&self) -> bool {
        matches!(self, Self::Effect { .. })
    }

    pub fn is_finished(&self) -> bool {
        matches!(self, Self::Finished { .. })
    }

    pub fn path(&self) -> Option<&NodePath> {
        match self {
            Self::Started | Self::Running | Self::Finished { output: _ } => {
                None
            }
            Self::Effect { path, .. } => Some(path),
        }
    }
}
