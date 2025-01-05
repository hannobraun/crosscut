use std::{sync::mpsc::SendError, thread};

use crate::{
    channel::{self, actor, Sender},
    cli::Command,
    game_io::{GameInput, GameIo},
};

pub fn start() -> anyhow::Result<(GameIo, Sender<Command>)> {
    let (color_tx, color_rx) = channel::create();

    let (events_tx, events_rx) = channel::create();

    let mut code = Code {
        color: [0., 0., 0., 1.],
    };

    println!("Color: {:?}", code.color);

    thread::spawn(move || {
        loop {
            let Ok(event) = events_rx.recv() else {
                // The other end has hung up. We should shut down too.
                break;
            };

            match event {
                Event::Command(Command::SetColor { color }) => {
                    code.color = color;
                }
                Event::GameInput(GameInput::RenderingFrame) => {
                    // This loop is coupled to the frame rate of the renderer.
                }
            }

            if let Err(SendError(_)) = color_tx.send(code.color) {
                // The other end has hung up. Time for us to shut down too.
                break;
            }
        }
    });

    let events_from_input = events_tx.clone();
    let events_from_commands = events_tx;

    let input = actor(move |input| {
        events_from_input.send(Event::GameInput(input)).is_ok()
    });
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
