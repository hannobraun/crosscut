mod game_engine;

fn main() {
    dbg!(game_engine::GameInput::RenderingFrame);
    let game_engine::GameOutput::SubmitColor { color } =
        game_engine::GameOutput::SubmitColor { color: [1.; 4] };
    dbg!(color);
}
