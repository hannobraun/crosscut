use crate::lang::{
    self,
    host::Host,
    interpreter::{InterpreterState, StepResult, Value},
};

#[test]
fn reset_interpreter_on_code_update_if_finished() {
    // If the interpreter is currently in a finished state, every update to the
    // code should reset it, so it starts again from the top.

    let host = Host::empty();
    let mut lang = lang::Instance::new();

    lang.on_char('1', &host);
    lang.run_until_finished();
    assert_eq!(lang.state(), InterpreterState::Finished);

    lang.on_char('2', &host);
    let initial_expression =
        lang.code.root().fragment.body.ids().next().unwrap();

    assert_eq!(lang.state(), InterpreterState::Running);
    assert_eq!(lang.interpreter.next(), Some(initial_expression));
}

#[test]
fn reset_interpreter_on_code_update_if_error() {
    // If the interpreter is currently in an error state, every update to the
    // code should reset it, so it starts again from the top.

    let host = Host::empty();
    let mut lang = lang::Instance::new();

    lang.on_code("identity", &host);
    let step = lang.interpreter.step(&lang.code);

    assert_eq!(step, StepResult::Error);
    assert_eq!(lang.state(), InterpreterState::Error);

    lang.on_char(' ', &host);
    lang.on_code("1", &host);
    let initial_expression =
        lang.code.root().fragment.body.ids().next().unwrap();

    assert_eq!(lang.state(), InterpreterState::Running);
    assert_eq!(lang.interpreter.next(), Some(initial_expression));
}

#[test]
fn update_interpreter_on_code_update() {
    // If code is changed through the editor, that is going to replace old
    // fragments with new ones. The interpreter, if it is currently in a running
    // state, is still going to point to those old fragments, however.
    //
    // It should be updated to point to whatever fragment has replaced the one
    // it currently points to.

    let host = Host::empty();
    let mut lang = lang::Instance::new();

    lang.on_code("identity 1", &host);
    let output = lang.run_until_finished();

    assert_eq!(output, Value::Integer { value: 1 });
}
