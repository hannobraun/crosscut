use crate::language::{language::Language, runtime::Value};

#[test]
fn define_and_evaluate() {
    // It is possible to define a function using a function literal, return that
    // function from the program, then tell the language to evaluate it.

    let mut language = Language::new();
    language
        .code("apply")
        .down()
        .code("fn")
        .down()
        .code("_")
        .down()
        .code("127");

    assert_eq!(
        language.step_until_finished().unwrap(),
        Value::Integer { value: 127 },
    );
}

#[test]
fn self_recursion() {
    // A function can recurse using the `self` keyword.

    let mut language = Language::new();
    language
        .code("apply")
        .down()
        .code("fn")
        .down()
        .code("_")
        .down()
        .code("apply")
        .down()
        .code("self");

    // This is a rather large number of steps, given the length of the program.
    // Should be proof enough, that it's recursing.
    for _ in 0..1024 {
        assert!(language.step().is_running());
    }
}
