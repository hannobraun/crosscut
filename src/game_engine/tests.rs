use crate::game_engine::{GameEngine, GameOutput};

// All tests in this module are currently broken. I (Hanno Braun) did that
// accidentally, when making a change to the game engine, forgetting to re-run
// the tests.
//
// What's important to note though, is that the functionality covered by the
// tests itself is not broken (supposedly; there no longer are any tests 😂).
// What's broken is the mechanism that all of the tests are using to observe the
// game engine's behavior.
//
// I decided not to fix this immediately. I'm currently working through an
// important decision that could have a big effect on the syntax. In that case,
// fixing these tests would be absolutely trivial.
//
// Right now, it wouldn't be. Any fix would make the tests more complicated. So
// I think it's fine to wait for a bit, as that syntax decision is the higher
// priority anyway.

#[test]
fn enter_expression_and_expect_game_output() {
    // This is basically just a high-level smoke that, to make sure the most
    // basic interaction works: If the developer enters an expression into the
    // editor, we expect that to get evaluated and result in game output.

    let mut game_engine = GameEngine::without_editor_ui();

    game_engine.enter_code("color 127");

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

    game_engine.enter_code("color 12");

    game_engine.enter_command_mode();
    game_engine.enter_command("clear");
    game_engine.execute_command();

    game_engine.enter_code("color 7");

    let GameOutput::SubmitColor { color } =
        game_engine.game_output().last().unwrap();
    let c = 7. / 255.;
    assert_eq!(color, [c, c, c, 1.0]);
}

#[test]
fn expect_aborted_command_to_have_no_effect() {
    // If a command aborts instead of executing, this should have no effect.

    let mut game_engine = GameEngine::without_editor_ui();

    game_engine.enter_code("color 12");

    game_engine.enter_command_mode();
    game_engine.enter_command("clear");
    game_engine.abort_command();

    game_engine.enter_code("7");

    let GameOutput::SubmitColor { color } =
        game_engine.game_output().last().unwrap();
    let c = 127. / 255.;
    assert_eq!(color, [c, c, c, 1.0]);
}
