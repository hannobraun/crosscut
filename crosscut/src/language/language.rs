use super::{
    code::{Codebase, NodePath},
    editor::{Editor, EditorCommand, EditorInput},
    runtime::{
        Effect, Evaluator, RuntimeState, Value, apply_intrinsic_function,
    },
};

#[derive(Debug)]
pub struct Language {
    codebase: Codebase,
    editor: Editor,
    evaluator: Evaluator,
}

impl Language {
    pub fn new() -> Self {
        let codebase = Codebase::new();
        let editor = Editor::new(codebase.root().path, &codebase);
        let evaluator = Evaluator::default();

        Self {
            codebase,
            editor,
            evaluator,
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

    pub fn on_input(&mut self, input: EditorInput) {
        self.editor
            .on_input(input, &mut self.codebase, &mut self.evaluator);
    }

    pub fn on_command(&mut self, command: EditorCommand) -> anyhow::Result<()> {
        self.editor.on_command(
            command,
            &mut self.codebase,
            &mut self.evaluator,
        )?;

        Ok(())
    }

    pub fn apply_function(&mut self, body: NodePath) {
        self.evaluator.apply_function(
            "".to_string(),
            body,
            Value::nothing(),
            self.codebase.nodes(),
        );
    }

    pub fn step(&mut self) -> &RuntimeState {
        self.evaluator.step(&self.codebase);

        if let RuntimeState::Effect {
            effect: Effect::ApplyProvidedFunction { name, input },
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
        self.editor
            .on_code(code, &mut self.codebase, &mut self.evaluator);
        self
    }

    pub fn down(&mut self) -> &mut Self {
        self.on_input(EditorInput::MoveCursorDown);
        self
    }

    pub fn up(&mut self) -> &mut Self {
        self.on_input(EditorInput::MoveCursorUp);
        self
    }

    pub fn remove_right(&mut self) -> &mut Self {
        self.on_input(EditorInput::RemoveRight { whole_node: false });
        self
    }

    pub fn remove_left(&mut self) -> &mut Self {
        self.on_input(EditorInput::RemoveLeft { whole_node: false });
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
        let mut i = 0;

        loop {
            match self.step() {
                RuntimeState::Started | RuntimeState::Running => {
                    // We're not concerned with intermediate results here.
                }
                RuntimeState::Effect { effect, .. } => match effect {
                    Effect::ApplyProvidedFunction { name, input } => {
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

impl Default for Language {
    fn default() -> Self {
        Self::new()
    }
}
