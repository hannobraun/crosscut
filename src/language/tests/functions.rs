use crate::language::{code::NodePath, instance::Language, runtime::Value};

#[test]
fn define_and_evaluate_function() {
    // It is possible to define a function using a function literal, return that
    // function from the program, then tell the language to evaluate it.

    let mut language = Language::without_package();

    language.enter_code("127 fn");
    let hash = match language.step_until_finished() {
        Ok(Value::Function { hash }) => hash,
        output => {
            panic!("Unexpected output: {output:?}");
        }
    };

    // Just conjuring a `NodePath` out of thin air like this, only works because
    // it is essentially equivalent to the `NodeHash` we already have. This
    // won't be the case for much longer.
    //
    // And while we can't get a `NodePath` from `Value::Function` (this would
    // create a circular dependency in how the hashes need to be computed), the
    // good new is that `Evaluator` has all the info we need. It just doesn't
    // return it when stepping.
    //
    // If it did, and all the intermediate layers passed on this information, we
    // have all we need to construct the `NodePath` for the function body here,
    // now and going forward.
    language.evaluate(NodePath { hash });
    assert_eq!(
        language.step_until_finished(),
        Ok(Value::Integer { value: 127 }),
    );
}
