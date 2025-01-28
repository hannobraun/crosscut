use crate::language::instance::Language;

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
