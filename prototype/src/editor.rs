use std::io::{self, stdout};

use crate::language::{
    code::{Code, Fragment, Token},
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
    for fragment in &code.fragments {
        write!(w, "    ")?;

        match fragment {
            Fragment::UnexpectedToken { token } => match token {
                Token::Identifier { name } => {
                    writeln!(w, "{name}")?;
                }
                Token::LiteralNumber { value } => {
                    writeln!(w, "{value}")?;
                }
            },
        }
    }

    writeln!(w)?;
    write!(w, "{} > ", interpreter.state(code))?;

    w.flush()?;

    Ok(())
}
