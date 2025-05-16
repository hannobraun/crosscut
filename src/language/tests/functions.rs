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
fn binding() {
    // A caller can pass arguments to a function, which can bind them to a name.

    let mut language = Language::new();
    language
        .code("apply")
        .down()
        .code("fn")
        .down()
        .remove_right() // remove the `_` placeholder
        .code("arg") // binding
        .down()
        .code("arg") // function body
        .down()
        .code("127");

    assert_eq!(
        language.step_until_finished().unwrap(),
        Value::Integer { value: 127 },
    );
}

#[test]
fn binding_inner_shadows_outer() {
    let mut language = Language::new();
    language
        .code("apply")
        .down()
        .code("fn")
        .down()
        .remove_right() // remove the `_` placeholder
        .code("arg") // outer binding
        .down()
        .code("apply") // outer function body
        .down()
        .code("fn")
        .down()
        .remove_right() // remove the `_` placeholder
        .code("arg") // inner binding
        .down()
        .code("arg") // inner function body
        .down()
        .code("127") // argument for inner binding
        .down()
        .code("255"); // argument for outer binding

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
