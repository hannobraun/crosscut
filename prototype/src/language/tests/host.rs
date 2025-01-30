use crate::language::{
    host::Host,
    instance::Language,
    interpreter::{StepResult, Value},
};

#[test]
fn host_functions() {
    // The host can define functions that Crosscut code can call.

    let mut host = Host::new();
    host.function(0, "halve");

    let mut language = Language::with_host(host);
    language.enter_code("64 halve");

    let output = loop {
        match language.step() {
            StepResult::FunctionApplied { output: _ } => {
                // We're not interested in intermediate results here.
            }
            StepResult::ApplyHostFunction { id, input } => match id {
                0 => {
                    match input {
                        Value::Integer { value } => {
                            language.set_current_value(Value::Integer {
                                value: value / 2,
                            });
                        }
                        input => {
                            panic!("Expected integer. Got instead: {input:?}");
                        }
                    };
                }
                id => {
                    unreachable!("Unexpected host function with ID `{id}`.");
                }
            },
            StepResult::Finished { output } => {
                break output;
            }
            StepResult::Error => {
                panic!("Unexpected runtime error.");
            }
        }
    };

    assert_eq!(output, Value::Integer { value: 32 });
}
