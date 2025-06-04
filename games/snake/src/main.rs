mod game;

fn main() -> anyhow::Result<()> {
    let game = Box::new(game::Snake::default());
    crosscut::start_and_wait(game)
}
