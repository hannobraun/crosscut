use super::{
    code::Codebase,
    editor::{Editor, EditorCommand, EditorInputEvent},
    packages::Package,
    runtime::{Effect, Interpreter, StepResult, Value},
};

#[derive(Debug)]
pub struct Language {
    codebase: Codebase,
    editor: Editor,
    interpreter: Interpreter,
    package: Package,
}

impl Language {
    pub fn with_host(package: Package) -> Self {
        let codebase = Codebase::new();
        let editor = Editor::new(&codebase);
        let interpreter = Interpreter::new(&codebase);

        Self {
            codebase,
            editor,
            interpreter,
            package,
        }
    }

    pub fn codebase(&self) -> &Codebase {
        &self.codebase
    }

    pub fn editor(&self) -> &Editor {
        &self.editor
    }

    pub fn interpreter(&self) -> &Interpreter {
        &self.interpreter
    }

    pub fn host(&self) -> &Package {
        &self.package
    }

    pub fn on_input(&mut self, event: EditorInputEvent) {
        self.editor.on_input(
            event,
            &mut self.codebase,
            &mut self.interpreter,
            &self.package,
        );
    }

    pub fn on_command(&mut self, command: EditorCommand) {
        self.editor.on_command(
            command,
            &mut self.codebase,
            &mut self.interpreter,
        );
    }

    pub fn step(&mut self) -> StepResult {
        self.interpreter.step(&self.codebase)
    }

    pub fn provide_host_function_output(&mut self, output: Value) {
        self.interpreter
            .provide_host_function_output(output, &self.codebase);
    }

    pub fn trigger_effect(&mut self, effect: Effect) {
        self.interpreter.trigger_effect(effect);
    }
}

#[cfg(test)]
impl Language {
    pub fn without_host() -> Self {
        Self::with_host(Package::new())
    }

    pub fn enter_code(&mut self, code: &str) {
        for ch in code.chars() {
            let event = if ch.is_whitespace() {
                EditorInputEvent::SubmitNode
            } else {
                EditorInputEvent::Insert { ch }
            };

            self.on_input(event);
        }
    }

    pub fn step_until_finished(&mut self) -> Result<Value, Effect> {
        self.step_until_finished_and_handle_host_functions(|id, _| {
            unreachable!("Unexpected host function with ID `{id}`.")
        })
    }

    pub fn step_until_finished_and_handle_host_functions(
        &mut self,
        mut handler: impl FnMut(u32, Value) -> Result<Value, Effect>,
    ) -> Result<Value, Effect> {
        let mut i = 0;

        loop {
            match self.step() {
                StepResult::FunctionApplied { output: _ } => {
                    // We're not concerned with intermediate results here.
                }
                StepResult::EffectTriggered { effect } => match effect {
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
                        break Err(effect);
                    }
                },
                StepResult::Finished { output } => {
                    break Ok(output);
                }
                StepResult::Error => {
                    panic!("Unexpected runtime error.");
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
