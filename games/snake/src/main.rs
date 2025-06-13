mod game;

fn main() -> anyhow::Result<()> {
    let init = Box::new(game::TrialOfTheCaterpillarInit::default());
    crosscut::start_and_wait(init)
}
