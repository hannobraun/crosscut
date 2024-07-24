use std::collections::{BTreeMap, BTreeSet};

use crate::repr::{
    fragments::{
        Fragment, FragmentExpression, FragmentId, FragmentMap, FragmentParent,
        FragmentPayload, Fragments, Function,
    },
    syntax::{Expression, ReferenceKind, Script},
};

pub fn generate_fragments(script: Script) -> Fragments {
    let mut fragments = FragmentMap {
        inner: BTreeMap::new(),
    };
    let mut by_function = Vec::new();

    for function in script.functions {
        let mut scopes =
            process_function(function.args.clone(), &function.body);
        let (start, environment) = compile_block(
            function.body,
            FragmentParent::Function {
                name: function.name.clone(),
            },
            &mut scopes,
            &mut fragments,
        );

        assert!(
            environment.is_empty(),
            "Functions have no environment that they could access.\n\
            - Function: {}\n\
            - Environment: {environment:#?}",
            function.name,
        );

        by_function.push(Function {
            name: function.name,
            args: function.args,
            start,
        });
    }

    Fragments {
        inner: fragments,
        by_function,
    }
}

pub fn process_function(args: Vec<String>, body: &[Expression]) -> Scopes {
    let mut scopes = Scopes {
        stack: vec![Bindings {
            inner: args.into_iter().collect(),
        }],
    };

    process_block(body, &mut scopes);

    scopes
}

fn process_block(body: &[Expression], scopes: &mut Scopes) {
    for expression in body {
        if let Expression::Binding { names } = expression {
            for name in names.iter().cloned().rev() {
                // Inserting bindings unconditionally like this does mean
                // that bindings can overwrite previously defined bindings.
                // This is undesirable, but it'll do for now.
                scopes.stack.last_mut().unwrap().inner.insert(name);
            }
        }
        if let Expression::Block { body, .. } = expression {
            scopes.stack.push(Bindings {
                inner: BTreeSet::new(),
            });
            process_block(body, scopes);
        }
    }
}

#[derive(Debug)]
pub struct Scopes {
    stack: Vec<Bindings>,
}

impl Scopes {
    pub fn resolve_binding(&self, name: &str) -> Option<BindingResolved> {
        let mut scopes = self.stack.iter().rev();

        if let Some(scope) = scopes.next() {
            if scope.inner.contains(name) {
                return Some(BindingResolved::InScope);
            }
        }

        for scope in scopes {
            if scope.inner.contains(name) {
                return Some(BindingResolved::InEnvironment);
            }
        }

        None
    }
}

pub enum BindingResolved {
    InScope,
    InEnvironment,
}

#[derive(Debug)]
struct Bindings {
    inner: BTreeSet<String>,
}

pub fn compile_block(
    expressions: Vec<Expression>,
    parent: FragmentParent,
    scopes: &mut Scopes,
    fragments: &mut FragmentMap,
) -> (FragmentId, BTreeSet<String>) {
    let mut next = {
        let terminator = Fragment {
            parent: parent.clone(),
            payload: FragmentPayload::Terminator,
        };
        let terminator_id = terminator.id();

        fragments.inner.insert(terminator_id, terminator);

        terminator_id
    };
    let mut environment = BTreeSet::new();

    for expression in expressions.into_iter().rev() {
        let fragment = compile_expression(
            expression,
            parent.clone(),
            next,
            &mut environment,
            scopes,
            fragments,
        );

        next = fragment.id();

        fragments.inner.insert(fragment.id(), fragment);
    }

    (next, environment)
}

pub fn compile_expression(
    expression: Expression,
    parent: FragmentParent,
    next: FragmentId,
    environment: &mut BTreeSet<String>,
    scopes: &mut Scopes,
    fragments: &mut FragmentMap,
) -> Fragment {
    let expression = match expression {
        Expression::Binding { names } => {
            FragmentExpression::BindingDefinitions { names }
        }
        Expression::Block { body, environment } => {
            let (start, _) = compile_block(
                body,
                FragmentParent::Fragment { id: next },
                scopes,
                fragments,
            );
            FragmentExpression::Block { start, environment }
        }
        Expression::Comment { text } => FragmentExpression::Comment { text },
        Expression::Reference { name, kind } => match kind {
            Some(ReferenceKind::Binding) => {
                if let Some(BindingResolved::InEnvironment) =
                    scopes.resolve_binding(&name)
                {
                    environment.insert(name.clone());
                }
                FragmentExpression::ResolvedBinding { name }
            }
            Some(ReferenceKind::BuiltinFunction) => {
                FragmentExpression::ResolvedBuiltinFunction { name }
            }
            Some(ReferenceKind::HostFunction) => {
                FragmentExpression::ResolvedHostFunction { name }
            }
            Some(ReferenceKind::UserFunction) => {
                FragmentExpression::ResolvedUserFunction { name }
            }
            None => FragmentExpression::UnresolvedWord { name },
        },
        Expression::Value(value) => FragmentExpression::Value(value),
    };

    Fragment {
        parent,
        payload: FragmentPayload::Expression { expression, next },
    }
}

#[cfg(test)]
mod tests {
    use capi_process::Value;

    use crate::{
        passes::generate_fragments,
        repr::{
            fragments::{
                Fragment, FragmentExpression, FragmentParent, FragmentPayload,
                Fragments,
            },
            syntax::Script,
        },
    };

    #[test]
    fn duplicate_payload() {
        let mut script = Script::default();
        script.function("f", [], |s| {
            s.v(1).v(1);
        });

        let fragments = generate_fragments(script);

        let body = body(fragments);
        assert_eq!(
            body,
            [
                FragmentExpression::Value(Value(1i32.to_le_bytes())),
                FragmentExpression::Value(Value(1i32.to_le_bytes())),
            ]
        );
    }

    #[test]
    fn terminator() {
        let mut script = Script::default();
        script.function("f", [], |_| {});

        let mut fragments = generate_fragments(script);

        let start = fragments.by_function.remove(0).start;
        let last_fragment = fragments.inner.iter_from(start).last().unwrap();
        assert_eq!(last_fragment.payload, FragmentPayload::Terminator);
    }

    #[test]
    fn block_parent() {
        let mut script = Script::default();
        script.function("f", [], |s| {
            s.block(|_| {});
        });

        let mut fragments = generate_fragments(script);

        let start = fragments.by_function.remove(0).start;
        let function_fragments =
            fragments.inner.iter_from(start).collect::<Vec<_>>();
        let block_fragments = {
            let Fragment {
                payload:
                    FragmentPayload::Expression {
                        expression: FragmentExpression::Block { start, .. },
                        ..
                    },
                ..
            } = function_fragments[0]
            else {
                panic!("Expected block")
            };

            fragments.inner.iter_from(*start).collect::<Vec<_>>()
        };

        assert_eq!(
            function_fragments[0].parent,
            FragmentParent::Function {
                name: String::from("f")
            },
        );
        assert_eq!(
            block_fragments[0].parent,
            FragmentParent::Fragment {
                id: function_fragments[1].id()
            },
        );
    }

    fn body(mut fragments: Fragments) -> Vec<FragmentExpression> {
        let mut body = Vec::new();

        let start = fragments.by_function.remove(0).start;

        body.extend(fragments.inner.iter_from(start).filter_map(|fragment| {
            match &fragment.payload {
                FragmentPayload::Expression { expression, .. } => {
                    Some(expression.clone())
                }
                FragmentPayload::Terminator => None,
            }
        }));

        body
    }
}
