use super::{
    code::{Codebase, NodePath},
    editor::{Editor, EditorCommand, EditorInputEvent},
    packages::{Function, Package, Packages},
    runtime::{Effect, Evaluator, RuntimeState, Value},
};

#[derive(Debug)]
pub struct Language {
    codebase: Codebase,
    editor: Editor,
    evaluator: Evaluator,
    packages: Packages,
}

impl Language {
    pub fn new() -> Self {
        let codebase = Codebase::new();
        let evaluator = Evaluator::new();
        let packages = Packages::new();

        let editor = Editor::new(codebase.root().path, &codebase, &packages);

        Self {
            codebase,
            editor,
            evaluator,
            packages,
        }
    }

    pub fn with_package<T: Function>(&mut self, package: &Package<T>) {
        self.packages.register_package(package);
    }

    pub fn codebase(&self) -> &Codebase {
        &self.codebase
    }

    pub fn editor(&self) -> &Editor {
        &self.editor
    }

    pub fn evaluator(&self) -> &Evaluator {
        &self.evaluator
    }

    pub fn packages(&self) -> &Packages {
        &self.packages
    }

    pub fn on_input(&mut self, event: EditorInputEvent) {
        self.editor.on_input(
            event,
            &mut self.codebase,
            &mut self.evaluator,
            &self.packages,
        );
    }

    pub fn on_command(&mut self, command: EditorCommand) {
        self.editor.on_command(
            command,
            &mut self.codebase,
            &mut self.evaluator,
            &self.packages,
        );
    }

    pub fn call_function(&mut self, root: NodePath, active_value: Value) {
        self.evaluator
            .call_function(root, active_value, &self.codebase);
    }

    pub fn step(&mut self) -> &RuntimeState {
        self.evaluator.step(&self.codebase);
        self.evaluator.state()
    }

    pub fn provide_host_function_output(&mut self, output: Value) {
        self.evaluator.provide_host_function_output(output);
    }

    pub fn trigger_effect(&mut self, effect: Effect) {
        self.evaluator.trigger_effect(effect);
    }
}

#[cfg(test)]
use super::packages::FunctionId;

#[cfg(test)]
impl Language {
    pub fn from_code(code: &str) -> Self {
        let mut language = Self::new();
        language.on_code(code);
        language
    }

    pub fn on_code(&mut self, code: &str) {
        self.editor.on_code(
            code,
            &mut self.codebase,
            &mut self.evaluator,
            &self.packages,
        );
    }

    pub fn step_until_finished(&mut self) -> Result<(Value, NodePath), Effect> {
        self.step_until_finished_and_handle_host_functions(|id, _| {
            unreachable!("Unexpected host function with ID `{id:?}`.")
        })
    }

    pub fn step_until_finished_and_handle_host_functions(
        &mut self,
        mut handler: impl FnMut(&FunctionId, &Value) -> Result<Value, Effect>,
    ) -> Result<(Value, NodePath), Effect> {
        use crate::game_engine::codebase_to_string;

        let mut i = 0;

        loop {
            match self.step() {
                RuntimeState::Started | RuntimeState::Running { .. } => {
                    // We're not concerned with intermediate results here.
                }
                RuntimeState::Effect { effect, path: _ } => match effect {
                    Effect::ApplyHostFunction { id, input } => {
                        match handler(id, input) {
                            Ok(output) => {
                                self.provide_host_function_output(output);
                            }
                            Err(effect) => {
                                self.trigger_effect(effect);
                            }
                        }
                    }
                    effect => {
                        break Err(effect.clone());
                    }
                },
                RuntimeState::Finished { output, path } => {
                    break Ok((output.clone(), path.clone()));
                }
                RuntimeState::Error { .. } => {
                    panic!(
                        "Unexpected runtime error from this code:\n\
                        {}",
                        codebase_to_string(&self.codebase),
                    );
                }
            }

            i += 1;

            if i > 1024 {
                // This function is only used in tests. And those are not so
                // complicated, as to require a large number of steps.
                panic!("Test seemingly ran into an endless loop.");
            }
        }
    }
}
