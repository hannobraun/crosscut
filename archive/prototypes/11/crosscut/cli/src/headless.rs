use std::{path::PathBuf, time::Instant};

use crosscut_game_engine::{
    command::Command, display::NUM_PIXEL_BYTES, game_engine::GameEngine,
};
use rand::random;

use crate::build_game::build_game_once;

pub async fn run(games_path: PathBuf) -> anyhow::Result<()> {
    let code = build_game_once(&games_path.join("snake")).await?;

    let mut pixels = [0; NUM_PIXEL_BYTES];
    let mut game_engine = GameEngine::new();

    game_engine.on_command(Command::UpdateCode {
        instructions: code.instructions,
    });

    let start_of_game = Instant::now();
    let mut start_of_loop;
    let mut start_of_frame = Instant::now();

    let mut times_net = Measurements::default();
    let mut times_gross = Measurements::default();

    while !game_engine.runtime.state().has_finished() {
        start_of_loop = Instant::now();

        while game_engine.push_random(random()) {}

        if !game_engine.run_until_end_of_frame(
            start_of_game.elapsed().as_secs_f64(),
            &mut pixels,
        ) {
            // Game engine decided that it's not time to run another frame yet.
            continue;
        }

        if let Some(effect) = game_engine.runtime.effect().inspect() {
            eprintln!("Unhandled effect: {effect:#?}");
            eprintln!("Current stack:\n{:#?}", game_engine.runtime.stack());
            break;
        }

        let frame_time_net = start_of_loop.elapsed().as_millis();
        let frame_time_gross = start_of_frame.elapsed().as_millis();

        times_net.measure(frame_time_net);
        times_gross.measure(frame_time_gross);

        if times_gross.total_ms >= 1000 {
            let avg_net = times_net.total_ms / times_net.num;
            let max_net = times_net.max_ms.unwrap();
            let min_net = times_net.min_ms.unwrap();

            let avg_gross = times_gross.total_ms / times_gross.num;
            let max_gross = times_gross.max_ms.unwrap();
            let min_gross = times_gross.min_ms.unwrap();

            eprintln!(
                "avg: {avg_net} / {avg_gross} ms; \
                max: {max_net} / {max_gross} ms; \
                min: {min_net} / {min_gross} ms",
            );

            times_net = Measurements::default();
            times_gross = Measurements::default();
        }

        // Do this after the whole frame time bookkeeping, so it's not
        // influencing the performance measurement.
        start_of_frame = Instant::now();
    }

    Ok(())
}

#[derive(Default)]
pub struct Measurements {
    total_ms: u128,
    min_ms: Option<u128>,
    max_ms: Option<u128>,
    num: u128,
}

impl Measurements {
    fn measure(&mut self, time_ms: u128) {
        self.total_ms += time_ms;
        self.num += 1;

        if let Some(min) = self.min_ms {
            if time_ms < min {
                self.min_ms = Some(time_ms);
            }
        } else {
            self.min_ms = Some(time_ms);
        }
        if let Some(max) = self.max_ms {
            if time_ms > max {
                self.max_ms = Some(time_ms);
            }
        } else {
            self.max_ms = Some(time_ms);
        }
    }
}
