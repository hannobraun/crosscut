use std::collections::{BTreeMap, BTreeSet};

use crate::repr::{
    fragments::{
        Fragment, FragmentAddress, FragmentAddressParent, FragmentId,
        FragmentPayload, Fragments, Function, FunctionFragments,
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
        let fragments = compile_function(
            function.name.clone(),
            &function.args,
            function.expressions,
            &functions,
        );
        by_function.push(Function {
            name: function.name,
            args: function.args,
            fragments,
        });
    }

    Fragments { by_function }
}

fn compile_function(
    name: String,
    args: &[String],
    body: Vec<Expression>,
    functions: &BTreeSet<String>,
) -> FunctionFragments {
    let mut bindings: BTreeSet<_> = args.iter().cloned().collect();

    for expression in &body {
        if let Expression::Binding { names } = expression {
            for name in names.iter().cloned().rev() {
                // Inserting bindings unconditionally like this does mean that
                // bindings can overwrite previously defined bindings. This is
                // undesirable, but it'll do for now.
                bindings.insert(name);
            }
        }
    }

    let mut function_fragments = BTreeMap::new();
    let mut next_fragment = None;

    for expression in body.into_iter().rev() {
        let fragment = compile_expression(
            expression,
            next_fragment.take(),
            name.clone(),
            &bindings,
            functions,
        );
        next_fragment = Some(fragment.id());
        function_fragments.insert(fragment.id(), fragment);
    }

    let first_fragment = next_fragment;
    FunctionFragments::new(first_fragment, function_fragments)
}

fn compile_expression(
    expression: Expression,
    next_fragment: Option<FragmentId>,
    function_name: String,
    bindings: &BTreeSet<String>,
    functions: &BTreeSet<String>,
) -> Fragment {
    let payload = match expression {
        Expression::Binding { names } => {
            FragmentPayload::BindingDefinitions { names }
        }
        Expression::Comment { text } => FragmentPayload::Comment { text },
        Expression::Value(value) => FragmentPayload::Value(value),
        Expression::Word { name } => {
            // The way this is written, bindings shadow built-in functions,
            // while user-defined functions shadow anything else.
            //
            // This isn't desirable. There should at least be a warning, if such
            // shadowing isn't forbidden outright. It'll do for now though.
            if functions.contains(&name) {
                FragmentPayload::FunctionCall { name }
            } else if bindings.contains(&name) {
                FragmentPayload::BindingEvaluation { name }
            } else {
                // This doesn't check whether the built-in function exists, and
                // given how built-in functions are currently defined, that's
                // not practical to implement.
                FragmentPayload::BuiltinCall { name }
            }
        }
    };

    Fragment {
        address: FragmentAddress {
            parent: FragmentAddressParent::Function {
                name: function_name,
            },
            next: next_fragment,
        },
        payload,
    }
}

#[cfg(test)]
mod tests {
    use capi_process::Value;

    use crate::{
        repr::syntax::Script, stages::script_to_fragments::FragmentPayload,
    };

    use super::script_to_fragments;

    #[test]
    fn arg_eval() {
        let mut script = Script::default();
        script.function("f", ["a"], |s| {
            s.w("a");
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
            vec![FragmentPayload::BindingEvaluation {
                name: String::from("a")
            }]
        );
    }

    #[test]
    fn binding_eval() {
        let mut script = Script::default();
        script.function("f", [], |s| {
            s.v(0).bind(["b"]).w("b");
        });

        let mut fragments = script_to_fragments(script);

        let fragment = fragments
            .by_function
            .remove(0)
            .fragments
            .map(|fragment| fragment.payload)
            .collect::<Vec<_>>()
            .last()
            .cloned()
            .unwrap();
        assert_eq!(
            fragment,
            FragmentPayload::BindingEvaluation {
                name: String::from("b")
            }
        );
    }

    #[test]
    fn builtin_call() {
        let mut script = Script::default();
        script.function("f", [], |s| {
            s.w("builtin");
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
            vec![FragmentPayload::BuiltinCall {
                name: String::from("builtin")
            }]
        );
    }

    #[test]
    fn function_call() {
        let mut script = Script::default();
        script.function("f", [], |_| {});
        script.function("g", [], |s| {
            s.w("f");
        });

        let mut fragments = script_to_fragments(script);

        let fragments = fragments
            .by_function
            .remove(1)
            .fragments
            .map(|fragment| fragment.payload)
            .collect::<Vec<_>>();
        assert_eq!(
            fragments,
            vec![FragmentPayload::FunctionCall {
                name: String::from("f")
            }]
        );
    }

    #[test]
    fn duplicate_payload() {
        let mut script = Script::default();
        script.function("f", [], |s| {
            s.v(1).v(1);
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
            ]
        );
    }
}
