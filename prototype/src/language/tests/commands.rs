use crate::language::{
    editor::EditorCommand, instance::Language, interpreter::Value,
};

#[test]
fn clear() {
    // The clear command should reset everything to its initial state.
    //
    // This is too dangerous of a capability to keep around long-term, but for
    // right now, it's a useful capability to have during development.

    let mut language = Language::new();

    language.enter_code("12");
    assert_eq!(language.step(), Value::Integer { value: 12 });

    language.on_command(EditorCommand::Clear);
    assert_eq!(language.step(), Value::None);

    language.enter_code("7");
    assert_eq!(language.step(), Value::Integer { value: 7 });
}
