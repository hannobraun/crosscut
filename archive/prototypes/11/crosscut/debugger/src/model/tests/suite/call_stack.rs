use crosscut_runtime::Effect;

use crate::model::{
    tests::infra::{
        debugger, ActiveFunctionsEntriesExt, ActiveFunctionsExt,
        DebugBranchExt, DebugExpressionExt, DebugFunctionExt, FunctionsExt,
    },
    ActiveFunctionsEntry, UserAction,
};

#[test]
fn basic_call_stack() {
    // All functions in the call stack should show up in the active functions
    // view, if the process is stopped. This test constructs a scenario that
    // requires no special handling to detect and fix the effects of tail call
    // elimination, to provide a baseline.
    //
    // This test expects all defined functions to be active functions. The order
    // of functions is inner to outer, as it's most useful to the developer to
    // display the instruction where we're currently paused up top.

    let transient = debugger()
        .provide_source_code(
            r"
                main: fn
                    br size_x, size_y ->
                        f
                        nop # make sure the previous call is not a tail call
                    end
                end
                f: fn
                    br ->
                        g
                        nop # make sure the previous call is not a tail call
                    end
                end
                g: fn
                    br ->
                        brk
                    end
                end
            ",
        )
        .run_program()
        .transient_state();

    let names = transient.active_functions.names();
    assert_eq!(names, vec!["g", "f", "main"]);
}

#[test]
fn stopped_at_host_function() {
    // If execution is stopped at a host function, it should be displayed as
    // such.

    let mut debugger = debugger();
    debugger
        .provide_source_code(
            r"
                main: fn
                    br size_x, size_y ->
                        halt
                    end
                end
            ",
        )
        .run_program();

    debugger
        .transient_state()
        .active_functions
        .expect_entries()
        .expect_functions()
        .with_name("main")
        .active_expression()
        .expect_call_to_host_function(
            "halt",
            &debugger
                .persistent_state()
                .code
                .inner
                .as_ref()
                .unwrap()
                .function_calls,
        );
}

#[test]
fn stopped_in_anonymous_function() {
    // If execution is stopped within an anonymous function, the function that
    // contains that block should appear as an active function, and the current
    // instruction should be visible.

    let mut debugger = debugger();
    debugger
        .provide_source_code(
            r"
                main: fn
                    br size_x, size_y ->
                        fn
                            br ->
                                brk
                            end
                        end
                        eval
                    end
                end
            ",
        )
        .run_program();

    let expression = debugger
        .transient_state()
        .active_functions
        .expect_entries()
        .expect_functions()
        .with_name("main")
        .only_branch()
        .expression(0)
        .expect_function()
        .only_branch()
        .expression(0);
    assert_eq!(expression.data.effect, Some(Effect::Breakpoint));

    expression.expect_call_to_intrinsic(
        "brk",
        &debugger
            .persistent_state()
            .code
            .inner
            .as_ref()
            .unwrap()
            .function_calls,
    );
}

#[test]
fn call_stack_reconstruction_missing_main() {
    // Tail call elimination can leave gaps in the call stack. If the `main`
    // function is missing due to that, it should be reconstructed.

    let mut debugger = debugger();
    debugger
        .provide_source_code(
            r"
                main: fn
                    br size_x, size_y ->
                        f
                    end
                end

                f: fn
                    br ->
                        brk
                    end
                end
            ",
        )
        .run_program();

    let transient = debugger.transient_state();

    let names = transient.active_functions.names();
    assert_eq!(names, vec!["f", "main"]);

    transient
        .active_functions
        .expect_entries()
        .expect_functions()
        .with_name("main")
        .active_expression()
        .expect_call_to_function(
            "f",
            &debugger
                .persistent_state()
                .code
                .inner
                .as_ref()
                .unwrap()
                .syntax_tree,
            &debugger
                .persistent_state()
                .code
                .inner
                .as_ref()
                .unwrap()
                .function_calls,
        );
}

