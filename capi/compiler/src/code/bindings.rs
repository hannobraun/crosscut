use std::collections::{BTreeMap, BTreeSet};

use super::syntax::{
    Branch, Expression, Function, FunctionLocation, Located, MemberLocation,
    Pattern, SyntaxTree,
};

/// # Tracks bindings
///
/// A binding is an identifier that refers to a value that was previously bound
/// to a name.
#[derive(Debug)]
pub struct Bindings {
    bindings: BTreeSet<MemberLocation>,
    environments: BTreeMap<FunctionLocation, Environment>,
}

impl Bindings {
    /// # Resolve all bindings
    pub fn resolve(syntax_tree: &SyntaxTree) -> Self {
        let mut bindings = BTreeSet::new();
        let mut environments = BTreeMap::new();

        resolve_bindings(syntax_tree, &mut bindings, &mut environments);

        Self {
            bindings,
            environments,
        }
    }

    /// # Determine, if the expression at the given location is a binding
    pub fn is_binding(&self, location: &MemberLocation) -> bool {
        self.bindings.contains(location)
    }

    /// # Access the environment of the function at the provided location
    pub fn environment_of(&self, location: &FunctionLocation) -> &Environment {
        static EMPTY: Environment = Environment::new();
        self.environments.get(location).unwrap_or(&EMPTY)
    }
}

/// # The environment of a function
///
/// The environment of a function is the set of bindings it accesses, that are
/// not its own parameters.
pub type Environment = BTreeSet<String>;

fn resolve_bindings(
    syntax_tree: &SyntaxTree,
    bindings: &mut BTreeSet<MemberLocation>,
    environments: &mut BTreeMap<FunctionLocation, Environment>,
) {
    let mut scopes = Scopes::new();

    for function in syntax_tree.named_functions() {
        resolve_bindings_in_function(
            function.into_located_function(),
            &mut scopes,
            bindings,
            environments,
        );
    }
}

fn resolve_bindings_in_function(
    function: Located<&Function>,
    scopes: &mut Scopes,
    bindings: &mut BTreeSet<MemberLocation>,
    environments: &mut BTreeMap<FunctionLocation, Environment>,
) -> Environment {
    let location = function.location.clone();
    let mut environment = Environment::new();

    for branch in function.branches() {
        resolve_bindings_in_branch(
            branch,
            scopes,
            bindings,
            &mut environment,
            environments,
        );
    }

    let overwritten_value = environments.insert(location, environment.clone());
    assert!(
        overwritten_value.is_none(),
        "Every function should be processed only once."
    );

    environment
}

fn resolve_bindings_in_branch(
    branch: Located<&Branch>,
    scopes: &mut Scopes,
    bindings: &mut BTreeSet<MemberLocation>,
    environment: &mut Environment,
    environments: &mut BTreeMap<FunctionLocation, Environment>,
) {
    let identifiers =
        branch.parameters.clone().into_iter().filter_map(|pattern| {
            if let Pattern::Identifier { name } = pattern {
                Some(name)
            } else {
                None
            }
        });

    scopes.push(identifiers.collect());

    for expression in branch.expressions() {
        match expression.fragment {
            Expression::Identifier { name } => {
                let is_known_binding =
                    scopes.iter().any(|scope| scope.contains(name));

                if is_known_binding {
                    bindings.insert(expression.location);

                    if let Some(scope) = scopes.last() {
                        if !scope.contains(name) {
                            // The binding is not known in the current scope,
                            // which means it comes from a parent scope.
                            environment.insert(name.clone());
                        }
                    }
                }
            }
            Expression::LocalFunction { function } => {
                let function = Located {
                    fragment: function,
                    location: FunctionLocation::from(expression.location),
                };

                let child_environment = resolve_bindings_in_function(
                    function,
                    scopes,
                    bindings,
                    environments,
                );

                for name in child_environment {
                    if let Some(bindings) = scopes.last() {
                        if !bindings.contains(&name) {
                            // The child function that we just resolved bindings
                            // in has a function in its environment that is not
                            // known in the current scope.
                            //
                            // This means it must come from this function's
                            // parent scopes, and must be added to this
                            // environment too.
                            environment.insert(name.clone());
                        }
                    }
                }
            }
            _ => {}
        }
    }

    scopes.pop();
}

