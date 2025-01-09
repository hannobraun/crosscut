#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Hash {
    inner: blake3::Hash,
}
