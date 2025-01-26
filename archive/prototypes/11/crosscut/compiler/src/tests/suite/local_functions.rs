use crate::tests::infra::runtime;

#[test]
fn eval() {
    runtime()
        .update_code(
            r"
                main: fn
                    br ->
                        fn
                            br ->
                                0 send
                            end
                        end
                        eval
                    end
                end
            ",
        )
        .run_until_receiving(0);
}

#[test]
fn parameter() {
    runtime()
        .update_code(
            r"
                main: fn
                    br ->
                        0
                        fn
                            br channel ->
                                channel
                            end
                        end
                        eval
                        send
                    end
                end
            ",
        )
        .run_until_receiving(0);
}

#[test]
fn parameter_shadowing() {
    runtime()
        .update_code(
            r"
                main: fn
                    br ->
                        0
                        fn
                            br channel ->
                                channel
                                fn
                                    br channel ->
                                        channel
                                    end
                                end
                                eval
                            end
                        end
                        eval
                        send
                    end
                end
            ",
        )
        .run_until_receiving(0);
}

#[test]
fn captured_binding() {
    runtime()
        .update_code(
            r"
                main: fn
                    br ->
                        0
                        fn
                            br channel ->
                                fn
                                    br ->
                                        # We are not using `channel` here, to
                                        # make sure that capturing works even
                                        # from a grandparent scope.

                                        fn
                                            br ->
                                                channel send
                                            end
                                        end
                                        eval
                                    end
                                end
                                eval
                            end
                        end
                        eval
                    end
                end
            ",
        )
        .run_until_receiving(0);
}
