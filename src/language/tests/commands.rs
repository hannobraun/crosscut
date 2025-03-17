use crate::language::{
    editor::EditorCommand, language::Language, runtime::Value,
    tests::infra::StepUntilFinishedResultExt,
};

#[test]
fn clear() {
    // The clear command should reset everything to its initial state.
    //
    // This is too dangerous of a capability to keep around long-term, but for
    // right now, it's a useful capability to have during development.

    let mut language = Language::new();

    language.on_code("12");
    assert_eq!(
        language.step_until_finished().expect_value(),
        Value::Integer { value: 12 },
    );

    language.on_command(EditorCommand::Clear);
    assert_eq!(
        language.step_until_finished().expect_value(),
        Value::Nothing,
    );

    language.on_code("7");
    assert_eq!(
        language.step_until_finished().expect_value(),
        Value::Integer { value: 7 },
    );
}
