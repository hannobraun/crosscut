use crate::language::{
    self,
    editor::EditorInputEvent,
    host::Host,
    interpreter::{StepResult, Value},
};

#[test]
fn update_on_each_character() {
    // The editor should compile the code and update the interpreter on every
    // character it receives.

    let host = Host::empty();
    let mut lang = language::Language::new();

    lang.on_char('1', &host);
    assert_eq!(
        lang.interpreter.step(&lang.code),
        StepResult::Finished {
            output: Value::Integer { value: 1 }
        },
    );

    lang.on_char('2', &host);
    assert_eq!(
        lang.interpreter.step(&lang.code),
        StepResult::Finished {
            output: Value::Integer { value: 12 }
        },
    );
}

#[test]
fn update_on_backspace() {
    // When deleting code while in edit mode, the editor should compile the code
    // and update the interpreter immediately.

    let host = Host::empty();
    let mut lang = language::Language::new();

    lang.on_char('1', &host);
    lang.on_char('2', &host);
    assert_eq!(
        lang.interpreter.step(&lang.code),
        StepResult::Finished {
            output: Value::Integer { value: 12 }
        },
    );

    lang.on_event(EditorInputEvent::RemoveCharacterLeft, &host);
    assert_eq!(
        lang.interpreter.step(&lang.code),
        StepResult::Finished {
            output: Value::Integer { value: 1 }
        },
    );
}
