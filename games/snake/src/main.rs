mod game;

fn main() -> anyhow::Result<()> {
    let game = Box::new(game::SnakeStart::default());
    crosscut::start_and_wait(game)
}