#[test]
fn call_stack_reconstruction_missing_single_branch_function() {
    // Tail call elimination can leave gaps in the call stack. If the missing
    // functions have only a single branch each, it is possible to add them back
    // without any additional hints being required.

    let mut debugger = debugger();
    debugger
        .provide_source_code(
            r"
                main: fn
                    br size_x, size_y ->
                        f
                        nop # make sure the previous call is not a tail call
                    end
                end

                f: fn
                    br ->
                        g
                    end
                end

                g: fn
                    br ->
                        brk
                    end
                end
            ",
        )
        .run_program();

    let transient = debugger.transient_state();

    let names = transient.active_functions.names();
    assert_eq!(names, vec!["g", "f", "main"]);

    transient
        .active_functions
        .expect_entries()
        .expect_functions()
        .with_name("f")
        .active_expression()
        .expect_call_to_function(
            "g",
            &debugger
                .persistent_state()
                .code
                .inner
                .as_ref()
                .unwrap()
                .syntax_tree,
            &debugger
                .persistent_state()
                .code
                .inner
                .as_ref()
                .unwrap()
                .function_calls,
        );
}

#[test]
fn display_gap_where_missing_function_is_called_from_multi_branch_function() {
    // Tail call elimination can leave gaps in the call stack. Some simpler
    // cases are already getting reconstructed, but right now, we're not doing
    // that yet for functions with multiple branches.

    let transient = debugger()
        .provide_source_code(
            r"
                main: fn
                    br 0, 0 ->
                        f
                    end

                    br size_x, size_y ->
                        f
                    end
                end

                f: fn
                    br ->
                        g
                    end
                end

                g: fn
                    br ->
                        brk
                    end
                end
            ",
        )
        .run_program()
        .transient_state();

    let entries = transient.active_functions.expect_entries();
    assert!(matches!(
        dbg!(entries.as_slice()),
        &[
            ActiveFunctionsEntry::Function(_),
            ActiveFunctionsEntry::Gap,
            ActiveFunctionsEntry::Function(_),
        ]
    ));
}

#[test]
#[should_panic] // https://github.com/hannobraun/crosscut/issues/47
fn display_gap_where_missing_fn_is_called_from_reconstructed_multi_branch_fn() {
    // Tail call elimination can leave gaps in the call stack. Some simpler
    // cases are already getting reconstructed, but right now, we're not doing
    // that yet for anonymous functions with multiple branches.

    let transient = debugger()
        .provide_source_code(
            r"
                main: fn
                    br size_x, size_y ->
                        0 f
                    end
                end

                f: fn
                    br 0 ->
                        g
                    end

                    br n ->
                        g
                    end
                end

                g: fn
                    br ->
                        h
                    end
                end

                h: fn
                    br ->
                        brk
                    end
                end
            ",
        )
        .run_program()
        .transient_state();

    let entries = transient.active_functions.expect_entries();
    assert!(matches!(
        dbg!(entries.as_slice()),
        &[
            ActiveFunctionsEntry::Function(_),
            ActiveFunctionsEntry::Function(_),
            ActiveFunctionsEntry::Gap,
            ActiveFunctionsEntry::Function(_),
        ]
    ));
}

#[test]
fn instruction_on_call_stack_with_no_associated_expression() {
    // If a host function has just been executed, it is possible that the
    // currently active instruction is a return instruction, if that is located
    // right after the call to the host function.
    //
    // Since a return is not associated with a expression, the debugger must not
    // rely on that being the case. Everything should work normally in this
    // case.
    //
    // Note that as of this writing, there is an explicit test about host
    // functions on the stack. But that is stopped _at_ the host function, not
    // right after it. So it doesn't test the same thing.

    let mut debugger = debugger();
    debugger
        .provide_source_code(
            r"
                main: fn
                    br size_x, size_y ->
                        submit
                        main
                    end
                end

                submit: fn
                    br ->
                        submit_frame # this is the call to the host function
                    end
                end
            ",
        )
        .run_program();

    // This will stop the program after the `submit_frame`, since that is the
    // only time when the game engine reacts to commands.
    debugger.on_user_action(UserAction::Stop).unwrap();

    let _ = debugger.transient_state();

    // Nothing else to do. We're not currently handling this case well[1], so
    // unless we panicked, everything's good, as far as this test is concerned.
    //
    // [1] https://github.com/hannobraun/crosscut/issues/53
}
