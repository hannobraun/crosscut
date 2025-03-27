#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Literal {
    #[allow(unused)] // temporary oddity; work to resolve it is ongoing
    Function,
    Integer {
        value: i32,
    },
    Tuple,
}
