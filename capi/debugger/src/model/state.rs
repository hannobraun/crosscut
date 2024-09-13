use capi_game_engine::{command::Command, memory::Memory};
use capi_process::{
    Breakpoints, Effect, Instruction, Instructions, ProcessState, Value,
};
use capi_protocol::{
    runtime_state::RuntimeState,
    updates::{Code, UpdateFromRuntime},
};

use super::{ActiveFunctions, DebugCode, UserAction};

#[derive(Clone, Debug, Default)]
pub struct PersistentState {
    pub code: DebugCode,
    pub breakpoints: Breakpoints,
    pub runtime_state: Option<RuntimeState>,
    pub memory: Option<Memory>,
}

impl PersistentState {
    pub fn on_new_code(&mut self, code: Code) -> Command {
        let instructions = self.apply_breakpoints(&code);
        self.code.inner = Some(code);
        Command::UpdateCode { instructions }
    }

    pub fn on_update_from_runtime(&mut self, update: UpdateFromRuntime) {
        match update {
            UpdateFromRuntime::Memory { memory } => {
                self.memory = Some(memory);
            }
            UpdateFromRuntime::Process(process) => {
                let runtime_state = match process.state() {
                    ProcessState::Running => RuntimeState::Running,
                    ProcessState::Finished => RuntimeState::Finished,
                    ProcessState::Stopped => RuntimeState::Stopped {
                        effects: process.effects().queue().collect(),
                        active_instructions: process
                            .evaluator()
                            .active_instructions()
                            .collect(),
                        current_operands: process
                            .stack()
                            .operands_in_current_stack_frame()
                            .copied()
                            .collect::<Vec<_>>(),
                    },
                };

                self.runtime_state = Some(runtime_state);
            }
        }
    }

    pub fn on_user_action(
        &mut self,
        action: UserAction,
        transient: &TransientState,
    ) -> anyhow::Result<Vec<Command>> {
        let mut commands = Vec::new();

        match action {
            UserAction::BreakpointClear { fragment, .. } => {
                let code = self.code.get()?;
                let address = self.code.fragment_to_instruction(&fragment)?;

                self.breakpoints.clear_durable(&address);

                commands.push(Command::UpdateCode {
                    instructions: self.apply_breakpoints(code),
                });
            }
            UserAction::BreakpointSet { fragment, .. } => {
                let code = self.code.get()?;
                let address = self.code.fragment_to_instruction(&fragment)?;

                self.breakpoints.set_durable(address);

                commands.push(Command::UpdateCode {
                    instructions: self.apply_breakpoints(code),
                });
            }
            UserAction::Continue => {
                commands.push(Command::Continue);
            }
            UserAction::Reset => {
                commands.push(Command::Reset);
            }
            UserAction::StepInto => {
                let branch = transient
                    .active_functions
                    .entries()?
                    .leaf()
                    .function()?
                    .active_branch()?;

                let origin = branch.active_fragment()?;
                let Some(target) = branch.fragment_after(origin)? else {
                    // No fragment after the active one in the current function,
                    // meaning we have to step out of the function.
                    //
                    // This code doesn't support this yet. Falling back to the
                    // previous behavior.

                    commands.push(Command::Step);
                    return Ok(commands);
                };

                let origin =
                    self.code.fragment_to_instruction(&origin.data.id)?;
                let target =
                    self.code.fragment_to_instruction(&target.data.id)?;

                self.breakpoints.set_ephemeral(target);
                commands.push(Command::UpdateCode {
                    instructions: self.apply_breakpoints(self.code.get()?),
                });

                let origin = self.code.instruction(&origin)?;
                if let Instruction::TriggerEffect {
                    effect: Effect::Breakpoint,
                } = origin
                {
                    commands.push(Command::IgnoreNextInstruction);
                }
            }
            UserAction::Stop => {
                commands.push(Command::Stop);
            }
        };

        Ok(commands)
    }

    pub fn apply_breakpoints(&self, code: &Code) -> Instructions {
        let mut instructions = code.instructions.clone();

        for address in self.breakpoints.iter() {
            instructions.replace(
                address,
                Instruction::TriggerEffect {
                    effect: Effect::Breakpoint,
                },
            );
        }

        instructions
    }

    pub fn generate_transient_state(&self) -> TransientState {
        let active_functions = ActiveFunctions::new(
            self.code.inner.as_ref(),
            &self.breakpoints,
            self.runtime_state.as_ref(),
        );
        let operands = match &self.runtime_state {
            Some(RuntimeState::Stopped {
                current_operands, ..
            }) => current_operands.clone(),
            _ => Vec::new(),
        };

        TransientState {
            active_functions,
            operands,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TransientState {
    pub active_functions: ActiveFunctions,
    pub operands: Vec<Value>,
}
