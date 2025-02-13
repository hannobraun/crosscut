use super::{
    code::{Codebase, NodePath},
    editor::{Editor, EditorCommand, EditorInputEvent},
    packages::Package,
    runtime::{Effect, Evaluator, StepResult, Value},
};

#[derive(Debug)]
pub struct Language {
    codebase: Codebase,
    editor: Editor,
    evaluator: Evaluator,
    package: Package,
}

impl Language {
    pub fn with_package(package: Package) -> Self {
        let codebase = Codebase::new();
        let editor = Editor::new(&codebase);
        let evaluator = Evaluator::new(codebase.root().path, &codebase);

        Self {
            codebase,
            editor,
            evaluator,
            package,
        }
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

    pub fn package(&self) -> &Package {
        &self.package
    }

    pub fn on_input(&mut self, event: EditorInputEvent) {
        self.editor.on_input(
            event,
            &mut self.codebase,
            &mut self.evaluator,
            &self.package,
        );
    }

    pub fn on_command(&mut self, command: EditorCommand) {
        self.editor.on_command(
            command,
            &mut self.codebase,
            &mut self.evaluator,
        );
    }

    pub fn evaluate(&mut self, root: NodePath, active_value: Value) {
        self.evaluator.evaluate(root, active_value, &self.codebase);
    }

    pub fn step(&mut self) -> StepResult {
        self.evaluator.step(&self.codebase)
    }

    pub fn provide_host_function_output(&mut self, output: Value) {
        self.evaluator.provide_host_function_output(output);
    }

    pub fn trigger_effect(&mut self, effect: Effect) {
        self.evaluator.trigger_effect(effect);
    }
}

#[cfg(test)]
use super::{packages::FunctionId, runtime::ValueWithSource};

#[cfg(test)]
impl Language {
    pub fn without_package() -> Self {
        Self::with_package(Package::new())
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

    pub fn step_until_finished(&mut self) -> Result<ValueWithSource, Effect> {
        self.step_until_finished_and_handle_host_functions(|id, _| {
            unreachable!("Unexpected host function with ID `{id:?}`.")
        })
    }

    pub fn step_until_finished_and_handle_host_functions(
        &mut self,
        mut handler: impl FnMut(FunctionId, Value) -> Result<Value, Effect>,
    ) -> Result<ValueWithSource, Effect> {
        let mut i = 0;

        loop {
            match self.step() {
                StepResult::Running { .. } => {
                    // We're not concerned with intermediate results here.
                }
                StepResult::Recursing => {}
                StepResult::Effect { effect, path: _ } => match effect {
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
