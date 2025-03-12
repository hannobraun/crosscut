use crate::language::code::{NodePath, Type};

use super::{Effect, RuntimeState, Value};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Context {
    pub next: Option<ContextNode>,
    pub active_value: Value,
}

impl Context {
    pub fn advance(&mut self) {
        self.next = self
            .next
            .take()
            .and_then(|next| next.parent.map(|child| *child));
    }

    pub fn unexpected_input(
        &self,
        expected: Type,
        path: NodePath,
    ) -> EvaluateUpdate {
        EvaluateUpdate::UpdateState {
            new_state: RuntimeState::Effect {
                effect: Effect::UnexpectedInput {
                    expected,
                    actual: self.active_value.clone(),
                },
                path,
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextNode {
    pub syntax_node: NodePath,
    pub parent: Option<Box<ContextNode>>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum EvaluateUpdate {
    UpdateState { new_state: RuntimeState },
    PushContext { root: NodePath, active_value: Value },
}
