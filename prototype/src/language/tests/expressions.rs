use crate::language::instance::Language;

#[test]
fn evaluate_integer() {
    let mut language = Language::new();

    language.enter_code("127");
    let output = language.run_until_finished();

    assert_eq!(output, 127);
}
