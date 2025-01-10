use std::collections::BTreeMap;

use super::Fragment;

pub type Fragments = BTreeMap<Id, Fragment>;

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable,
)]
pub struct Id {
    hash: [u8; 32],
}
impl Id {
    pub fn of(fragment: &Fragment) -> Self {
        let hash = udigest::hash::<blake3::Hasher>(fragment).into();
        Self { hash }
    }
}
