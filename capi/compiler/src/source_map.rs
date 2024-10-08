use std::collections::BTreeMap;

use capi_runtime::InstructionAddress;

use crate::fragments::{FragmentId, FragmentLocation, FunctionLocation};

#[derive(
    Clone, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize,
)]
pub struct SourceMap {
    instruction_to_fragment:
        BTreeMap<InstructionAddress, (FragmentId, FragmentLocation)>,
    fragment_to_instructions: BTreeMap<FragmentId, Vec<InstructionAddress>>,
    function_to_instruction_range:
        BTreeMap<FunctionLocation, [InstructionAddress; 2]>,
}

impl SourceMap {
    pub fn define_mapping(
        &mut self,
        instruction: InstructionAddress,
        fragment: (FragmentId, FragmentLocation),
    ) {
        self.instruction_to_fragment
            .insert(instruction, fragment.clone());
        let (fragment, _) = fragment;
        self.fragment_to_instructions
            .entry(fragment)
            .or_default()
            .push(instruction);
    }

    /// # Define which instructions map to the given function
    pub fn define_instruction_range(
        &mut self,
        location: FunctionLocation,
        range: [InstructionAddress; 2],
    ) {
        self.function_to_instruction_range.insert(location, range);
    }

    /// Get the ID of the fragment that the given instruction maps to
    ///
    /// Can return `None`, as there are a few compiler-generated instructions
    /// that call the `main` function.
    pub fn instruction_to_fragment(
        &self,
        instruction: &InstructionAddress,
    ) -> Option<(FragmentId, FragmentLocation)> {
        self.instruction_to_fragment.get(instruction).cloned()
    }

    /// Get the address of the instruction that the given fragment maps to
    ///
    /// Can return a reference to an empty `Vec`, as comments have no mapping to
    /// instructions.
    pub fn fragment_to_instructions(
        &self,
        fragment: &FragmentId,
    ) -> &Vec<InstructionAddress> {
        static EMPTY: Vec<InstructionAddress> = Vec::new();

        self.fragment_to_instructions
            .get(fragment)
            .unwrap_or(&EMPTY)
    }

    /// # Access the function from which this instruction was generated
    ///
    /// Can return `None`, as the instruction that call the `main` function were
    /// not themselves generated by a function.
    pub fn instruction_to_function(
        &self,
        instruction: &InstructionAddress,
    ) -> Option<&FunctionLocation> {
        self.function_to_instruction_range.iter().find_map(
            |(location, [min, max])| {
                if instruction.index >= min.index
                    && instruction.index <= max.index
                {
                    Some(location)
                } else {
                    None
                }
            },
        )
    }
}
