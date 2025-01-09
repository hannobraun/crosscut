#[derive(Clone, Debug, PartialEq)]
pub struct Hash {
    inner: blake3::Hash,
}
