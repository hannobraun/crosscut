mod game;

fn main() -> anyhow::Result<()> {
    let game_start = Box::new(game::SnakeStart::default());
    crosscut::start_and_wait(game_start)
}
