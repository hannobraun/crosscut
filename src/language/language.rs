use super::{
    code::{Codebase, IntrinsicFunction, NodePath},
    editor::{Editor, EditorCommand, EditorInputEvent},
    packages::Packages,
    runtime::{
        Effect, Evaluator, RuntimeState, Value, apply_intrinsic_function,
    },
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
        let mut packages = Packages::default();

        let editor = Editor::new(codebase.root().path, &codebase);

        packages.new_package([
            IntrinsicFunction::Add,
            IntrinsicFunction::Drop,
            IntrinsicFunction::Identity,
        ]);

        Self {
            codebase,
            editor,
            evaluator,
            packages,
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

    pub fn packages_mut(&mut self) -> &mut Packages {
        &mut self.packages
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
        );
    }

    pub fn apply_function(&mut self, root: NodePath) {
        self.evaluator.apply_function_raw(root, &self.codebase);
    }

    pub fn step(&mut self) -> &RuntimeState {
        self.evaluator.step(&self.codebase);

        if let RuntimeState::Effect {
            effect: Effect::ProvidedFunction { name, input },
            ..
        } = self.evaluator.state()
        {
            match apply_intrinsic_function(name, input) {
                Some(Ok(value)) => {
                    self.evaluator.exit_from_provided_function(value);
                }
                Some(Err(effect)) => {
                    self.evaluator.trigger_effect(effect);
                }
                None => {
                    // Function is not an intrinsic function and was not
                    // handled. Nothing else to do here. The host can take
                    // care of the effect.
                }
            }
        }

        self.evaluator.state()
    }

    pub fn provide_host_function_output(&mut self, output: Value) {
        self.evaluator.exit_from_provided_function(output);
    }

    pub fn trigger_effect(&mut self, effect: Effect) {
        self.evaluator.trigger_effect(effect);
    }
}

#[cfg(test)]
impl Language {
    pub fn code(&mut self, code: &str) -> &mut Self {
        self.editor.on_code(
            code,
            &mut self.codebase,
            &mut self.evaluator,
            &self.packages,
        );
        self
    }

    pub fn down(&mut self) -> &mut Self {
        self.on_input(EditorInputEvent::MoveCursorDown);
        self
    }

    pub fn step_until_finished(&mut self) -> Result<Value, Effect> {
        self.step_until_finished_and_handle_host_functions(|name, _| {
            unreachable!("Unexpected host function: `{name}`")
        })
    }

    pub fn step_until_finished_and_handle_host_functions(
        &mut self,
        mut handler: impl FnMut(&str, &Value) -> Result<Value, Effect>,
    ) -> Result<Value, Effect> {
        use crate::game_engine::codebase_to_string;

        let mut i = 0;

        loop {
            match self.step() {
                RuntimeState::Started | RuntimeState::Running { .. } => {
                    // We're not concerned with intermediate results here.
                }
                RuntimeState::Effect { effect, .. } => match effect {
                    Effect::ProvidedFunction { name, input } => {
                        match handler(name, input) {
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
                RuntimeState::Finished { output } => {
                    break Ok(output.clone());
                }
                RuntimeState::Error { .. } => {
                    panic!(
                        "Unexpected runtime error from this code:\n\
                        {}",
                        codebase_to_string(&self.codebase, &self.packages),
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
