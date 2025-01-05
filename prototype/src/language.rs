use std::{sync::mpsc::SendError, thread};

use crate::{
    channel::{self, actor, Receiver},
    cli::Command,
    game_io::{GameInput, GameIo},
};

pub fn start(commands_rx: Receiver<Command>) -> anyhow::Result<GameIo> {
    let (color_tx, color_rx) = channel::create();

    let (events_tx, events_rx) = channel::create();

    let events_from_input = events_tx.clone();
    let events_from_commands = events_tx;

    let input = actor(move |input| {
        events_from_input.send(Event::GameInput(input)).is_ok()
    });

    thread::spawn(move || {
        while let Ok(command) = commands_rx.recv() {
            if let Err(SendError(_)) =
                events_from_commands.send(Event::Command(command))
            {
                break;
            };
        }
    });

    thread::spawn(move || {
        let mut code = Code {
            color: [0., 0., 0., 1.],
        };

        println!("Color: {:?}", code.color);

        loop {
            // The channel has no buffer, so this is synchronized to the
            // frame rate of the renderer.
            if let Err(SendError(_)) = color_tx.send(code.color) {
                // The other end has hung up. Time for us to shut down too.
                break;
            }

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
        }
    });

    Ok(GameIo {
        input,
        output: color_rx,
    })
}

enum Event {
    Command(Command),
    GameInput(GameInput),
}

struct Code {
    color: [f64; 4],
}
