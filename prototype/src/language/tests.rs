use pretty_assertions::assert_eq;

use crate::language::{
    compiler::tests::infra::compile, host::Host, interpreter::StepResult,
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
    assert_eq!(interpreter.step(&code), StepResult::Finished { output: 1 },);
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
    assert_eq!(interpreter.step(&code), StepResult::Finished { output: 1 },);
    assert_eq!(interpreter.step(&code), StepResult::Error);
}

#[test]
fn call_to_host_function() {
    // The host can define functions. Those functions take one argument, return
    // one value, and can be called from Crosscut code.

    let mut code = Code::default();

    let host = Host::from_functions(["half"]);
    compile("half 64", &host, &mut code);

    let mut interpreter = Interpreter::new(&code);
    let output = loop {
        match interpreter.step(&code) {
            StepResult::CallToHostFunction { id, input, output } => {
                assert_eq!(id, 0);
                *output = input / 2;
            }
            StepResult::Finished { output } => {
                break output;
            }
            state => {
                panic!("Unexpected state: {state:#?}");
            }
        }
    };

    assert_eq!(output, 32);
}
