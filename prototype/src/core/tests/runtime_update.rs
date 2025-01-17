use crate::core::{self, host::Host, interpreter::InterpreterState};

#[test]
fn reset_interpreter_on_code_update_if_finished() {
    // If the interpreter is currently in a finished state, every update to the
    // code should reset it, so it starts again from the top.

    let host = Host::empty();
    let mut core = core::Instance::new();

    assert_eq!(
        core.interpreter.state(&core.code),
        InterpreterState::Finished,
    );

    core.edit("1", &host);
    let initial_expression = core
        .code
        .fragments()
        .get(&core.code.root)
        .body
        .ids()
        .next()
        .unwrap();

    assert_eq!(
        core.interpreter.state(&core.code),
        InterpreterState::Running,
    );
    assert_eq!(core.interpreter.next(), Some(initial_expression));
}
