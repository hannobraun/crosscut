use crate::language::{
    code::Type,
    language::Language,
    runtime::{Effect, Value},
};

#[test]
fn host_functions() {
    // The host can define functions that Crosscut code can call.

    let mut language = Language::new();
    language
        .code("apply")
        .down()
        .code("halve")
        .down()
        .code("64");

    let output = language.step_until_finished_and_handle_host_functions(
        |name, input| match name {
            "halve" => match input {
                Value::Integer { value } => {
                    Ok(Value::Integer { value: value / 2 })
                }
                input => {
                    panic!("Expected integer. Got instead: {input:?}");
                }
            },
            _ => {
                panic!("Unexpected function: `{name}`");
            }
        },
    );

    assert_eq!(output, Ok(Value::Integer { value: 32 }));
}

#[test]
fn host_functions_can_trigger_effects() {
    // A host function, instead of returning a value, can trigger an effect. For
    // example to indicate an error.

    let mut language = Language::new();
    language.code("apply").down().code("halve");

    let effect = Effect::UnexpectedInput {
        expected: Type::Integer,
        actual: Value::nothing(),
    };
    let output = language.step_until_finished_and_handle_host_functions(
        |name, input| match name {
            "halve" => match input {
                value if value.is_nothing() => Err(effect.clone()),
                input => {
                    unreachable!("Unexpected input: `{input:?}`");
                }
            },
            _ => {
                panic!("Unexpected function: `{name}`");
            }
        },
    );

    assert_eq!(output.as_ref(), Err(&effect));
    assert!(language.step().is_effect());
}
