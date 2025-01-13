use crossbeam_channel::select;

use crate::{
    editor::{Editor, EditorInput},
    io::editor::output::Renderer,
    language::{
        host::Host,
        interpreter::{Interpreter, StepResult, Value},
    },
    thread::{self, ChannelDisconnected, Sender, ThreadHandle},
};

pub struct GameEngine {
    pub handle: ThreadHandle,
    pub editor_input: Sender<Option<EditorInput>>,
    pub game_input: Sender<GameInput>,
}

impl GameEngine {
    pub fn start(game_output_tx: Sender<GameOutput>) -> anyhow::Result<Self> {
        let host = Host::from_functions(["dim"]);
        let mut editor = Editor::default();
        let mut renderer = Renderer::new();
        let mut interpreter = Interpreter::new(editor.code());

        renderer.render(&editor, &host, Some(&interpreter))?;

        // Need to specify the types of the channels explicitly, to work around
        // this bug in rust-analyzer:
        // https://github.com/rust-lang/rust-analyzer/issues/15984
        let (editor_input_tx, editor_input_rx) =
            thread::channel::<Option<EditorInput>>();
        let (game_input_tx, game_input_rx) = thread::channel::<GameInput>();

        let handle = thread::spawn(move || {
            let event = select! {
                recv(editor_input_rx.inner()) -> result => {
                    result.map(|maybe_input|
                        if let Some(input) = maybe_input {
                            Event::EditorInput { input }}
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
                Event::EditorInput { input } => {
                    let render =
                        editor.process_input(input, &host, &mut interpreter);

                    loop {
                        match interpreter.step(editor.code()) {
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
                            StepResult::Error => {
                                // Not handling errors right now. Eventually,
                                // those should be properly encoded in `Code`
                                // and therefore visible in the editor. But in
                                // any case, there's nothing to do here, at
                                // least for now.
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

                    if render {
                        renderer.render(&editor, &host, Some(&interpreter))?;
                    }
                }
                Event::GameInput {
                    input: GameInput::RenderingFrame,
                } => {
                    // This loop is coupled to the frame rate of the renderer.
                }
                Event::Heartbeat => {}
            }

            Ok(())
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
        input: EditorInput,
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
