fn main() -> anyhow::Result<()> {
    let game_start = Box::new(crosscut::PureCrosscutGameStart::default());
    crosscut::start_and_wait(game_start)
}
