use std::ops::ControlFlow;

use crossbeam_channel::select;

use crate::{
    io::editor::output::EditorOutput,
    lang::{
        self, editor,
        host::Host,
        interpreter::{StepResult, Value},
    },
    threads::{self, ChannelDisconnected, Sender, ThreadHandle},
};

pub struct GameEngineThread {
    pub handle: ThreadHandle,
    pub editor_input: Sender<Option<editor::InputEvent>>,
    pub game_input: Sender<GameInput>,
}

impl GameEngineThread {
    pub fn start(game_output_tx: Sender<GameOutput>) -> anyhow::Result<Self> {
        let mut game_engine = GameEngine::new()?;
        game_engine.render_editor()?;

        // Need to specify the types of the channels explicitly, to work around
        // this bug in rust-analyzer:
        // https://github.com/rust-lang/rust-analyzer/issues/15984
        let (editor_input_tx, editor_input_rx) =
            threads::channel::<Option<editor::InputEvent>>();
        let (game_input_tx, game_input_rx) = threads::channel::<GameInput>();

        let handle = threads::spawn(move || {
            let event = select! {
                recv(editor_input_rx.inner()) -> result => {
                    result.map(|maybe_event|
                        if let Some(event) = maybe_event {
                            GameEngineEvent::EditorInput { event }}
                        else {
                            GameEngineEvent::Heartbeat
                        }
                    )
                }
                recv(game_input_rx.inner()) -> result => {
                    result.map(|input| GameEngineEvent::GameInput { input })
                }
            };
            let Ok(event) = event else {
                return Err(ChannelDisconnected.into());
            };

            match event {
                GameEngineEvent::EditorInput { event } => {
                    let mut game_events = Vec::new();
                    game_engine.on_editor_input(event, &mut game_events)?;

                    for event in game_events {
                        game_output_tx.send(event)?;
                    }
                }
                GameEngineEvent::GameInput {
                    input: GameInput::RenderingFrame,
                } => {
                    // This loop is coupled to the frame rate of the renderer.
                }
                GameEngineEvent::Heartbeat => {}
            }

            Ok(ControlFlow::Continue(()))
        });

        Ok(Self {
            handle,
            editor_input: editor_input_tx,
            game_input: game_input_tx,
        })
    }
}

#[derive(Debug)]
enum GameEngineEvent {
    EditorInput {
        event: editor::InputEvent,
    },

    GameInput {
        input: GameInput,
    },

    /// # An event that has no effect when processed
    ///
    /// If a thread shuts down, either because of an error, or because the
    /// application is supposed to shut down as a whole, that needs to propagate
    /// to the other threads.
    ///
    /// For some threads, this is easily achieved, because they block on reading
    /// from a channel from another thread, which will fail the moment that
    /// other thread shuts down. Other threads block on something else, and
    /// don't benefit from this mechanism.
    ///
    /// Those other threads need to instead _send_ to another thread from time
    /// to time, to learn about the shutdown. This is what this event is for.
    Heartbeat,
}

pub struct GameEngine {
    host: Host,
    lang: lang::Instance,
    editor_output: EditorOutput,
}

impl GameEngine {
    pub fn new() -> anyhow::Result<Self> {
        let editor_output = EditorOutput::new()?;

        Ok(Self {
            host: Host::from_functions(["dim"]),
            lang: lang::Instance::new(),
            editor_output,
        })
    }

    pub fn on_editor_input(
        &mut self,
        event: editor::InputEvent,
        game_output: &mut Vec<GameOutput>,
    ) -> anyhow::Result<()> {
        self.lang.on_event(event, &self.host);

        loop {
            match self.lang.interpreter.step(&self.lang.code) {
                StepResult::CallToHostFunction { id, input, output } => {
                    match id {
                        0 => {
                            // `dim`

                            let Value::Integer { value: input } = input;
                            let Value::Integer { value: output } = output;

                            *output = input / 2;
                        }
                        id => {
                            unreachable!("Undefined host function: `{id}`");
                        }
                    }

                    continue;
                }
                StepResult::CallToIntrinsicFunction => {
                    // Nothing to be done about this.
                    continue;
                }
                StepResult::Error => {
                    // Not handling errors right now. They should be properly
                    // encoded in `Code` and therefore visible in the editor.
                }
                StepResult::Finished { output } => {
                    let Value::Integer { value: output } = output;
                    let color = output as f64 / 255.;

                    game_output.push(GameOutput::SubmitColor {
                        color: [color, color, color, 1.],
                    });
                }
            }

            break;
        }

        self.render_editor()?;

        Ok(())
    }

    fn render_editor(&mut self) -> anyhow::Result<()> {
        self.editor_output.render(
            &self.lang.editor,
            &self.lang.code,
            &self.lang.interpreter,
            &self.host,
        )?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum GameInput {
    RenderingFrame,
}

pub enum GameOutput {
    SubmitColor { color: [f64; 4] },
}
