use crate::game_engine::{GameEngine, GameOutput, TerminalInputEvent};

#[test]
fn return_to_edit_mode_after_command_execution() {
    // After a command has been executed, the editor should return to edit mode.

    let mut game_engine = GameEngine::without_editor_ui();

    // Start editing the code.
    game_engine.on_code("12");

    // Execute a command in between editing.
    game_engine
        .on_editor_input(TerminalInputEvent::Escape)
        .unwrap();
    game_engine.on_input("nop");
    game_engine
        .on_editor_input(TerminalInputEvent::Enter)
        .unwrap();

    // Drain output. We're only interested in the result of the next change.
    let _ = game_engine.game_output();

    // Continue editing. Won't work, unless we're back in edit mode.
    game_engine.on_code("7");

    let GameOutput::SubmitColor { color } =
        game_engine.game_output().next().unwrap();
    let c = 127. / 255.;
    assert_eq!(color.as_slice(), [c, c, c, 1.0].as_slice());
}

#[test]
fn abort_command_without_executing_on_escape_key() {
    // When entering a command, the command should be aborted when pressing the
    // escape key. This should return us to edit mode, without a command being
    // executed.

    let mut game_engine = GameEngine::without_editor_ui();

    // Start editing the code.
    game_engine.on_code("12");

    // Enter a command, but abort it.
    game_engine
        .on_editor_input(TerminalInputEvent::Escape)
        .unwrap();
    game_engine.on_input("clear");
    game_engine
        .on_editor_input(TerminalInputEvent::Escape)
        .unwrap();

    // Drain output. We're only interested in the result of the next change.
    let _ = game_engine.game_output();

    // Continue editing. If the previous command was executed, this won't have
    // the desired result.
    game_engine.on_code("7");

    let GameOutput::SubmitColor { color } =
        game_engine.game_output().next().unwrap();
    let c = 127. / 255.;
    assert_eq!(color.as_slice(), [c, c, c, 1.0].as_slice());
}
