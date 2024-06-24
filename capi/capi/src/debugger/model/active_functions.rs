use capi_compiler::{source_map::SourceMap, syntax};
use capi_process::Process;

use super::Function;

#[derive(Clone)]
pub enum ActiveFunctions {
    Functions { functions: Vec<Function> },
    Message { message: &'static str },
}

impl ActiveFunctions {
    pub fn new(
        source_code: Option<(&syntax::Functions, &SourceMap)>,
        process: Option<&Process>,
    ) -> Self {
        let Some((functions, source_map)) = source_code else {
            return Self::Message {
                message: "No connection to Caterpillar process.",
            };
        };
        let Some(process) = process else {
            return Self::Message {
                message: "No process available.",
            };
        };

        if process.state().can_step() {
            return Self::Message {
                message: "Process is running.",
            };
        }
        if process.state().has_finished() {
            return Self::Message {
                message: "Process is finished.",
            };
        }

        let functions = process
            .stack()
            .all_next_instructions_in_frames()
            .filter_map(|runtime_location| {
                let syntax_location =
                    source_map.runtime_to_syntax(&runtime_location);
                let function =
                    functions.get_from_location(syntax_location).cloned()?;

                Some(Function::new(function, source_map, process))
            })
            .collect();

        Self::Functions { functions }
    }
}
