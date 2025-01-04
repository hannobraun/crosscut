use std::thread;

use tokio::{
    runtime::Runtime,
    sync::mpsc::{self, error::SendError},
};

pub fn start_in_background() -> anyhow::Result<GameIo> {
    let runtime = Runtime::new()?;

    let (render_tx, mut render_rx) = mpsc::unbounded_channel();
    let (color_tx, color_rx) = mpsc::channel(1);

    thread::spawn(move || {
        runtime.block_on(async {
            let color = [0., 0., 0., 1.];

            println!("Color: {color:?}");

            loop {
                // The channel has no buffer, so this is synchronized to the
                // frame rate of the renderer.
                if let Err(SendError(_)) = color_tx.send(color).await {
                    // The other end has hung up. Time for us to shut down too.
                    break;
                }

                match render_rx.recv().await {
                    Some(GameInput::RenderingFrame) => {
                        // This loop is coupled to the frame rate of the
                        // renderer.
                    }
                    None => {
                        // The other end has hung up. We should shut down too.
                        break;
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

pub struct GameIo {
    pub input: mpsc::UnboundedSender<GameInput>,
    pub output: mpsc::Receiver<[f64; 4]>,
}

pub enum GameInput {
    RenderingFrame,
}
