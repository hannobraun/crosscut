use crate::{
    channel::{self, actor, Sender},
    cli::Command,
    game_io::{GameInput, GameIo},
};

pub fn start() -> anyhow::Result<(GameIo, Sender<Command>)> {
    let (color_tx, color_rx) = channel::create();

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

    Ok((
        GameIo {
            input,
            output: color_rx,
        },
        commands,
    ))
}

enum Event {
    Command(Command),
    GameInput(GameInput),
}

struct Code {
    color: [f64; 4],
}
