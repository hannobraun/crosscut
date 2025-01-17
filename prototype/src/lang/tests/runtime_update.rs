use crate::lang::{self, host::Host, interpreter::InterpreterState};

#[test]
fn reset_interpreter_on_code_update_if_finished() {
    // If the interpreter is currently in a finished state, every update to the
    // code should reset it, so it starts again from the top.

    let host = Host::empty();
    let mut lang = lang::Instance::new();

    assert_eq!(
        lang.interpreter.state(&lang.code),
        InterpreterState::Finished,
    );

    lang.edit("1", &host);
    let initial_expression = lang
        .code
        .fragments()
        .get(&lang.code.root)
        .body
        .ids()
        .next()
        .unwrap();

    assert_eq!(
        lang.interpreter.state(&lang.code),
        InterpreterState::Running,
    );
    assert_eq!(lang.interpreter.next(), Some(initial_expression));
}
