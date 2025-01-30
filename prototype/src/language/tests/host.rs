use crate::language::{host::Host, instance::Language, interpreter::Value};

#[test]
fn host_functions() {
    // The host can define functions that Crosscut code can call.

    let mut host = Host::new();
    host.function(0, "halve");

    let mut language = Language::with_host(host);
    language.enter_code("64 halve");

    let output =
        language.step_until_finished_and_handle_host_functions(|id, input| {
            match id {
                0 => match input {
                    Value::Integer { value } => {
                        Value::Integer { value: value / 2 }
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

    assert_eq!(output, Value::Integer { value: 32 });
}
