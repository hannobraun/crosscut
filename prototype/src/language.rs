use std::{
    sync::mpsc::{self, SendError},
    thread,
};

pub fn start_in_background() -> mpsc::Receiver<[f64; 4]> {
    let (color_tx, color_rx) = mpsc::sync_channel(0);

    thread::spawn(move || {
        loop {
            // The channel has no buffer, to this is synchronized to the frame
            // rate of the renderer.
            if let Err(SendError(_)) = color_tx.send([0., 0., 0., 1.]) {
                // The other end has hung up. Time for us to shut down too.
                break;
            }
        }
    });

    color_rx
}
