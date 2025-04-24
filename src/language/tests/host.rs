use crate::language::{
    code::Type,
    language::Language,
    packages::Function,
    runtime::{Effect, Value},
};

#[test]
fn host_functions() {
    // The host can define functions that Crosscut code can call.

    let mut language = Language::new();
    let package = language.packages_mut().new_package([Halve]);

    language.code("halve 64");

    let output =
        language.step_until_finished_and_handle_host_functions(|id, input| {
            match package.function_by_id(id).unwrap() {
                Halve => match input {
                    Value::Integer { value } => {
                        Ok(Value::Integer { value: value / 2 })
                    }
                    input => {
                        panic!("Expected integer. Got instead: {input:?}");
                    }
                },
            }
        });

    assert_eq!(output, Ok(Value::Integer { value: 32 }));
}

#[test]
fn host_functions_can_trigger_effects() {
    // A host function, instead of returning a value, can trigger an effect. For
    // example to indicate an error.

    let mut language = Language::new();
    let package = language.packages_mut().new_package([Halve]);

    language.code("halve");

    let effect = Effect::UnexpectedInput {
        expected: Type::Integer,
        actual: Value::nothing(),
    };
    let output =
        language.step_until_finished_and_handle_host_functions(|id, input| {
            match package.function_by_id(id).unwrap() {
                Halve => match input {
                    value if value.is_nothing() => Err(effect.clone()),
                    input => {
                        unreachable!("Unexpected input: `{input:?}`");
                    }
                },
            }
        });

    assert_eq!(output.as_ref(), Err(&effect));
    assert!(language.step().is_effect());
}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
struct Halve;

impl Function for Halve {
    fn name(&self) -> &str {
        "halve"
    }
}
