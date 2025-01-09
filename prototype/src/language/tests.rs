use pretty_assertions::assert_eq;

use crate::language::{
    compiler::tests::infra::compile, host::Host, interpreter::InterpreterState,
};

use super::{code::Code, interpreter::Interpreter};

#[test]
fn evaluate_single_expression() {
    // If the program consists only of a single expression, it should be
    // evaluated, and its value returned by the interpreter.

    let host = Host::empty();
    let mut code = Code::default();
    let mut interpreter = Interpreter::new(code.entry());

    compile("1", &host, &mut code);

    assert_eq!(
        interpreter.step(&code),
        InterpreterState::Finished { output: 1 },
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
    let mut interpreter = Interpreter::new(code.entry());

    compile("1 2", &host, &mut code);

    assert_eq!(
        interpreter.step(&code),
        InterpreterState::Finished { output: 1 },
    );
    assert_eq!(interpreter.step(&code), InterpreterState::Error);
}
