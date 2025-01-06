use crate::code::model::{Code, Expression};

pub fn update(code: &Code) {
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
