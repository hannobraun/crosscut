use crate::language::code::NodePath;

use super::{Effect, Value, ValueWithSource};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeState {
    Running {
        active_value: Value,
        path: Option<NodePath>,
    },
    Effect {
        effect: Effect,
        path: NodePath,
    },
    Finished {
        output: ValueWithSource,
    },
    Error {
        path: NodePath,
    },
}

impl RuntimeState {
    #[cfg(test)]
    pub fn active_value(&self) -> Option<&Value> {
        if let Self::Running { active_value, .. } = self {
            Some(active_value)
        } else {
            None
        }
    }

    pub fn path(&self) -> Option<&NodePath> {
        match self {
            Self::Running { path, .. } => path.as_ref(),
            Self::Effect { path, .. } => Some(path),
            Self::Error { path } => Some(path),
            Self::Finished { .. } => None,
        }
    }
}
