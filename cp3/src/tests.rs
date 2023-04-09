use crate::cp;

pub struct TestReport {
    pub module: String,
    pub name: String,
    pub result: Result<(), Error>,
}

pub fn run() -> anyhow::Result<Vec<TestReport>> {
    let code = r#"
        mod bool {
            test "true" { true }
            test "false not" { false not }
            test "and - true true" { true true and }
            test "and - true false" { true false and not }
            test "and - false true" { false true and not }
            test "and - false false" { false false and not }
        }

        mod basics {
            test "drop" { true false drop }
            test "clone" { true clone drop }
            test "binding" { true false => t f . t }
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
    "#;

    let mut functions = cp::Functions::new();
    let data_stack = cp::execute(code.chars(), &mut functions)?;
    if !data_stack.is_empty() {
        anyhow::bail!("Importing tests left values on stack: {data_stack:?}")
    }

    let mut results = Vec::new();

    for (name, function) in functions.tests() {
        let mut call_stack = cp::CallStack;
        let mut data_stack = cp::DataStack::new();

        let result = cp::evaluate(
            function.body,
            &functions,
            &mut call_stack,
            &mut data_stack,
        )
        .map_err(Error::Evaluator)
        .and_then(|()| {
            if data_stack.pop_bool()? {
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
            module: function.module,
            name,
            result,
        });
    }

    results.sort_by_key(|report| report.result.is_ok());
    results.reverse();

    Ok(results)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Evaluator(cp::EvaluatorError),

    #[error(transparent)]
    ReturnValue(#[from] cp::DataStackError),

    #[error("Test did not return `true`")]
    TestFailed,

    #[error("Test returned too many values")]
    TestReturnedTooMuch,
}
