use crate::language::code::NodePath;

use super::{Effect, Value};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum RuntimeState {
    #[default]
    Started,

    Running,

    Effect {
        effect: Effect,
        path: NodePath,
    },

    Finished {
        output: Value,
    },
}

impl RuntimeState {
    pub fn is_started(&self) -> bool {
        matches!(self, Self::Started)
    }

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
