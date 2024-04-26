use std::collections::BTreeMap;

use crate::{
    builtins, evaluator::EvaluatorState, source_map::SourceMap, DebugEvent,
    Evaluator, Functions, InstructionAddress, SourceLocation, Value,
};

#[derive(Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct Program {
    pub functions: Functions,
    pub source_map: SourceMap,
    pub breakpoints: BTreeMap<InstructionAddress, bool>,
    pub evaluator: Evaluator,
    pub state: ProgramState,
    pub entry_address: InstructionAddress,
}

impl Program {
    pub fn push(&mut self, arguments: impl IntoIterator<Item = Value>) {
        for value in arguments {
            self.evaluator.data_stack.push(value);
        }
    }

    pub fn reset(&mut self) {
        self.evaluator.next_instruction = self.entry_address;
    }

    pub fn apply_debug_event(&mut self, event: DebugEvent) {
        match event {
            DebugEvent::ToggleBreakpoint {
                address,
                location:
                    SourceLocation {
                        function,
                        index: line,
                    },
            } => {
                let breakpoint =
                    self.breakpoints.entry(address).or_insert(false);
                *breakpoint = !*breakpoint;

                let line: usize = line.try_into().unwrap();

                let function = self
                    .functions
                    .inner
                    .iter_mut()
                    .find(|f| f.name == function)
                    .unwrap();
                let syntax_element = function.syntax.get_mut(line).unwrap();

                syntax_element.breakpoint = !syntax_element.breakpoint;
            }
        }
    }

    pub fn step(&mut self, mem: &mut [u8]) -> ProgramState {
        let state = self.step_inner(mem);
        self.state = state.clone();
        state
    }

    fn step_inner(&mut self, mem: &mut [u8]) -> ProgramState {
        if let Some(location) = self.breakpoint_set_for_next_instruction() {
            return ProgramState::Paused { location };
        }

        self.evaluator.step(mem).into()
    }

    fn breakpoint_set_for_next_instruction(&self) -> Option<SourceLocation> {
        let next_location = self
            .source_map
            .address_to_location(&self.evaluator.next_instruction)?;

        let function = self
            .functions
            .inner
            .iter()
            .find(|function| function.name == next_location.function)
            .unwrap();
        let expression = function
            .syntax
            .iter()
            .find(|expression| expression.location == next_location)
            .unwrap();

        if expression.breakpoint {
            return Some(next_location);
        }

        None
    }
}

#[derive(
    Clone, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize,
)]
pub enum ProgramState {
    Running,

    Paused {
        /// The location at which the program is paused
        location: SourceLocation,
    },

    #[default]
    Finished,

    Error {
        err: builtins::Error,
        instruction: InstructionAddress,
    },
}

impl ProgramState {
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }

    pub fn is_paused(&self) -> bool {
        matches!(self, Self::Paused { .. })
    }
}

impl From<EvaluatorState> for ProgramState {
    fn from(state: EvaluatorState) -> Self {
        match state {
            EvaluatorState::Running => Self::Running,
            EvaluatorState::Finished => Self::Finished,
            EvaluatorState::Error { err, instruction } => {
                Self::Error { err, instruction }
            }
        }
    }
}