type Scopes = Vec<BindingsInScope>;
type BindingsInScope = BTreeSet<String>;

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::code::{
        syntax::{Expression, FunctionLocation, SyntaxTree},
        Tokens,
    };

    use super::Bindings;

    #[test]
    fn resolve_parameter_of_function() {
        // An identifier with the same name as a function's parameter should be
        // resolved as a binding.

        let (syntax_tree, bindings) = resolve_bindings(
            r"
                f: fn
                    \ parameter ->
                        parameter
                        no_parameter
                end
            ",
        );

        let (parameter, no_parameter) = syntax_tree
            .function_by_name("f")
            .unwrap()
            .into_located_function()
            .find_single_branch()
            .unwrap()
            .expressions()
            .map(|expression| expression.location)
            .collect_tuple()
            .unwrap();

        assert!(bindings.is_binding(&parameter));
        assert!(!bindings.is_binding(&no_parameter));
    }

    #[test]
    fn resolve_parameter_of_parent_function() {
        // An identifier with the same name as the parameter of a parent
        // function should be resolved as a binding.

        let (syntax_tree, bindings) = resolve_bindings(
            r"
                f: fn
                    \ parameter ->
                        fn
                            \ ->
                                parameter
                                no_parameter
                        end
                end
            ",
        );

        let function = syntax_tree
            .function_by_name("f")
            .unwrap()
            .into_located_function()
            .find_single_branch()
            .unwrap()
            .expressions()
            .filter_map(|expression| expression.into_local_function())
            .next()
            .unwrap()
            .cloned();
        let (parameter, no_parameter) = function
            .as_ref()
            .find_single_branch()
            .unwrap()
            .expressions()
            .map(|expression| expression.location)
            .collect_tuple()
            .unwrap();

        assert!(bindings.is_binding(&parameter));
        assert!(!bindings.is_binding(&no_parameter));

        assert!(bindings
            .environment_of(&function.location)
            .contains("parameter"));
    }

    #[test]
    fn do_not_resolve_parameter_of_child_function() {
        // Identifiers that share a name with a parameter of a child function
        // should not be resolved as bindings.

        let (syntax_tree, bindings) = resolve_bindings(
            r"
                f: fn
                    \ ->
                        fn
                            \ child_parameter ->
                        end
                    
                    child_parameter
                end
            ",
        );

        let child_parameter = syntax_tree
            .function_by_name("f")
            .unwrap()
            .into_located_function()
            .find_single_branch()
            .unwrap()
            .expressions()
            .map(|expression| expression.location)
            .nth(1)
            .unwrap();

        assert!(!bindings.is_binding(&child_parameter));
    }

    #[test]
    fn track_indirect_bindings_in_environment() {
        // If a function doesn't access a binding from a parent scope itself,
        // but one of its child functions does, the binding still needs to be
        // part of the function's environment.

        let (syntax_tree, bindings) = resolve_bindings(
            r"
                f: fn
                    \ binding ->
                        fn
                            \ ->
                                fn
                                    \ ->
                                        binding
                                end
                        end
                        
                end
            ",
        );

        let function = syntax_tree
            .function_by_name("f")
            .unwrap()
            .into_located_function()
            .find_single_branch()
            .unwrap()
            .expressions()
            .filter_map(|expression| {
                if let Expression::LocalFunction { function: _ } =
                    expression.fragment
                {
                    let location = FunctionLocation::from(expression.location);
                    Some(location)
                } else {
                    None
                }
            })
            .next()
            .unwrap();

        assert!(bindings.environment_of(&function).contains("binding"));
    }

    fn resolve_bindings(input: &str) -> (SyntaxTree, Bindings) {
        let tokens = Tokens::tokenize(input);
        let syntax_tree = SyntaxTree::parse(tokens);
        let bindings = Bindings::resolve(&syntax_tree);

        (syntax_tree, bindings)
    }
}
