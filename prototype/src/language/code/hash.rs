#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Hash {
    inner: blake3::Hash,
}

impl Ord for Hash {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.inner.as_bytes().cmp(other.inner.as_bytes())
    }
}

impl PartialOrd for Hash {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
