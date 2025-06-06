fn main() -> anyhow::Result<()> {
    let game = Box::new(crosscut::PureCrosscutGameStart::default());
    crosscut::start_and_wait(game)
}
