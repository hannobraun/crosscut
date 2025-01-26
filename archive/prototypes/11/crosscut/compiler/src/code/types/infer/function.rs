use crate::code::{Index, Signature};

use super::{
    signature::{unify_type_list, IndirectSignature},
    types::{InferredType, InferredTypes},
};

#[derive(Clone, Debug)]
pub struct InferredFunction {
    /// # The inferred inputs of the function
    ///
    /// Since function parameters are defined as part of each of its branches,
    /// we always know at least the number of parameters. Even if we don't know
    /// their types.
    pub inputs: Vec<Index<InferredType>>,

    /// # The inferred outputs of the function
    ///
    /// Can be `None`, if inference fails partway through a function, as we
    /// might not know anything about the output in this case.
    pub outputs: Option<Vec<Index<InferredType>>>,
}

impl InferredFunction {
    pub fn unify_with(&mut self, other: &mut Self, types: &mut InferredTypes) {
        unify_type_list([&self.inputs, &other.inputs], types);

        match (&self.outputs, &other.outputs) {
            (Some(self_outputs), Some(other_outputs)) => {
                unify_type_list([self_outputs, other_outputs], types);
            }
            (Some(_), None) => {
                other.outputs = self.outputs.clone();
            }
            (None, Some(_)) => {
                self.outputs = other.outputs.clone();
            }
            (None, None) => {}
        }
    }

    pub fn to_signature(&self) -> Option<IndirectSignature> {
        let inputs = self.inputs.clone();
        let outputs = self.outputs.clone()?;

        Some(Signature { inputs, outputs })
    }
}

#[cfg(test)]
mod tests {
    use crate::code::{
        types::infer::types::{InferredType, InferredTypes},
        Signature, Type,
    };

    use super::InferredFunction;

    #[test]
    fn unify_known_and_unknown_types() {
        let mut types = InferredTypes::default();

        let a = InferredFunction {
            inputs: vec![
                types.push(InferredType::Direct(Type::Number)),
                types.push(InferredType::Unknown),
            ],
            outputs: Some(vec![
                types.push(InferredType::Direct(Type::Number)),
                types.push(InferredType::Unknown),
            ]),
        };
        let b = InferredFunction {
            inputs: vec![
                types.push(InferredType::Unknown),
                types.push(InferredType::Direct(Type::Number)),
            ],
            outputs: Some(vec![
                types.push(InferredType::Unknown),
                types.push(InferredType::Direct(Type::Number)),
            ]),
        };

        for [mut a, mut b] in [[a.clone(), b.clone()], [b, a]] {
            // This is sketchy. The first permutation is going to update
            // `types`, which will then influence the second permutation.
            a.unify_with(&mut b, &mut types);

            for function in [a, b] {
                let signature = function.to_signature().unwrap();
                let signature =
                    signature.to_direct(&mut types).unwrap().unwrap();

                assert_eq!(
                    signature,
                    Signature {
                        inputs: vec![Type::Number, Type::Number],
                        outputs: vec![Type::Number, Type::Number]
                    },
                )
            }
        }
    }

    #[test]
    fn unify_available_and_unavailable_outputs() {
        let mut types = InferredTypes::default();

        let a = InferredFunction {
            inputs: vec![],
            outputs: Some(vec![types.push(InferredType::Direct(Type::Number))]),
        };
        let b = InferredFunction {
            inputs: vec![],
            outputs: None,
        };

        for [mut a, mut b] in [[a.clone(), b.clone()], [b, a]] {
            // This is sketchy. See comment on the equivalent code in the test
            // above.
            a.unify_with(&mut b, &mut types);

            for function in [a, b] {
                let signature = function.to_signature().unwrap();
                let signature =
                    signature.to_direct(&mut types).unwrap().unwrap();

                assert_eq!(
                    signature,
                    Signature {
                        inputs: vec![],
                        outputs: vec![Type::Number]
                    },
                )
            }
        }
    }
}
