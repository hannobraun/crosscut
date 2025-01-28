use crate::language::instance::Language;

#[test]
fn number_literal() {
    // A number literal is a function that takes `None` and returns the number
    // it represents.

    let mut language = Language::new();

    language.enter_code("127");
    let output = language.step();

    assert_eq!(output, 127);
}
