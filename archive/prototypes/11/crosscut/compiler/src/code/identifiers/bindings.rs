use std::collections::{BTreeMap, BTreeSet};

use crate::code::syntax::{
    Binding, Branch, Expression, Function, FunctionLocation, Located,
    MemberLocation, ParameterLocation, SyntaxTree,
};

/// # Tracks bindings
///
/// A binding is an identifier that refers to a value that was previously bound
/// to a name.
#[derive(Debug)]
pub struct Bindings {
    bindings: BindingsMap,
    environments: EnvironmentsMap,
}

impl Bindings {
    /// # Resolve all bindings
    pub fn resolve(syntax_tree: &SyntaxTree) -> Self {
        let mut bindings = BTreeMap::new();
        let mut environments = BTreeMap::new();

        resolve_bindings(syntax_tree, &mut bindings, &mut environments);

        Self {
            bindings,
            environments,
        }
    }

    /// # Determine, if the expression at the given location is a binding
    pub fn is_binding(
        &self,
        location: &MemberLocation,
    ) -> Option<&ParameterLocation> {
        self.bindings.get(location)
    }

    /// # Access the environment of the function at the provided location
    pub fn environment_of(&self, location: &FunctionLocation) -> &Environment {
        static EMPTY: Environment = Environment::new();
        self.environments.get(location).unwrap_or(&EMPTY)
    }
}

type BindingsMap = BTreeMap<MemberLocation, ParameterLocation>;
type EnvironmentsMap = BTreeMap<FunctionLocation, Environment>;

/// # The environment of a function
///
/// The environment of a function is the set of bindings it accesses, that are
/// not its own parameters.
#[derive(Clone, Debug)]
pub struct Environment {
    inner: BTreeSet<ParameterLocation>,
}

impl Environment {
    /// # Create a new instance of [`Environment`]
    pub const fn new() -> Self {
        Self {
            inner: BTreeSet::new(),
        }
    }

    /// # Iterate over the bindings in the environment
    ///
    /// ## Panics
    ///
    /// Panics, if a binding tracked in this [`Environment`] can not be found in
    /// the provided [`SyntaxTree`].
    pub fn bindings<'r>(
        &'r self,
        syntax_tree: &'r SyntaxTree,
    ) -> impl Iterator<Item = Located<&'r Binding>> {
        self.inner.iter().map(|location| {
            let Some(binding) = syntax_tree.binding_by_location(location)
            else {
                panic!(
                    "This function expects to find all tracked bindings in the \
                    provided `SyntaxTree`."
                );
            };

            binding
        })
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

