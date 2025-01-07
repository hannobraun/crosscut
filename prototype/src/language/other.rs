use std::collections::BTreeMap;

use itertools::Itertools;

use crate::{
    actor::{Actor, Sender, ThreadHandle},
    editor,
};

use super::{
    code::{Code, HostFunction, Signature},
    compiler::compile,
    host::Host,
    interpreter::Interpreter,
};

pub fn start(
    game_output: Sender<GameOutput>,
) -> anyhow::Result<(ThreadHandle, Actor<String>, Actor<GameInput>)> {
    let host = Host {
        functions: BTreeMap::from(
            [(
                "submit_color",
                Signature {
                    input: (),
                    output: (),
                },
            )]
            .map(|(name, function)| {
                (
                    name.to_string(),
                    HostFunction {
                        signature: function,
                    },
                )
            }),
        ),
    };
    let mut code = Code::default();
    let mut interpreter = Interpreter::default();
    let mut values = Vec::new();

    editor::update(&code, &interpreter)?;

    let handle_events = Actor::spawn(move |event| {
        match event {
            Event::EditorInput { line } => {
                compile(line, &host, &mut code);
                editor::update(&code, &interpreter)?;
            }
            Event::GameInput(GameInput::RenderingFrame) => {
                // This loop is coupled to the frame rate of the renderer.
            }
        }

        if let Some(value) = interpreter.step(&code) {
            values.push(value);

            interpreter.active_function = if let Some([r, g, b, a]) =
                values.iter().copied().collect_array()
            {
                values.clear();
                game_output.send(GameOutput::SubmitColor {
                    color: [r, g, b, a],
                })?;

                None
            } else {
                // Functions can only have one input, but we need 4 values for a
                // color. Let's get some more using currying.
                Some(Signature {
                    input: (),
                    output: (),
                })
            }
        };

        Ok(())
    });

    let events_from_commands = handle_events.sender.clone();
    let handle_editor_input = Actor::spawn(move |line| {
        events_from_commands.send(Event::EditorInput { line })?;
        Ok(())
    });

    let events_from_input = handle_events.sender;
    let input_to_event = Actor::spawn(move |input| {
        events_from_input.send(Event::GameInput(input))?;
        Ok(())
    });

    Ok((handle_events.handle, handle_editor_input, input_to_event))
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
