mod game_engine;
mod threads;

fn main() -> anyhow::Result<()> {
    let (game_input_tx, game_input_rx) = threads::channel();
    let (game_output_tx, game_output_rx) = threads::channel();

    game_input_tx.send(game_engine::GameInput::RenderingFrame)?;
    dbg!(game_input_rx.try_recv())?;

    game_output_tx
        .send(game_engine::GameOutput::SubmitColor { color: [1.; 4] })?;
    if let Some(game_output) = game_output_rx.try_recv()? {
        let game_engine::GameOutput::SubmitColor { color } = game_output;
        dbg!(color);
    }

    Ok(())
}