fn resolve_bindings(
    syntax_tree: &SyntaxTree,
    bindings: &mut BindingsMap,
    environments: &mut EnvironmentsMap,
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
    bindings: &mut BindingsMap,
    environments: &mut EnvironmentsMap,
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
    bindings: &mut BindingsMap,
    environment: &mut Environment,
    environments: &mut EnvironmentsMap,
) {
    scopes.push(
        branch
            .bindings()
            .map(|binding| (binding.location.clone(), binding.cloned()))
            .collect(),
    );

    for expression in branch.expressions() {
        match expression.fragment {
            Expression::Identifier { name } => {
                let binding = scopes.iter().rev().find_map(|scope| {
                    scope.values().find(|binding| binding.name == *name)
                });

                if let Some(binding) = binding {
                    bindings
                        .insert(expression.location, binding.location.clone());

                    if let Some(scope) = scopes.last() {
                        if !scope.contains_key(&binding.location) {
                            // The binding is not known in the current scope,
                            // which means it comes from a parent scope.
                            environment.inner.insert(binding.location.clone());
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

                for binding in child_environment.inner {
                    if let Some(bindings) = scopes.last() {
                        if !bindings.contains_key(&binding) {
                            // The child function that we just resolved bindings
                            // in has a function in its environment that is not
                            // known in the current scope.
                            //
                            // This means it must come from this function's
                            // parent scopes, and must be added to this
                            // environment too.
                            environment.inner.insert(binding);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    scopes.pop();
}

type Scopes = Vec<BTreeMap<ParameterLocation, Located<Binding>>>;

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
                    br parameter ->
                        parameter
                        no_parameter
                    end
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

        assert!(bindings.is_binding(&parameter).is_some());
        assert!(bindings.is_binding(&no_parameter).is_none());
    }

    #[test]
    fn resolve_parameter_of_parent_function() {
        // An identifier with the same name as the parameter of a parent
        // function should be resolved as a binding.

        let (syntax_tree, bindings) = resolve_bindings(
            r"
                f: fn
                    br parameter ->
                        fn
                            br ->
                                parameter
                                no_parameter
                            end
                        end
                    end
                end
            ",
        );

        let f = syntax_tree
            .function_by_name("f")
            .unwrap()
            .into_located_function();
        let f_branch = f.find_single_branch().unwrap();
        let binding = f_branch.bindings().next().unwrap();
        let f_local = f_branch
            .expressions()
            .filter_map(|expression| expression.into_local_function())
            .next()
            .unwrap()
            .cloned();
        let (parameter, no_parameter) = f_local
            .as_ref()
            .find_single_branch()
            .unwrap()
            .expressions()
            .map(|expression| expression.location)
            .collect_tuple()
            .unwrap();

        assert!(bindings.is_binding(&parameter).is_some());
        assert!(bindings.is_binding(&no_parameter).is_none());

        assert!(bindings
            .environment_of(&f_local.location)
            .bindings(&syntax_tree)
            .any(|b| b == binding));
    }

    #[test]
    fn resolve_parameter_that_shadows_parent_parameter() {
        // If a function parameter has the same name as the parameter of a
        // parent function, the closer parameter should be resolved.

        let (syntax_tree, bindings) = resolve_bindings(
            r"
                f: fn
                    br parameter ->
                        fn
                            br parameter ->
                                parameter
                            end
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
            .filter_map(|expression| expression.into_local_function())
            .next()
            .unwrap()
            .cloned();
        let function = function.as_ref();
        let branch = function.find_single_branch().unwrap();
        let parameter = branch
            .expressions()
            .map(|expression| expression.location)
            .next()
            .unwrap();

        if let Some(location) = bindings.is_binding(&parameter) {
            assert_eq!(*location.parent, branch.location);
        } else {
            panic!("Expected identifier to be a binding.");
        }
        assert!(bindings.is_binding(&parameter).is_some());
    }

    #[test]
    fn do_not_resolve_parameter_of_child_function() {
        // Identifiers that share a name with a parameter of a child function
        // should not be resolved as bindings.

        let (syntax_tree, bindings) = resolve_bindings(
            r"
                f: fn
                    br ->
                        fn
                            br child_parameter ->
                            end
                        end
                    
                        child_parameter
                    end
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

        assert!(bindings.is_binding(&child_parameter).is_none());
    }

    #[test]
    fn track_indirect_bindings_in_environment() {
        // If a function doesn't access a binding from a parent scope itself,
        // but one of its child functions does, the binding still needs to be
        // part of the function's environment.

        let (syntax_tree, bindings) = resolve_bindings(
            r"
                f: fn
                    br binding ->
                        fn
                            br ->
                                fn
                                    br ->
                                        binding
                                    end
                                end
                            end
                        end
                    end
                end
            ",
        );

        let f = syntax_tree
            .function_by_name("f")
            .unwrap()
            .into_located_function();
        let branch = f.find_single_branch().unwrap();
        let binding = branch.bindings().next().unwrap();
        let f_local = branch
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

        assert!(bindings
            .environment_of(&f_local)
            .bindings(&syntax_tree)
            .any(|b| b == binding));
    }

    fn resolve_bindings(input: &str) -> (SyntaxTree, Bindings) {
        let tokens = Tokens::tokenize(input);
        let syntax_tree = SyntaxTree::parse(tokens);
        let bindings = Bindings::resolve(&syntax_tree);

        (syntax_tree, bindings)
    }
}
