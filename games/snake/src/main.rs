fn main() -> anyhow::Result<()> {
    let game = Box::new(crosscut::PureCrosscutGame);
    crosscut::start_and_wait(game)
}
