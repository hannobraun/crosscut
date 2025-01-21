use crate::lang::{self, editor::InputEvent, host::Host, interpreter::Value};

#[test]
fn reset_interpreter_on_reset_command() {
    // If the reset command is given, the interpreter should begin again from
    // the start.

    let host = Host::empty();
    let mut lang = lang::Instance::new();

    lang.on_code("1", &host);
    lang.run_until_finished();
    assert!(lang.state().is_finished());

    lang.on_event(InputEvent::Enter, &host); // enter command mode
    lang.on_input("reset", &host);
    lang.on_event(InputEvent::Enter, &host);
    assert!(lang.state().is_running());

    let start = lang.code.root().fragment.body.ids().next().unwrap();
    assert_eq!(lang.interpreter.next(), Some(start));
}

#[test]
fn return_to_edit_mode_after_command_execution() {
    // After a command has been executed, the editor should return to edit mode.

    let host = Host::empty();
    let mut lang = lang::Instance::new();

    // Start editing the code.
    lang.on_code("1", &host);

    // Execute a command in between editing.
    lang.on_event(InputEvent::Enter, &host); // enter command mode
    lang.on_input("nop", &host);
    lang.on_event(InputEvent::Enter, &host);

    // Continue editing.
    lang.on_code("2", &host);

    assert_eq!(lang.run_until_finished(), Value::Integer { value: 12 });
}
