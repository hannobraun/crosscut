use std::collections::BTreeMap;

use super::Fragment;

pub type Fragments = BTreeMap<Hash, Fragment>;

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable,
)]
pub struct Hash {
    value: [u8; 32],
}
impl Hash {
    pub fn of(fragment: &Fragment) -> Self {
        let value = udigest::hash::<blake3::Hasher>(fragment).into();
        Self { value }
    }
}
