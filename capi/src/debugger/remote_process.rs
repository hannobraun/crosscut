use crate::{
    process::{Memory, Process},
    source_map::SourceMap,
    syntax,
    updates::Update,
};

use super::model::{ActiveFunctions, Debugger};

pub struct RemoteProcess {
    pub source_code: Option<(syntax::Functions, SourceMap)>,
    pub process: Option<Process>,
    pub memory: Option<Memory>,
}

impl RemoteProcess {
    pub fn new() -> Self {
        Self {
            source_code: None,
            process: None,
            memory: None,
        }
    }

    pub fn on_update(&mut self, update: Update) {
        match update {
            Update::Memory { memory } => {
                self.memory = Some(memory);
            }
            Update::Process(process) => {
                self.process = Some(process);
            }
            Update::SourceCode {
                functions,
                source_map,
            } => {
                self.source_code = Some((functions, source_map));
            }
        }
    }

    pub fn to_debugger(&self) -> Debugger {
        let active_functions = ActiveFunctions::new(
            self.source_code.as_ref().map(|(functions, _)| functions),
            self.source_code.as_ref().map(|(_, source_map)| source_map),
            self.process.as_ref(),
        );
        let data_stacks = self.process.as_ref().map(|process| {
            [
                process.previous_data_stack.clone(),
                process.evaluator.data_stack().clone(),
            ]
        });
        let memory = self.memory.clone();

        Debugger {
            active_functions,
            data_stacks,
            memory,
        }
    }
}
