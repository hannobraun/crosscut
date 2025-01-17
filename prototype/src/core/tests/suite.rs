use pretty_assertions::assert_eq;

use crate::core::{
    self,
    code::Code,
    host::Host,
    interpreter::{Interpreter, StepResult, Value},
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

#[test]
fn call_to_host_function() {
    // The host can define functions. Those functions take one argument, return
    // one value, and can be called from Crosscut code.

    let output = compile_and_run("half 64");
    assert_eq!(output, Value::Integer { value: 32 });
}

#[test]
fn nested_calls_to_host_function() {
    // It is possible use a function call as the argument of another function
    // call.

    let output = compile_and_run("half half 64");
    assert_eq!(output, Value::Integer { value: 16 });
}

fn compile_and_run(input_code: &str) -> Value {
    let host = Host::from_functions(["half"]);
    let mut core = core::Instance::new();

    core.edit(input_code, &host);

    // The editor already resets the interpreter, but only if it's not running.
    // The tests that use this function provide multiple tokens. The editor
    // resets the interpreter after the first token, and then it _is_ running,
    // but never actually stepped until all code has been provided.
    //
    // As a result, the interpreter then tries to run the fragment resulting
    // from that first token, which doesn't have arguments yet, resulting in an
    // error.
    //
    // So we need to reset again manually, once all code has been provided, so
    // the interpreter actually runs the most recent version of the code.
    core.interpreter.reset(&core.code);

    run(&core.code, &mut core.interpreter)
}

fn run(code: &Code, interpreter: &mut Interpreter) -> Value {
    loop {
        match interpreter.step(code) {
            StepResult::CallToHostFunction { id, input, output } => {
                assert_eq!(id, 0);

                let Value::Integer { value: input } = input;
                let Value::Integer { value: output } = output;

                *output = input / 2;
            }
            StepResult::Finished { output } => {
                break output;
            }
            state => {
                panic!("Unexpected state: {state:#?}");
            }
        }
    }
}
