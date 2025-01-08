use crate::{
    actor::{Actor, Sender, ThreadHandle},
    editor,
    language::{
        code::Code,
        compiler::compile,
        interpreter::{Interpreter, InterpreterState},
    },
};

pub struct GameEngine {
    pub handle: ThreadHandle,
    pub handle_editor_input: Actor<String>,
    pub handle_game_input: Actor<GameInput>,
}

impl GameEngine {
    pub fn start(game_output: Sender<GameOutput>) -> anyhow::Result<Self> {
        let mut code = Code::default();
        let mut interpreter = Interpreter::default();

        editor::update(&code, &interpreter)?;

        let handle_events = Actor::spawn(move |event| {
            match event {
                Event::EditorInput { line } => {
                    compile(&line, &mut code);

                    match interpreter.step(&code) {
                        InterpreterState::Error => {
                            // Not handling errors right now. Eventually, those
                            // should be properly encoded in `Code` and
                            // therefore visible in the editor. But in any case,
                            // there's nothing to do here, at least for now.
                        }
                        InterpreterState::Finished { output } => {
                            game_output.send(GameOutput::SubmitColor {
                                color: [output, output, output, 1.],
                            })?;
                        }
                    }

                    editor::update(&code, &interpreter)?;
                }
                Event::GameInput(GameInput::RenderingFrame) => {
                    // This loop is coupled to the frame rate of the renderer.
                }
            }

            Ok(())
        });

        let events_from_editor_input = handle_events.sender.clone();
        let handle_editor_input = Actor::spawn(move |line| {
            events_from_editor_input.send(Event::EditorInput { line })?;
            Ok(())
        });

        let events_from_game_input = handle_events.sender;
        let handle_game_input = Actor::spawn(move |input| {
            events_from_game_input.send(Event::GameInput(input))?;
            Ok(())
        });

        Ok(GameEngine {
            handle: handle_events.handle,
            handle_editor_input,
            handle_game_input,
        })
    }
}

enum Event {
    EditorInput { line: String },
    GameInput(GameInput),
}

pub enum GameInput {
    RenderingFrame,
}

pub enum GameOutput {
    SubmitColor { color: [f64; 4] },
}
