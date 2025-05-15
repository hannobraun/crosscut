use crate::language::language::Language;

#[test]
fn do_not_step_beyond_errors() {
    // If there's an error in the code, the interpreter should never step beyond
    // that, if it encounters it.

    let mut language = Language::new();
    language.code("unresolved");

    assert!(language.step().is_error());
    assert!(language.step().is_error());
}
