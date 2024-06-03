use crate::InstructionAddress;

#[derive(
    Clone, Debug, Eq, PartialEq, Default, serde::Deserialize, serde::Serialize,
)]
pub struct CallStack {
    pub inner: Vec<InstructionAddress>,
}

impl CallStack {
    pub fn contains(&self, address: InstructionAddress) -> bool {
        self.inner.contains(&address.next())
    }

    pub fn push(&mut self, address: InstructionAddress) {
        self.inner.push(address);
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }
}
