fn main() -> anyhow::Result<()> {
    let game = Box::new(crosscut::PureCrosscutGame::default());
    crosscut::start_and_wait(game)
}
