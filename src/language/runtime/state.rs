use crate::language::code::NodePath;

use super::{Effect, ValueWithSource};

#[cfg(test)]
use super::Value;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvaluatorState {
    Running { active_value: ValueWithSource },
    Effect { effect: Effect, path: NodePath },
    Finished { output: ValueWithSource },
    Error { path: NodePath },
}

impl EvaluatorState {
    #[cfg(test)]
    pub fn active_value(&self) -> Option<Value> {
        if let Self::Running { active_value, .. } = self {
            Some(active_value.inner.clone())
        } else {
            None
        }
    }

    pub fn path(&self) -> Option<&NodePath> {
        match self {
            Self::Running { active_value } => active_value.source.as_ref(),
            Self::Effect { path, .. } => Some(path),
            Self::Error { path } => Some(path),
            Self::Finished { .. } => None,
        }
    }
}
