use pretty_assertions::assert_eq;

use crate::language::{
    compiler::tests::infra::compile,
    host::Host,
    interpreter::{StepResult, Value},
};

use super::{code::Code, interpreter::Interpreter};

#[test]
fn evaluate_single_expression() {
    // If the program consists only of a single expression, it should be
    // evaluated, and its value returned by the interpreter.

    let host = Host::empty();
    let mut code = Code::default();

    compile("1", &host, &mut code);

    let mut interpreter = Interpreter::new(&code);
    assert_eq!(
        interpreter.step(&code),
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
    let mut code = Code::default();

    compile("1 2", &host, &mut code);

    let mut interpreter = Interpreter::new(&code);
    assert_eq!(
        interpreter.step(&code),
        StepResult::Finished {
            output: Value::Integer { value: 1 }
        },
    );
    assert_eq!(interpreter.step(&code), StepResult::Error);
}

#[test]
fn call_to_host_function() {
    // The host can define functions. Those functions take one argument, return
    // one value, and can be called from Crosscut code.

    let mut code = Code::default();
    let host = TestHost::new();

    compile("half 64", &host.inner, &mut code);
    let output = host.run(&code);

    assert_eq!(output, Value::Integer { value: 32 });
}

#[test]
fn nested_calls_to_host_function() {
    // It is possible use a function call as the argument of another function
    // call.

    let mut code = Code::default();
    let host = TestHost::new();

    compile("half half 64", &host.inner, &mut code);
    let output = host.run(&code);

    assert_eq!(output, Value::Integer { value: 16 });
}

struct TestHost {
    inner: Host,
}

impl TestHost {
    fn new() -> Self {
        Self {
            inner: Host::from_functions(["half"]),
        }
    }

    fn run(&self, code: &Code) -> Value {
        let mut interpreter = Interpreter::new(code);

        loop {
            match interpreter.step(code) {
                StepResult::CallToHostFunction { id, input, output } => {
                    assert_eq!(id, 0);
                    let Value::Integer { value: input } = input;
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
}
