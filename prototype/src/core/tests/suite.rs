use pretty_assertions::assert_eq;

use crate::core::{
    self,
    host::Host,
    interpreter::{StepResult, Value},
};

#[test]
fn evaluate_single_expression() {
    // If the program consists only of a single expression, it should be
    // evaluated, and its value returned by the interpreter.

    let host = Host::empty();
    let mut core = core::Instance::new();

    core.edit("1", &host);

    assert_eq!(
        core.interpreter.step(&core.code),
        StepResult::Finished {
            output: Value::Integer { value: 1 }
        },
    );
}

#[test]
fn code_after_expression_is_an_error() {
    // An expression returns a value. That value can be returned when the
    // program finishes, or it can be used as the argument of a function call.
    //
    // Either way, any code that comes after an expression makes no sense, and
    // is an error.

    let host = Host::empty();
    let mut core = core::Instance::new();

    core.edit("1 2", &host);

    assert_eq!(
        core.interpreter.step(&core.code),
        StepResult::Finished {
            output: Value::Integer { value: 1 }
        },
    );
    assert_eq!(core.interpreter.step(&core.code), StepResult::Error);
}
