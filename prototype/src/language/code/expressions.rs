use super::IntrinsicFunction;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expression {
    HostFunction { id: u32 },
    IntrinsicFunction { function: IntrinsicFunction },
}
