use pretty_assertions::assert_eq;

use crate::language::interpreter::InterpreterState;

use super::{code::Code, compiler, host::Host, interpreter::Interpreter};

#[test]
fn evaluate_single_expression() {
    // If the program consists only of a single expression, it should be
    // evaluated, and its value returned by the interpreter.

    let host = Host::without_functions();
    let mut code = Code::default();
    let mut interpreter = Interpreter::default();

    compile("1", &host, &mut code);

    assert_eq!(
        interpreter.step(&code),
        InterpreterState::Finished { output: 1. },
    );
}

#[test]
fn code_after_expression_is_an_error() {
    // An expression returns a value. That value can only be returned from the
    // interpreter (which would mean the program has finished), or it can be
    // used as the argument of a function call.
    //
    // Any code that comes after an expression makes no sense, and is an error.

    let host = Host::without_functions();
    let mut code = Code::default();
    let mut interpreter = Interpreter::default();

    compile("1 2", &host, &mut code);

    assert_eq!(
        interpreter.step(&code),
        InterpreterState::Finished { output: 1. },
    );
    assert_eq!(interpreter.step(&code), InterpreterState::Error);
}

#[test]
fn call_to_host_function() {
    // The host can define functions which Crosscut code can call. This should
    // result in the interpreter notifying the host of this call, so it may
    // handle it.

    let host = Host::from_function_names(["host_fn"]);
    let mut code = Code::default();
    let mut interpreter = Interpreter::default();

    compile("host_fn 1", &host, &mut code);

    let (id, input) = loop {
        if let InterpreterState::CallToHostFunction { id, input } =
            interpreter.step(&code)
        {
            break (id, input);
        }
    };

    assert_eq!(id, host.function_by_name("host_fn").unwrap().id);
    assert_eq!(input, 1.);
}

fn compile(input: &str, host: &Host, code: &mut Code) {
    let mut copy_of_code = code.clone();

    compiler::compile(input, host, code);

    // The tests pass the input code in a simple manner. But things should work
    // the same, if it's passed in multiple updates.
    for input in input.split_whitespace() {
        compiler::compile(input, host, &mut copy_of_code);
    }
    assert_eq!(*code, copy_of_code);
}
