use itertools::Itertools;

use crate::{
    actor::{Actor, Sender, ThreadHandle},
    code::{Code, Expression},
    editor,
    interpreter::Interpreter,
};

pub fn start(
    game_output: Sender<GameOutput>,
) -> anyhow::Result<(ThreadHandle, Actor<String>, Actor<GameInput>)> {
    let mut code = Code::default();
    let mut interpreter = Interpreter::default();
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

        if let Some(expression) =
            code.expressions.get(interpreter.next_expression)
        {
            let value = match expression {
                Expression::InvalidNumber { .. } => {
                    return Ok(());
                }
                Expression::LiteralNumber { value } => value,
            };
            values.push(*value);
            interpreter.next_expression += 1;

            let Some((r, g, b, a)) = values.iter().copied().collect_tuple()
            else {
                // Don't have enough values yet to constitute a color.
                return Ok(());
            };

            values.clear();
            game_output.send(GameOutput::SubmitColor {
                color: [r, g, b, a],
            })?;
        }

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
        .map(|channel| match channel.parse::<f64>() {
            Ok(value) => Expression::LiteralNumber { value },
            Err(_) => Expression::InvalidNumber {
                invalid: channel.to_string(),
            },
        })
        .collect()
}
