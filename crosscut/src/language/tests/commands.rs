use crate::language::{
    editor::EditorCommand, language::Language, runtime::Value,
};

#[test]
fn clear() -> anyhow::Result<()> {
    // The clear command should reset everything to its initial state.
    //
    // This is too dangerous of a capability to keep around long-term, but for
    // right now, it's a useful capability to have during development.

    let mut language = Language::new();

    language.code("12");
    assert_eq!(
        language.step_until_finished().unwrap(),
        Value::Integer { value: 12 },
    );

    language.on_editor_command(EditorCommand::Clear)?;
    assert_eq!(language.step_until_finished().unwrap(), Value::nothing());

    language.code("7");
    assert_eq!(
        language.step_until_finished().unwrap(),
        Value::Integer { value: 7 },
    );

    Ok(())
}
