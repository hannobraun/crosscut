use itertools::Itertools;

use crate::{
    actor::{Actor, Sender, ThreadHandle},
    editor,
};

use super::{
    code::Code,
    compiler::compile,
    host::Host,
    interpreter::{ActiveCall, Interpreter},
};

pub fn start(
    game_output: Sender<GameOutput>,
) -> anyhow::Result<(ThreadHandle, Actor<String>, Actor<GameInput>)> {
    let host = Host::from_function_names(["color", "__color_currying"]);
    let mut code = Code::default();
    let mut interpreter = Interpreter::default();
    let mut values = Vec::new();

    editor::update(&code, &interpreter)?;

    let handle_events = Actor::spawn(move |event| {
        match event {
            Event::EditorInput { line } => {
                compile(&line, &host, &mut code);
                editor::update(&code, &interpreter)?;
            }
            Event::GameInput(GameInput::RenderingFrame) => {
                // This loop is coupled to the frame rate of the renderer.
            }
        }

        if let Some((_, value)) = interpreter.step(&code) {
            values.push(value);

            interpreter.active_call = if let Some([r, g, b, a]) =
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
                let Some(function) = host.function_by_name("__color_currying")
                else {
                    unreachable!("Function has been defined above.");
                };
                Some(ActiveCall { function })
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
