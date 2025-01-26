use crate::code::{Index, Signature, Type};

use super::types::{InferredType, InferredTypes, Result};

pub type IndirectSignature = Signature<Index<InferredType>>;

impl IndirectSignature {
    pub fn from_direct(
        signature: Signature,
        types: &mut InferredTypes,
    ) -> Self {
        let mut map = |from: Vec<Type>| {
            from.into_iter()
                .map(|type_| types.push(InferredType::Direct(type_)))
                .collect()
        };

        Signature {
            inputs: map(signature.inputs),
            outputs: map(signature.outputs),
        }
    }

    pub fn to_direct(
        &self,
        types: &mut InferredTypes,
    ) -> Result<Option<Signature<Type>>> {
        let mut try_map = |from: &Vec<Index<InferredType>>| {
            from.iter()
                .map(|index| {
                    let type_ = types.resolve(index)?;
                    type_.into_type(types)
                })
                .collect::<Result<Option<_>>>()
        };

        let inputs = try_map(&self.inputs)?;
        let outputs = try_map(&self.outputs)?;

        let signature = inputs
            .zip(outputs)
            .map(|(inputs, outputs)| Signature { inputs, outputs });

        Ok(signature)
    }
}

pub fn unify([a, b]: [&IndirectSignature; 2], types: &mut InferredTypes) {
    unify_type_list([&a.inputs, &b.inputs], types);
    unify_type_list([&a.outputs, &b.outputs], types);
}

pub fn unify_type_list(
    [a, b]: [&Vec<Index<InferredType>>; 2],
    types: &mut InferredTypes,
) {
    assert_eq!(
        a.len(),
        b.len(),
        "Expecting type lists to have the same length.",
    );

    for (a, b) in a.iter().zip(b.iter()) {
        types.unify([a, b]);
    }
}
