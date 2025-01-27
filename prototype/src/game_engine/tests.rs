use crate::game_engine::{GameEngine, GameOutput};

#[test]
fn enter_expression_and_expect_game_output() {
    // This is basically just a high-level smoke that, to make sure the most
    // basic interaction works: If the developer enters an expression into the
    // editor, we expect that to get evaluated and result in game output.

    let mut game_engine = GameEngine::without_editor_ui();

    game_engine.enter_code("127");

    let game_output = game_engine
        .game_output()
        .map(|GameOutput::SubmitColor { color }| color)
        .collect::<Vec<_>>();
    let expected = [1, 12, 127]
        .map(|value| {
            let value = value as f64 / 255.;
            [value, value, value, 1.]
        })
        .to_vec();

    assert_eq!(game_output, expected);
}

#[test]
fn expect_clear_command_to_clear_previously_entered_code() {
    // If the `clear` command executes, previously entered code should have no
    // effect.

    let mut game_engine = GameEngine::without_editor_ui();

    game_engine.enter_code("12");

    game_engine.enter_command_mode();
    game_engine.enter_command("clear");
    game_engine.execute_command();

    game_engine.enter_code("7");

    let GameOutput::SubmitColor { color } =
        game_engine.game_output().last().unwrap();
    let c = 7. / 255.;
    assert_eq!(color, [c, c, c, 1.0]);
}
