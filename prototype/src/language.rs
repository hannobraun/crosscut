use std::thread;

use tokio::{
    runtime::Runtime,
    sync::mpsc::{self, error::SendError},
};

use crate::game_io::{GameInput, GameIo};

pub fn start_in_background() -> anyhow::Result<GameIo> {
    let runtime = Runtime::new()?;

    let (render_tx, mut render_rx) = mpsc::unbounded_channel();
    let (color_tx, color_rx) = mpsc::unbounded_channel();

    thread::spawn(move || {
        runtime.block_on(async {
            let color = [0., 0., 0., 1.];

            println!("Color: {color:?}");

            loop {
                // The channel has no buffer, so this is synchronized to the
                // frame rate of the renderer.
                if let Err(SendError(_)) = color_tx.send(color) {
                    // The other end has hung up. Time for us to shut down too.
                    break;
                }

                let event = {
                    let Some(game_input) = render_rx.recv().await else {
                        // The other end has hung up. We should shut down too.
                        break;
                    };

                    Event::GameInput(game_input)
                };

                match event {
                    Event::GameInput(GameInput::RenderingFrame) => {
                        // This loop is coupled to the frame rate of the
                        // renderer.
                    }
                }
            }
        });
    });

    Ok(GameIo {
        input: render_tx,
        output: color_rx,
    })
}

enum Event {
    GameInput(GameInput),
}
