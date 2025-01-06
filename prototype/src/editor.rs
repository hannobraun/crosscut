use std::io::{self, stdout};

use crate::{
    code::model::{Code, Expression},
    interpreter::Interpreter,
};

pub fn update(code: &Code, interpreter: &Interpreter) -> anyhow::Result<()> {
    render_code(code, interpreter, stdout())?;
    Ok(())
}

fn render_code(
    code: &Code,
    interpreter: &Interpreter,
    mut w: impl io::Write,
) -> anyhow::Result<()> {
    for expression in &code.expressions {
        write!(w, "    ")?;

        match expression {
            Expression::LiteralNumber { value } => {
                writeln!(w, "{value}")?;
            }
            Expression::InvalidNumber { invalid } => {
                writeln!(w, "invalid number `{invalid}`")?;
            }
        }
    }

    let state = interpreter.state(code);

    writeln!(w)?;
    write!(w, "{} > ", state)?;

    w.flush()?;

    Ok(())
}
