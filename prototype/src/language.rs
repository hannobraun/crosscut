use anyhow::anyhow;
use itertools::Itertools;

use crate::{
    actor::{Actor, Sender, ThreadHandle},
    code::model::{Code, Expression},
};

pub fn start(
    game_output: Sender<GameOutput>,
) -> anyhow::Result<(ThreadHandle, Actor<String>, Actor<GameInput>)> {
    let mut code = Code {
        expressions: vec![],
    };

    print_output(&code);

    let handle_events = Actor::spawn(move |event| {
        match event {
            Event::EditorInput { line } => {
                let command = match parse_command(line) {
                    Ok(command) => command,
                    Err(err) => {
                        println!("{err}");
                        return Ok(());
                    }
                };

                for Command::Insert { color } in command {
                    code.expressions.extend(color.map(|channel| {
                        Expression::LiteralNumber { value: channel }
                    }));
                }

                print_output(&code);
            }
            Event::GameInput(GameInput::RenderingFrame) => {
                // This loop is coupled to the frame rate of the renderer.
            }
        }

        let mut values = Vec::new();
        for expression in &code.expressions {
            let Expression::LiteralNumber { value } = expression;
            values.push(*value);

            let Some((r, g, b, a)) = values.iter().copied().collect_tuple()
            else {
                // Don't have enough values yet to constitute a color.
                continue;
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

pub enum Command {
    Insert { color: [f64; 4] },
}

pub enum GameInput {
    RenderingFrame,
}

pub enum GameOutput {
    SubmitColor { color: [f64; 4] },
}

fn print_output(code: &Code) {
    println!("{:#?}", code);
}

fn parse_command(command: String) -> anyhow::Result<Vec<Command>> {
    let Ok(channels) = command
        .split_whitespace()
        .map(|channel| channel.parse::<f64>())
        .collect::<Result<Vec<_>, _>>()
    else {
        return Err(anyhow!("Can't parse color channels as `f64`."));
    };

    let Some((r, g, b, a)) = channels.into_iter().collect_tuple() else {
        return Err(anyhow!("Unexpected number of color channels."));
    };

    Ok(vec![Command::Insert {
        color: [r, g, b, a],
    }])
}
