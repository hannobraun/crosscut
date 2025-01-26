use std::collections::BTreeMap;

use crate::code::{
    syntax::{FunctionLocation, MemberLocation, ParameterLocation},
    Index, Signature, Type,
};

use super::{
    function::InferredFunction,
    signature::IndirectSignature,
    types::{InferredType, InferredTypes},
};

#[derive(Default)]
pub struct InferenceContext {
    pub types: InferredTypes,
    pub functions: BTreeMap<FunctionLocation, InferredFunction>,
    pub bindings: BTreeMap<ParameterLocation, Index<InferredType>>,
    pub expressions: BTreeMap<MemberLocation, IndirectSignature>,
}

impl InferenceContext {
    pub fn function(
        &mut self,
        location: &FunctionLocation,
        functions: &BTreeMap<FunctionLocation, Signature>,
    ) -> Option<IndirectSignature> {
        functions
            .get(location)
            .map(|signature| {
                IndirectSignature::from_direct(
                    signature.clone(),
                    &mut self.types,
                )
            })
            .or_else(|| {
                let function = self.functions.get(location).cloned()?;
                function.to_signature()
            })
    }

    pub fn binding(
        &mut self,
        location: &ParameterLocation,
        parameters: &BTreeMap<ParameterLocation, Type>,
    ) -> Option<Index<InferredType>> {
        parameters
            .get(location)
            .map(|type_| self.types.push(InferredType::Direct(type_.clone())))
            .or_else(|| self.bindings.get(location).cloned())
    }
}
