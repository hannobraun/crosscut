use crate::actor::{actor, Sender};

#[allow(clippy::type_complexity)] // temporary; should be removed any commit now
pub fn start(
    color_tx: Sender<[f64; 4]>,
) -> anyhow::Result<(Sender<GameInput>, Sender<Command>)> {
    let mut code = Code {
        color: [0., 0., 0., 1.],
    };

    println!("Color: {:?}", code.color);

    let events = actor(move |event| {
        match event {
            Event::Command(Command::SetColor { color }) => {
                code.color = color;
            }
            Event::GameInput(GameInput::RenderingFrame) => {
                // This loop is coupled to the frame rate of the renderer.
            }
        }

        color_tx.send(code.color).is_ok()
    });

    let events_from_input = events.clone();
    let input = actor(move |input| {
        events_from_input.send(Event::GameInput(input)).is_ok()
    });

    let events_from_commands = events;
    let commands = actor(move |command| {
        events_from_commands.send(Event::Command(command)).is_ok()
    });

    Ok((input, commands))
}

struct Code {
    color: [f64; 4],
}

enum Event {
    Command(Command),
    GameInput(GameInput),
}

pub enum Command {
    SetColor { color: [f64; 4] },
}

pub enum GameInput {
    RenderingFrame,
}
