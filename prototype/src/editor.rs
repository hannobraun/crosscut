use crate::{
    code::model::{Code, Expression},
    interpreter::Interpreter,
};

pub fn update(code: &Code, _: &Interpreter) -> anyhow::Result<()> {
    render_code(code)?;
    Ok(())
}

fn render_code(code: &Code) -> anyhow::Result<()> {
    for expression in &code.expressions {
        print!("    ");

        match expression {
            Expression::LiteralNumber { value } => {
                println!("{value}");
            }
            Expression::InvalidNumber { invalid } => {
                println!("invalid number `{invalid}`");
            }
        }
    }

    Ok(())
}
