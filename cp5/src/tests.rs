use crate::{
    cp,
    test_report::{Error, TestReport},
};

pub fn define(functions: &mut cp::Functions) -> anyhow::Result<cp::Functions> {
    let code = r#"
        mod bool {
            test "true" { true }
            test "false not" { false not }
            test "and - true true" { true true and }
            test "and - true false" { true false and not }
            test "and - false true" { false true and not }
            test "and - false false" { false false and not }
        }

        mod binding {
            test "binding" { true false => t f . t }
            test "tokenization" { true=>t.t }
        }

        mod basics {
            test "drop" { true false drop }
            test "clone" { true clone drop }
        }

        mod block {
            test "eval" { { true } eval }
            test "lazy evaluation" { true { drop } drop }
            test "tokenization" { {true}eval{true}eval and }
        }

        mod array {
            test "unwrap" { [ true ] unwrap }
            test "eager evaluation" { true false [ drop ] drop }
            test "tokenization" { [true]unwrap[true]unwrap and }
        }

        mod fn_ {
            test "fn" { fn f { true } f }
        }

        mod if_ {
            test "then" { true { true } { false } if }
            test "else" { false { false } { true } if }
        }

        mod string {
            test "=" { "a" "a" = }
            test "= not" { "a" "b" = not }
            test "tokenization" { "a""a"="b""b"= and }
        }

        mod std {
            test "times" { { true } 2 times drop }
        }
    "#;

    let mut data_stack = cp::DataStack::new();
    let mut bindings = cp::Bindings::new();
    let mut tests = cp::Functions::new();

    cp::execute(code, &mut data_stack, &mut bindings, functions, &mut tests)?;

    if !data_stack.is_empty() {
        anyhow::bail!("Importing tests left values on stack: {data_stack:?}")
    }

    Ok(tests)
}

pub fn run(
    functions: &cp::Functions,
    tests: &cp::Functions,
) -> anyhow::Result<Vec<TestReport>> {
    let mut results = Vec::new();

    for (name, function) in tests {
        let expressions = cp::StageInput::from(function.body.clone());

        let mut data_stack = cp::DataStack::new();
        let mut bindings = cp::Bindings::new();
        let mut tests = cp::Functions::new();

        let result = cp::evaluate_all(
            expressions,
            &mut data_stack,
            &mut bindings,
            functions,
            &mut tests,
        );

        let result = result
            .map_err(Error::Evaluator)
            .and_then(|()| {
                let test_passed = data_stack.pop_bool()?;
                if test_passed {
                    Ok(())
                } else {
                    Err(Error::TestFailed)
                }
            })
            .and_then(|()| {
                if data_stack.is_empty() {
                    Ok(())
                } else {
                    Err(Error::TestReturnedTooMuch)
                }
            });

        results.push(TestReport {
            module: function.module.clone(),
            name: name.clone(),
            result,
        })
    }

    Ok(results)
}
