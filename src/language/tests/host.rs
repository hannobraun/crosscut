use crate::language::{
    code::Type,
    instance::Language,
    packages::{Function, FunctionId, Package},
    runtime::{Effect, StepResult, Value},
};

#[test]
fn host_functions() {
    // The host can define functions that Crosscut code can call.

    let mut package = Package::new();
    package.function(Halve, "halve");

    let mut language = Language::with_package(package);
    language.enter_code("64 halve");

    let output =
        language.step_until_finished_and_handle_host_functions(|id, input| {
            match id.id {
                0 => match input {
                    Value::Integer { value } => {
                        Ok(Value::Integer { value: value / 2 })
                    }
                    input => {
                        panic!("Expected integer. Got instead: {input:?}");
                    }
                },
                id => {
                    unreachable!("Unexpected host function with ID `{id}`.");
                }
            }
        });

    assert_eq!(output, Ok(Value::Integer { value: 32 }));
}

#[test]
fn host_functions_can_trigger_effects() {
    // A host function, instead of returning a value, can trigger an effect. For
    // example to indicate an error.

    let mut package = Package::new();
    package.function(Halve, "halve");

    let mut language = Language::with_package(package);
    language.enter_code("halve");

    let effect = Effect::UnexpectedInput {
        expected: Type::Integer,
        actual: Value::None,
    };
    let output =
        language.step_until_finished_and_handle_host_functions(|id, input| {
            match id.id {
                0 => match input {
                    Value::None => Err(effect),
                    input => {
                        unreachable!("Unexpected input: `{input:?}`");
                    }
                },
                id => {
                    unreachable!("Unexpected host function with ID `{id}`.");
                }
            }
        });

    assert_eq!(output, Err(effect));
    assert_eq!(language.step(), StepResult::EffectTriggered { effect });
}

struct Halve;

impl Function for Halve {
    fn id(&self) -> FunctionId {
        FunctionId { id: 0 }
    }
}
