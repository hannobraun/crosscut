use std::collections::BTreeSet;

use crosscut_runtime::InstructionAddress;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Breakpoints {
    durable: BTreeSet<InstructionAddress>,
    ephemeral: BTreeSet<InstructionAddress>,
}

impl Breakpoints {
    pub fn durable_at(&self, instruction: &InstructionAddress) -> bool {
        self.durable.contains(instruction)
    }

    pub fn set_durable(&mut self, instruction: InstructionAddress) {
        self.durable.insert(instruction);
    }

    pub fn clear_durable(&mut self, instruction: &InstructionAddress) -> bool {
        self.durable.remove(instruction)
    }

    pub fn set_ephemeral(&mut self, instruction: InstructionAddress) {
        self.ephemeral.insert(instruction);
    }

    pub fn clear_all_ephemeral(&mut self) {
        self.ephemeral.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = InstructionAddress> + '_ {
        self.durable.iter().chain(self.ephemeral.iter()).copied()
    }
}
