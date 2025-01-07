use std::io::{self, stdout};

use crate::{
    code::{Code, Expression},
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
            Expression::Identifier { name: invalid } => {
                writeln!(w, "invalid number `{invalid}`")?;
            }
            Expression::LiteralNumber { value } => {
                writeln!(w, "{value}")?;
            }
        }
    }

    writeln!(w)?;
    write!(w, "{} > ", interpreter.state(code))?;

    w.flush()?;

    Ok(())
}
