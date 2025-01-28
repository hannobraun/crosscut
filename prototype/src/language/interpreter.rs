#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Value {
    None,
    Integer { value: i32 },
}
