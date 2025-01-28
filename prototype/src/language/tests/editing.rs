use crate::language::{editor::EditorInputEvent, instance::Language};

#[test]
fn update_on_every_character() {
    // The editor should compile the code on every new character. If the program
    // has finished running, as is the case here, it should also reset the
    // interpreter, so the next step will run the new code.

    let mut language = Language::new();

    language.enter_code("1");
    assert_eq!(language.step(), Some(1));

    language.enter_code("2");
    assert_eq!(language.step(), Some(12));
}

#[test]
fn update_after_removing_character() {
    // Removing a character should have an immediate effect on the program, just
    // like adding one.

    let mut language = Language::new();

    language.enter_code("127");
    assert_eq!(language.step(), Some(127));

    language.on_input(EditorInputEvent::RemoveCharacterLeft);
    assert_eq!(language.step(), Some(12));

    language.on_input(EditorInputEvent::RemoveCharacterLeft);
    assert_eq!(language.step(), Some(1));
}

#[test]
fn edit_at_cursor_location() {
    // The editor should edit the code wherever the cursor is currently located.

    let mut language = Language::new();

    language.enter_code("2");
    assert_eq!(language.step(), Some(2));

    language.on_input(EditorInputEvent::MoveCursorLeft);
    language.enter_code("1");
    assert_eq!(language.step(), Some(12));
}
