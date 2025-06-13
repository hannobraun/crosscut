fn main() -> anyhow::Result<()> {
    let init = Box::new(crosscut::PureCrosscutGameInit::default());
    crosscut::start_and_wait(init)
}
