use std::thread;

use crossbeam_channel::{select, SendError};

use crate::{
    channel::{self, Receiver},
    cli::Command,
    game_io::{GameInput, GameIo},
};

pub fn start(commands: Receiver<Command>) -> anyhow::Result<GameIo> {
    // Specifying type parameter explicitly, to work around this bug in
    // rust-analyzer: https://github.com/rust-lang/rust-analyzer/issues/15984
    let (input_tx, input_rx) = channel::create::<GameInput>();
    let (color_tx, color_rx) = channel::create();

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

            let event = select! {
                recv(input_rx) -> game_input => {
                    let Ok(game_input) = game_input else {
                        // The other end has hung up. We should shut down too.
                        break;
                    };

                    Event::GameInput(game_input)
                }
                recv(commands) -> command => {
                    let Ok(command) = command else {
                        // The other end has hung up. We should shut down too.
                        break;
                    };

                    Event::Command(command)
                }
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
        input: input_tx,
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
