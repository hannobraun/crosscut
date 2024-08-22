use std::{collections::VecDeque, fmt};

use capi_compiler::fragments::FragmentId;
use capi_process::{InstructionAddress, Process};
use capi_protocol::updates::Code;

use super::Function;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActiveFunctions {
    Functions { functions: Vec<Function> },
    Message { message: ActiveFunctionsMessage },
}

impl ActiveFunctions {
    pub fn new(code: Option<&Code>, process: Option<&Process>) -> Self {
        let Some(code) = code else {
            return Self::Message {
                message: ActiveFunctionsMessage::NoServer,
            };
        };
        let Some(process) = process else {
            return Self::Message {
                message: ActiveFunctionsMessage::NoProcess,
            };
        };

        if process.can_step() {
            return Self::Message {
                message: ActiveFunctionsMessage::ProcessRunning,
            };
        }
        if process.has_finished() {
            return Self::Message {
                message: ActiveFunctionsMessage::ProcessFinished,
            };
        }

        let mut call_stack: VecDeque<InstructionAddress> =
            process.evaluator().active_instructions().collect();

        let mut functions = VecDeque::new();

        while let Some(instruction) = call_stack.pop_front() {
            let fragment_id = instruction_to_fragment(&instruction, code);
            let (function, _) = code
                .fragments
                .find_function_by_fragment_in_body(&fragment_id)
                .expect(
                    "Expecting code that is referenced on call stack to be \
                    part of a known function.",
                );

            functions.push_front(function.clone());
        }

        Self::Functions {
            functions: functions
                .into_iter()
                .map(|function| {
                    Function::new(
                        function,
                        &code.fragments,
                        &code.source_map,
                        process,
                    )
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActiveFunctionsMessage {
    NoServer,
    NoProcess,
    ProcessRunning,
    ProcessFinished,
}

impl fmt::Display for ActiveFunctionsMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NoServer => {
                write!(f, "No connection to server.")?;
            }
            Self::NoProcess => {
                write!(f, "No connection to process.")?;
            }
            Self::ProcessRunning => {
                write!(f, "Process is running.")?;
            }
            Self::ProcessFinished => {
                write!(f, "Process is finished.")?;
            }
        }

        Ok(())
    }
}

fn instruction_to_fragment(
    instruction: &InstructionAddress,
    code: &Code,
) -> FragmentId {
    let Some(fragment) = code.source_map.instruction_to_fragment(instruction)
    else {
        panic!(
            "Expecting all instructions referenced on call stack to map to a \
            fragment, but instruction `{instruction}` does not."
        );
    };

    fragment
}
