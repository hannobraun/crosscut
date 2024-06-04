use crate::InstructionAddress;

#[derive(
    Clone, Debug, Eq, PartialEq, Default, serde::Deserialize, serde::Serialize,
)]
pub struct CallStack {
    inner: Vec<InstructionAddress>,
}

impl CallStack {
    pub fn new(next: InstructionAddress) -> Self {
        Self { inner: vec![next] }
    }

    pub fn next(&self) -> Option<InstructionAddress> {
        self.inner.last().copied()
    }

    pub fn contains(&self, address: InstructionAddress) -> bool {
        self.inner.contains(&address.next())
    }

    pub fn advance(&mut self) {
        let instruction = self.inner.last_mut().unwrap();
        instruction.increment();
    }

    pub fn push(
        &mut self,
        address: InstructionAddress,
    ) -> Result<(), CallStackOverflow> {
        self.inner.push(address);
        Ok(())
    }

    pub fn pop(&mut self) -> Option<InstructionAddress> {
        self.inner.pop()
    }

    pub fn iter(&self) -> impl Iterator<Item = &InstructionAddress> {
        self.inner.iter()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct CallStackOverflow;
