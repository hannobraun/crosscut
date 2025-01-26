use crosscut_runtime::{Effect, PopOperandError};

use crate::tests::infra::runtime;

#[test]
#[should_panic] // https://github.com/hannobraun/crosscut/issues/57
fn access_operand_from_parent_scope() {
    // Operands defined in a parent scope should be inaccessible.

    let effect = runtime()
        .update_code(
            r"
                main: {
                    \ ->
                        1 # local operand; not passed to `f`
                        f
                }

                f: {
                    \ ->
                        drop # no operand should be accessible
                }
            ",
        )
        .run_until_effect();

    assert_eq!(
        effect,
        Some(Effect::PopOperand {
            source: PopOperandError::MissingOperand
        })
    );
}
