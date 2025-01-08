use pretty_assertions::assert_eq;

use crate::language::interpreter::InterpreterState;

use super::{code::Code, compiler, interpreter::Interpreter};

#[test]
fn evaluate_single_expression() {
    // If the program consists only of a single expression, it should be
    // evaluated, and its value returned by the interpreter.

    let mut code = Code::default();
    let mut interpreter = Interpreter::default();

    compile("1", &mut code);

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

    let mut code = Code::default();
    let mut interpreter = Interpreter::default();

    compile("1 2", &mut code);

    assert_eq!(
        interpreter.step(&code),
        InterpreterState::Finished { output: 1. },
    );
    assert_eq!(interpreter.step(&code), InterpreterState::Error);
}

fn compile(input: &str, code: &mut Code) {
    let mut copy_of_code = code.clone();

    compiler::compile(input, code);

    // The tests pass the input code in a simple manner. But things should work
    // the same, if it's passed in multiple updates.
    for input in input.split_whitespace() {
        compiler::compile(input, &mut copy_of_code);
    }
    assert_eq!(*code, copy_of_code);
}
