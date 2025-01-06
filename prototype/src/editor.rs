use crate::{
    code::model::{Code, Expression},
    interpreter::Interpreter,
};

pub fn update(code: &Code, _: &Interpreter) {
    render_code(code);
}

fn render_code(code: &Code) {
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
}
