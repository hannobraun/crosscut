use pretty_assertions::assert_eq;

use crate::core::{
    self,
    code::Code,
    compiler::tests::infra::compile_all,
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

fn compile_and_run(input: &str) -> Value {
    let host = Host::from_functions(["half"]);
    let mut code = Code::default();

    compile_all(input, &host, &mut code);
    let mut interpreter = Interpreter::new(&code);
    run(&code, &mut interpreter)
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
