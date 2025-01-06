use crate::code::model::Code;

pub fn update(code: &Code) {
    for expression in &code.expressions {
        println!("{expression}");
    }
}
