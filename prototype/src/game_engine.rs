use crate::{
    editor::Editor,
    language::{
        host::Host,
        interpreter::{Interpreter, StepResult, Value},
    },
    thread::{self, Sender, ThreadHandle},
};

pub struct GameEngine {
    pub threads: GameEngineThreads,
    pub senders: GameEngineSenders,
}

impl GameEngine {
    pub fn start(game_output_tx: Sender<GameOutput>) -> anyhow::Result<Self> {
        let host = Host::from_functions(["dim"]);
        let mut editor = Editor::default();
        let mut interpreter = Interpreter::new(editor.code());

        editor.render(&host, &interpreter)?;

        let (events_tx, events_rx) = thread::channel();

        let handle_events = thread::spawn(move || {
            let event = events_rx.recv()?;

            match event {
                Event::EditorInput { line } => {
                    editor.process_input(line, &host, &mut interpreter);

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

                    editor.render(&host, &interpreter)?;
                }
                Event::GameInput(GameInput::RenderingFrame) => {
                    // This loop is coupled to the frame rate of the renderer.
                }
            }

            Ok(())
        });

        let (editor_input_tx, editor_input_rx) = thread::channel();
        let events_from_editor_input = events_tx.clone();
        let handle_editor_input = thread::spawn(move || {
            let line = editor_input_rx.recv()?;
            if let Some(line) = line {
                events_from_editor_input.send(Event::EditorInput { line })?;
            }
            Ok(())
        });

        let (game_input_tx, game_input_rx) = thread::channel();
        let events_from_game_input = events_tx;
        let handle_game_input = thread::spawn(move || {
            let input = game_input_rx.recv()?;
            events_from_game_input.send(Event::GameInput(input))?;
            Ok(())
        });

        let threads = GameEngineThreads {
            handle: handle_events,
            handle_editor_input,
            handle_game_input,
        };
        let senders = GameEngineSenders {
            editor_input: editor_input_tx,
            game_input: game_input_tx,
        };

        Ok(Self { threads, senders })
    }
}

pub struct GameEngineThreads {
    handle: ThreadHandle,
    handle_editor_input: ThreadHandle,
    handle_game_input: ThreadHandle,
}

impl GameEngineThreads {
    pub fn join(self) -> anyhow::Result<()> {
        self.handle.join()?;
        self.handle_editor_input.join()?;
        self.handle_game_input.join()?;

        Ok(())
    }
}

pub struct GameEngineSenders {
    pub editor_input: Sender<Option<String>>,
    pub game_input: Sender<GameInput>,
}

enum Event {
    EditorInput { line: String },
    GameInput(GameInput),
}

#[derive(Debug)]
pub enum GameInput {
    RenderingFrame,
}

pub enum GameOutput {
    SubmitColor { color: [f64; 4] },
}
