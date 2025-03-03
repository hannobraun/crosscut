use crate::language::{
    code::Type,
    instance::Language,
    packages::{Function, Package},
    runtime::{Effect, RuntimeState, Value},
    tests::functions::IntoFunctionBody,
};

#[test]
fn host_functions() {
    // The host can define functions that Crosscut code can call.

    let package = Package::new().with_function(Halve);

    let mut language = Language::new();
    language.with_package(&package);

    language.on_code("64 halve");

    let output =
        language.step_until_finished_and_handle_host_functions(|id, input| {
            match package.function_by_id(id) {
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

    let package = Package::new().with_function(Halve);

    let mut language = Language::new();
    language.with_package(&package);

    language.on_code("halve");

    let effect = Effect::UnexpectedInput {
        expected: Type::Integer,
        actual: Value::Nothing,
    };
    let output =
        language.step_until_finished_and_handle_host_functions(|id, input| {
            match package.function_by_id(id) {
                Halve => match input {
                    Value::Nothing => Err(effect.clone()),
                    input => {
                        unreachable!("Unexpected input: `{input:?}`");
                    }
                },
            }
        });

    assert_eq!(output.as_ref(), Err(&effect));
    assert!(matches!(language.step(), RuntimeState::Effect { .. }));
}

#[test]
fn host_functions_can_inject_opaque_value() {
    // A host function can define an opaque value and inject that into a
    // function.

    let package = Package::new().with_function(ObserveOpaqueValue);

    let mut language = Language::new();
    language.with_package(&package);

    language.on_code("observe_opaque_value fn");

    let path = match language.step_until_finished().into_function_body() {
        Ok(path) => path,
        output => {
            panic!("Unexpected output: {output:?}");
        }
    };

    let opaque_value = Value::Opaque {
        id: 0,
        display: "opaque",
    };
    language.push_context(path, opaque_value.clone());

    let mut value_observed = false;
    let output =
        language.step_until_finished_and_handle_host_functions(|id, input| {
            match package.function_by_id(id) {
                ObserveOpaqueValue => {
                    value_observed = input == &opaque_value;
                    Ok(input.clone())
                }
            }
        });

    assert!(value_observed);
    assert_eq!(output, Ok(opaque_value));
}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
struct Halve;

impl Function for Halve {
    fn name(&self) -> &str {
        "halve"
    }
}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
struct ObserveOpaqueValue;

impl Function for ObserveOpaqueValue {
    fn name(&self) -> &str {
        "observe_opaque_value"
    }
}
