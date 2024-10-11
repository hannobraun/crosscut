use std::collections::BTreeMap;

use capi_runtime::InstructionAddress;

use crate::{
    fragments::{FragmentLocation, Function, FunctionLocation},
    hash::Hash,
};

#[derive(
    Clone, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize,
)]
pub struct SourceMap {
    fragment_to_instructions:
        BTreeMap<FragmentLocation, Vec<InstructionAddress>>,
    instruction_to_fragment: BTreeMap<InstructionAddress, FragmentLocation>,
    function_to_instructions:
        BTreeMap<FunctionLocation, [InstructionAddress; 2]>,

    /// # Mapping of functions to the instructions that call them
    ///
    /// ## Implementation Note
    ///
    /// This data doesn't really fit here, as it's only ostensibly related to
    /// the purpose of the source map.
    ///
    /// The source map is sent to the debugger, and plays an essential role
    /// there to make it work. This data isn't used by the debugger, however.
    /// It's just used internally by the compiler, to update calls to changes
    /// functions.
    ///
    /// This should probably live as a field in `Compiler`.
    function_to_calling_instructions:
        BTreeMap<Hash<Function>, Vec<InstructionAddress>>,
}

impl SourceMap {
    /// # Define a mapping between a fragment and a number of instructions
    ///
    /// This function only accepts the location of the fragment. To append the
    /// associated instructions, use the returned [`Mapping`].
    pub fn map_fragment_to_instructions(
        &mut self,
        fragment: FragmentLocation,
    ) -> Mapping {
        // Make sure we don't have a previous mapping whose leftovers might
        // corrupt the new one.
        self.fragment_to_instructions.remove(&fragment);

        Mapping {
            fragment,
            source_map: self,
        }
    }

    /// # Define which instructions map to the given function
    pub fn map_function_to_instructions(
        &mut self,
        function: FunctionLocation,
        range: [InstructionAddress; 2],
    ) {
        self.function_to_instructions.insert(function, range);
    }

    /// # Map a function to the instructions that call it
    pub fn map_function_to_calling_instructions(
        &mut self,
        function: Hash<Function>,
        call: InstructionAddress,
    ) {
        self.function_to_calling_instructions
            .entry(function)
            .or_default()
            .push(call);
    }

    /// Get the ID of the fragment that the given instruction maps to
    ///
    /// Can return `None`, as there are a few compiler-generated instructions
    /// that call the `main` function.
    pub fn instruction_to_fragment(
        &self,
        instruction: &InstructionAddress,
    ) -> Option<&FragmentLocation> {
        self.instruction_to_fragment.get(instruction)
    }

    /// Get the address of the instruction that the given fragment maps to
    ///
    /// Can return a reference to an empty `Vec`, as comments have no mapping to
    /// instructions.
    pub fn fragment_to_instructions(
        &self,
        fragment: &FragmentLocation,
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
        self.function_to_instructions.iter().find_map(
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

    /// # Consume all instructions that call the provided function
    ///
    /// Completely removes the entry for that function. This is done to prevent
    /// the source map from accumulating old calls that should no longer be
    /// relevant, and might possibly cause problems in future updates.
    pub fn consume_calls_to_function(
        &mut self,
        function: &Hash<Function>,
    ) -> Vec<InstructionAddress> {
        self.function_to_calling_instructions
            .remove(function)
            .unwrap_or_default()
    }
}

/// # A mapping of a fragment to a number of instructions
///
/// Returned by [`SourceMap::define_mapping`].
pub struct Mapping<'r> {
    fragment: FragmentLocation,
    source_map: &'r mut SourceMap,
}

impl Mapping<'_> {
    pub fn append_instruction(&mut self, instruction: InstructionAddress) {
        self.source_map
            .fragment_to_instructions
            .entry(self.fragment.clone())
            .or_default()
            .push(instruction);
        self.source_map
            .instruction_to_fragment
            .insert(instruction, self.fragment.clone());
    }
}
