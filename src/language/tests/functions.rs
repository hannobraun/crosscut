use crate::language::{language::Language, runtime::Value};

#[test]
fn define_and_evaluate() {
    // It is possible to define a function using a function literal, return that
    // function from the program, then tell the language to evaluate it.

    let mut language = Language::from_code("127 fn eval");
    assert_eq!(
        language.step_until_finished(),
        Ok(Value::Integer { value: 127 }),
    );
}

#[test]
fn self_recursion() {
    // A function can recurse using the `self` keyword.

    let mut language = Language::from_code("self fn eval");

    // This is a rather large number of steps, given the length of the program.
    // Should be proof enough, that it's recursing.
    for _ in 0..1024 {
        assert!(language.step().is_running());
    }
}

#[test]
fn empty_function() {
    // If an `fn` node doesn't have a child, an empty syntax node should be
    // created as a child for it.

    let mut language = Language::from_code("fn eval");
    assert_eq!(language.step_until_finished(), Ok(Value::Nothing));
}
