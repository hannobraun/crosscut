use crosscut_runtime::{Instruction, InstructionAddress};

/// # Compiled instructions for the runtime to execute
#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Instructions {
    inner: Vec<(InstructionAddress, Instruction)>,
}

impl Instructions {
    pub fn push(&mut self, instruction: Instruction) -> InstructionAddress {
        let address = InstructionAddress {
            index: self.inner.len().try_into().unwrap(),
        };
        self.inner.push((address, instruction));
        address
    }

    pub fn get(&self, address: &InstructionAddress) -> Option<&Instruction> {
        let (stored_address, instruction) =
            self.inner.get(address.to_usize())?;
        assert_eq!(address, stored_address);
        Some(instruction)
    }

    pub fn replace(
        &mut self,
        address: &InstructionAddress,
        instruction: Instruction,
    ) {
        let (stored_address, stored_instruction) =
            self.inner.get_mut(address.to_usize()).unwrap();
        assert_eq!(address, stored_address);
        *stored_instruction = instruction;
    }

    pub fn to_runtime_instructions(&self) -> crosscut_runtime::Instructions {
        crosscut_runtime::Instructions { inner: &self.inner }
    }
}
