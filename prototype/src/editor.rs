use std::io::{self, stdout};

use crate::language::{
    code::{Code, Expression, Fragment, Token},
    interpreter::Interpreter,
};

#[derive(Default)]
pub struct Editor {
    pub code: Code,
}

impl Editor {
    pub fn update(
        code: &Code,
        interpreter: &Interpreter,
    ) -> anyhow::Result<()> {
        render_code(code, interpreter, stdout())?;
        Ok(())
    }
}

fn render_code(
    code: &Code,
    interpreter: &Interpreter,
    mut w: impl io::Write,
) -> anyhow::Result<()> {
    for fragment in &code.fragments {
        write!(w, "    ")?;

        match fragment {
            Fragment::Expression { expression } => match expression {
                Expression::LiteralValue { value } => {
                    writeln!(w, "{value}")?;
                }
            },
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
