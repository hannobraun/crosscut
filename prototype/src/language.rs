use std::collections::BTreeMap;

use itertools::Itertools;

use crate::{
    actor::{Actor, Sender, ThreadHandle},
    code::{Code, Expression},
    editor,
    interpreter::{Function, Interpreter},
};

pub fn start(
    game_output: Sender<GameOutput>,
) -> anyhow::Result<(ThreadHandle, Actor<String>, Actor<GameInput>)> {
    let mut code = Code::default();
    let mut interpreter = Interpreter {
        functions: BTreeMap::from(
            [(
                "submit_color",
                Function {
                    input: (),
                    output: (),
                },
            )]
            .map(|(name, function)| (name.to_string(), function)),
        ),
        next_expression: 0,
        active_function: None,
    };
    let mut values = Vec::new();

    editor::update(&code, &interpreter)?;

    let handle_events = Actor::spawn(move |event| {
        match event {
            Event::EditorInput { line } => {
                let expressions = parse(line);
                code.expressions.extend(expressions);
                editor::update(&code, &interpreter)?;
            }
            Event::GameInput(GameInput::RenderingFrame) => {
                // This loop is coupled to the frame rate of the renderer.
            }
        }

        if let Some(value) = interpreter.step(&code) {
            values.push(value);

            if let Some([r, g, b, a]) = values.iter().copied().collect_array() {
                values.clear();
                game_output.send(GameOutput::SubmitColor {
                    color: [r, g, b, a],
                })?;

                interpreter.active_function = None;
            } else {
                interpreter.active_function = Some(Function {
                    input: (),
                    output: (),
                });
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

fn parse(line: String) -> Vec<Expression> {
    line.split_whitespace()
        .map(|token| match token.parse::<f64>() {
            Ok(value) => Expression::LiteralNumber { value },
            Err(_) => Expression::Identifier {
                name: token.to_string(),
            },
        })
        .collect()
}
