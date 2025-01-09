#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Hash {
    inner: [u8; 32],
}
