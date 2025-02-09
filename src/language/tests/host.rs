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
    package.function(Halve);

    let mut language = Language::with_package(package);
    language.enter_code("64 halve");

    let output =
        language.step_until_finished_and_handle_host_functions(|id, input| {
            match Function::from_verified_id(id) {
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

    assert_eq!(
        output.map(|value| value.inner),
        Ok(Value::Integer { value: 32 }),
    );
}

#[test]
fn host_functions_can_trigger_effects() {
    // A host function, instead of returning a value, can trigger an effect. For
    // example to indicate an error.

    let mut package = Package::new();
    package.function(Halve);

    let mut language = Language::with_package(package);
    language.enter_code("halve");

    let effect = Effect::UnexpectedInput {
        expected: Type::Integer,
        actual: Value::None,
    };
    let output =
        language.step_until_finished_and_handle_host_functions(|id, input| {
            match Function::from_verified_id(id) {
                Halve => match input {
                    Value::None => Err(effect),
                    input => {
                        unreachable!("Unexpected input: `{input:?}`");
                    }
                },
            }
        });

    assert_eq!(output, Err(effect));
    assert_eq!(language.step(), StepResult::EffectTriggered { effect });
}

struct Halve;

impl Function for Halve {
    fn from_id(FunctionId { id }: FunctionId) -> Option<Self> {
        match id {
            0 => Some(Self),
            _ => None,
        }
    }

    fn id(&self) -> FunctionId {
        FunctionId { id: 0 }
    }

    fn name(&self) -> &str {
        "halve"
    }
}
