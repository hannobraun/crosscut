use std::ops::ControlFlow;

use crossbeam_channel::select;

use crate::{
    io::editor::output::Renderer,
    lang::{
        self, editor,
        host::Host,
        interpreter::{StepResult, Value},
    },
    thread::{self, ChannelDisconnected, Sender, ThreadHandle},
};

pub struct GameEngine {
    pub handle: ThreadHandle,
    pub editor_input: Sender<Option<editor::InputEvent>>,
    pub game_input: Sender<GameInput>,
}

impl GameEngine {
    pub fn start(game_output_tx: Sender<GameOutput>) -> anyhow::Result<Self> {
        let host = Host::from_functions(["dim"]);

        let mut lang = lang::Instance::new();
        let mut renderer = Renderer::new()?;

        renderer.render(
            &lang.editor,
            &lang.code,
            Some(&lang.interpreter),
            &host,
        )?;

        // Need to specify the types of the channels explicitly, to work around
        // this bug in rust-analyzer:
        // https://github.com/rust-lang/rust-analyzer/issues/15984
        let (editor_input_tx, editor_input_rx) =
            thread::channel::<Option<editor::InputEvent>>();
        let (game_input_tx, game_input_rx) = thread::channel::<GameInput>();

        let handle = thread::spawn(move || {
            let event = select! {
                recv(editor_input_rx.inner()) -> result => {
                    result.map(|maybe_event|
                        if let Some(event) = maybe_event {
                            Event::EditorInput { event }}
                        else {
                            Event::Heartbeat
                        }
                    )
                }
                recv(game_input_rx.inner()) -> result => {
                    result.map(|input| Event::GameInput { input })
                }
            };
            let Ok(event) = event else {
                return Err(ChannelDisconnected.into());
            };

            match event {
                Event::EditorInput { event } => {
                    lang.on_input(event, &host);

                    loop {
                        match lang.interpreter.step(&lang.code) {
                            StepResult::CallToHostFunction {
                                id,
                                input,
                                output,
                            } => {
                                match id {
                                    0 => {
                                        // `dim`

                                        let Value::Integer { value: input } =
                                            input;
                                        let Value::Integer { value: output } =
                                            output;

                                        *output = input / 2;
                                    }
                                    id => {
                                        unreachable!(
                                            "Undefined host function: `{id}`"
                                        );
                                    }
                                }

                                continue;
                            }
                            StepResult::CallToIntrinsicFunction => {
                                // Nothing to be done about this.
                            }
                            StepResult::Error => {
                                // Not handling errors right now. They should be
                                // properly encoded in `Code` and therefore
                                // visible in the editor.
                            }
                            StepResult::Finished { output } => {
                                let Value::Integer { value: output } = output;
                                let color = output as f64 / 255.;

                                game_output_tx.send(
                                    GameOutput::SubmitColor {
                                        color: [color, color, color, 1.],
                                    },
                                )?;
                            }
                        }

                        break;
                    }

                    renderer.render(
                        &lang.editor,
                        &lang.code,
                        Some(&lang.interpreter),
                        &host,
                    )?;
                }
                Event::GameInput {
                    input: GameInput::RenderingFrame,
                } => {
                    // This loop is coupled to the frame rate of the renderer.
                }
                Event::Heartbeat => {}
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
enum Event {
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

#[derive(Debug)]
pub enum GameInput {
    RenderingFrame,
}

pub enum GameOutput {
    SubmitColor { color: [f64; 4] },
}
