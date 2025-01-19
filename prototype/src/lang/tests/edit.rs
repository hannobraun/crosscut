use crate::lang::{
    self,
    host::Host,
    interpreter::{StepResult, Value},
};

#[test]
fn update_on_each_character() {
    // The editor should compile the code and update the interpreter on every
    // character it receives.

    let host = Host::empty();
    let mut lang = lang::Instance::new();

    lang.on_command("edit", &host);

    lang.on_char('1', &host);
    assert_eq!(
        lang.interpreter.step(&lang.code),
        StepResult::Finished {
            output: Value::Integer { value: 1 }
        },
    );

    lang.on_char('2', &host);
    assert_eq!(
        lang.interpreter.step(&lang.code),
        StepResult::Finished {
            output: Value::Integer { value: 12 }
        },
    );
}
