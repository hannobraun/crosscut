use crate::language::{
    language::Language,
    runtime::{Effect, RuntimeState, Value},
};

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
        .down() // navigate past the parameter
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
        .down() // navigate past the function body
        .code("255"); // argument for outer binding

    assert_eq!(
        language.step_until_finished().unwrap(),
        Value::Integer { value: 127 },
    );
}

#[test]
fn binding_inner_does_not_interfere_with_outer() {
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
        .code("+")
        .down()
        .code("tuple")
        .down()
        .code("apply")
        .down()
        .code("fn")
        .down()
        .remove_right() // remove the `_` placeholder
        .code("arg") // inner binding
        .down()
        .code("arg") // inner function body; refers to inner binding
        .down()
        .down() // navigate past the inner function body
        .code("1") // argument for inner binding
        .down()
        .code("arg") // refers to outer binding
        .down()
        .down() // navigate past the tuple
        .down() // navigate past the outer function body
        .code("2"); // argument for outer binding

    assert_eq!(
        language.step_until_finished().unwrap(),
        Value::Integer { value: 3 },
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
        .down() // navigate past the parameter
        .code("apply")
        .down()
        .code("self");

    // This is a rather large number of steps, given the length of the program.
    // Should be proof enough, that it's recursing.
    for _ in 0..1024 {
        assert!(language.step().is_running());
    }
}

#[test]
fn tail_call_arguments_have_access_to_function_argument() {
    // The arguments of a tail call can access the arguments of the function.

    let mut language = Language::new();
    language
        .code("apply")
        .down()
        .code("fn")
        .down()
        .code("arg")
        .down()
        .code("apply")
        .down()
        .code("test")
        .down()
        .code("arg")
        .down()
        .code("apply")
        .down()
        .code("self")
        .down()
        .code("arg")
        .down()
        .down() // navigate past function body
        .code("127");

    let mut saw_argument = 0;

    for _ in 0..1024 {
        if saw_argument >= 2 {
            // Saw it twice, which means the `self` call successfully passed it
            // into the next iteration.
            //
            // Return from the test, to circumvent the panic below.
            return;
        }

        if let RuntimeState::Effect {
            effect: Effect::ApplyProvidedFunction { name, input },
            ..
        } = language.step()
        {
            assert_eq!(name, "test");
            assert_eq!(input, &Value::Integer { value: 127 });

            saw_argument += 1;

            language.provide_host_function_output(Value::nothing());
        }
    }

    panic!();
}
