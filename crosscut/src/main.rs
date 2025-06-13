fn main() -> anyhow::Result<()> {
    let game_start = Box::new(crosscut::PureCrosscutGameInit::default());
    crosscut::start_and_wait(game_start)
}
