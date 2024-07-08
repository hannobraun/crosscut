use std::collections::{BTreeMap, BTreeSet};

use crate::repr::{
    fragments::{
        Fragment, FragmentAddress, FragmentPayload, Fragments, Function,
        FunctionFragments,
    },
    syntax::{Expression, Script},
};

pub fn script_to_fragments(script: Script) -> Fragments {
    let mut functions = BTreeSet::new();

    for function in &script.functions {
        if functions.contains(&function.name) {
            panic!("Can't re-define existing function `{}`.", function.name);
        }

        functions.insert(function.name.clone());
    }

    let mut by_function = Vec::new();

    for function in script.functions {
        let expressions = function.expressions;

        let mut fragments = BTreeMap::new();
        let mut next_fragment = None;

        for expression in expressions.into_iter().rev() {
            let payload = match expression {
                Expression::Binding { names } => {
                    FragmentPayload::BindingDefinitions { names }
                }
                Expression::Comment { text } => {
                    FragmentPayload::Comment { text }
                }
                Expression::Value(value) => FragmentPayload::Value(value),
                Expression::Word { name } => {
                    if functions.contains(&name) {
                        FragmentPayload::FunctionCall { name }
                    } else {
                        FragmentPayload::Word { name }
                    }
                }
            };

            let fragment = Fragment {
                address: FragmentAddress {
                    function: function.name.clone(),
                    next: next_fragment.take(),
                },
                payload,
            };
            next_fragment = Some(fragment.id());
            fragments.insert(fragment.id(), fragment);
        }

        let first_fragment = next_fragment;
        let fragments = FunctionFragments::new(first_fragment, fragments);

        by_function.push(Function {
            name: function.name,
            args: function.args,
            fragments,
        });
    }

    Fragments { by_function }
}

#[cfg(test)]
mod tests {
    use capi_process::Value;

    use crate::{
        repr::syntax::Script, stages::script_to_fragments::FragmentPayload,
    };

    use super::script_to_fragments;

    #[test]
    fn basic() {
        let mut script = Script::default();
        script.function("inc", ["x"], |s| {
            s.w("x").v(1).w("add");
        });

        let mut fragments = script_to_fragments(script);

        let fragments = fragments
            .by_function
            .remove(0)
            .fragments
            .map(|fragment| fragment.payload)
            .collect::<Vec<_>>();
        assert_eq!(
            fragments,
            vec![
                FragmentPayload::Word {
                    name: String::from("x")
                },
                FragmentPayload::Value(Value(1)),
                FragmentPayload::Word {
                    name: String::from("add")
                }
            ]
        );
    }

    #[test]
    fn duplicate_payload() {
        let mut script = Script::default();
        script.function("two", [], |s| {
            s.v(1).v(1).w("add");
        });

        let mut fragments = script_to_fragments(script);

        let fragments = fragments
            .by_function
            .remove(0)
            .fragments
            .map(|fragment| fragment.payload)
            .collect::<Vec<_>>();
        assert_eq!(
            fragments,
            vec![
                FragmentPayload::Value(Value(1)),
                FragmentPayload::Value(Value(1)),
                FragmentPayload::Word {
                    name: String::from("add")
                }
            ]
        );
    }
}
