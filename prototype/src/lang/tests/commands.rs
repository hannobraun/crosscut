use crate::lang::{self, editor::Command, host::Host};

#[test]
fn reset_interpreter_on_reset_command() {
    // If the reset command is given, the interpreter should begin again from
    // the start.

    let host = Host::empty();
    let mut lang = lang::Instance::new();

    lang.on_code("1", &host);
    lang.run_until_finished();
    assert!(lang.state().is_finished());

    lang.editor.on_command(
        Command::Reset,
        &mut lang.code,
        &mut lang.interpreter,
    );
    assert!(lang.state().is_running());

    let start = lang.code.root().fragment.body.ids().next().unwrap();
    assert_eq!(lang.interpreter.next(), Some(start));
}
