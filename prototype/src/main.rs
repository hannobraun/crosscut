mod game_engine;
mod io;
mod threads;

fn main() -> anyhow::Result<()> {
    let (game_input_tx, game_input_rx) = threads::channel();
    let (game_output_tx, game_output_rx) = threads::channel();

    dbg!(game_input_rx.try_recv())?;
    game_output_tx
        .send(game_engine::GameOutput::SubmitColor { color: [1.; 4] })?;

    io::game_engine::start_and_wait(game_input_tx, game_output_rx)?;

    Ok(())
}
