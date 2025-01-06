use itertools::Itertools;

use crate::{
    actor::{Actor, ActorHandle, Sender},
    code::model::{Code, Expression},
};

pub fn start(
    game_output: Sender<GameOutput>,
) -> anyhow::Result<(ActorHandle, Actor<Command>, Actor<GameInput>)> {
    let mut code = Code {
        expressions: vec![],
    };

    print_output(&code);

    let handle_events = Actor::spawn(move |event| {
        match event {
            Event::EditorInput(Command::Insert { color }) => {
                code.expressions.extend(color.map(|channel| {
                    Expression::LiteralNumber { value: channel }
                }));
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
    let command_to_event = Actor::spawn(move |command| {
        events_from_commands.send(Event::EditorInput(command))?;
        Ok(())
    });

    let events_from_input = handle_events.sender;
    let input_to_event = Actor::spawn(move |input| {
        events_from_input.send(Event::GameInput(input))?;
        Ok(())
    });

    Ok((handle_events.handle, command_to_event, input_to_event))
}

enum Event {
    EditorInput(Command),
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
