mod repr;
mod resolve;

pub use self::repr::{Dependencies, DependencyCluster};

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use itertools::Itertools;

    use crate::{
        code::{
            syntax::{
                Branch, Expression, Function, FunctionLocation, Member,
                NamedFunction, SyntaxTree,
            },
            Dependencies, FunctionCalls, IndexMap, Tokens,
        },
        host::NoHost,
    };

    // This test suite uses a custom compiler pipeline that creates all possible
    // permutations of the code, to make sure these tests don't accidentally
    // rely on a specific order of definitions in the code.
    //
    // As a consequence, these tests must not query compiler output by position,
    // as other test suites are generally free to do.

    #[test]
    fn function_dependencies_without_recursion() {
        let compiler_output = resolve_dependencies(
            "
                f: fn
                    br ->
                        g
                    end
                end
                g: fn
                    br ->
                        nop
                    end
                end
            ",
        );

        for (syntax_tree, dependencies) in compiler_output {
            let [f, g] = ["f", "g"].map(|name| {
                syntax_tree.function_by_name(name).unwrap().location()
            });

            assert_eq!(
                dependencies_by_function(&dependencies, &syntax_tree),
                [[g], [f]]
            );
        }
    }

    #[test]
    fn function_dependencies_in_the_presence_of_self_recursion() {
        let compiler_output = resolve_dependencies(
            "
                f: fn
                    br ->
                        g
                    end
                end
                g: fn
                    br ->
                        g
                    end
                end
            ",
        );

        for (syntax_tree, dependencies) in compiler_output {
            let [f, g] = ["f", "g"].map(|name| {
                syntax_tree.function_by_name(name).unwrap().location()
            });

            assert_eq!(
                dependencies_by_function(&dependencies, &syntax_tree),
                [[g], [f]]
            );
        }
    }

    #[test]
    fn function_dependencies_in_the_presence_of_mutual_recursion() {
        let compiler_output = resolve_dependencies(
            "
                f: fn
                    br ->
                        0 g
                    end
                end

                g: fn
                    br 0 ->
                        0 h
                    end
                    br n ->
                        0
                    end
                end

                h: fn
                    br 0 ->
                        1 h
                    end
                    br n ->
                        n g
                    end
                end
            ",
        );

        for (syntax_tree, dependencies) in compiler_output {
            let [f, g, h] = ["f", "g", "h"]
                .map(|name| syntax_tree.function_by_name(name).unwrap())
                .map(|function| function.location());

            let clusters =
                dependencies_by_function(&dependencies, &syntax_tree);
            let [cluster_a, cluster_b] = clusters.as_slice() else {
                panic!("Expected two clusters.");
            };

            // `g` and `h` are mutually recursive, so their order is not
            // defined.
            assert!(cluster_a.contains(&g));
            assert!(cluster_a.contains(&h));

            assert_eq!(cluster_b, &vec![f]);
        }
    }

    #[test]
    fn branch_dependencies_in_the_presence_of_mutual_recursion() {
        let compiler_output = resolve_dependencies(
            "
                f: fn
                    # a
                    br 0 ->
                        0 g
                    end

                    # b
                    br n ->
                        0
                    end
                end

                g: fn
                    # c
                    br 0 ->
                        1 g
                    end

                    # d
                    br n ->
                        n f
                    end
                end
            ",
        );

        for (syntax_tree, dependencies) in compiler_output {
            let [a, b, c, d] = {
                let mut branches = ["f", "g"]
                    .map(|name| syntax_tree.function_by_name(name).unwrap())
                    .into_iter()
                    .flat_map(|function| {
                        function.into_located_function().branches().map(
                            |branch| {
                                let name = branch
                                    .comment
                                    .clone()
                                    .unwrap()
                                    .lines
                                    .into_iter()
                                    .next()
                                    .unwrap();
                                (name, branch.location)
                            },
                        )
                    })
                    .collect::<BTreeMap<_, _>>();

                ["a", "b", "c", "d"].map(|name| branches.remove(name).unwrap())
            };

            let mut branches = dependencies
                .clusters()
                .next()
                .unwrap()
                .branches(&syntax_tree)
                .map(|branch| branch.location);
            let (first, second) =
                branches.by_ref().take(2).collect_tuple().unwrap();

            assert_eq!(first, b);
            assert_eq!(second, d);

            // The other two are mutually dependent, so there is no clear order.
            let rest = branches.collect::<BTreeSet<_>>();
            assert!(rest.contains(&a));
            assert!(rest.contains(&c));
        }
    }

    #[test]
    fn branch_dependencies_in_the_presence_of_local_functions() {
        let compiler_output = resolve_dependencies(
            "
                f: fn
                    # a
                    br ->
                        fn
                            # b
                            br 0 ->
                                0
                            end

                            # c
                            br n ->
                                f
                            end
                        end
                    end
                end
            ",
        );

        for (syntax_tree, dependencies) in compiler_output {
            for function in syntax_tree.all_functions() {
                println!(
                    "{}: {}",
                    function.location.display(&syntax_tree),
                    function.branches.len()
                );
            }

            let [a, b, c] = {
                let mut branches = syntax_tree
                    .all_functions()
                    .flat_map(|function| function.branches())
                    .map(|branch| {
                        let name = branch
                            .comment
                            .clone()
                            .unwrap()
                            .lines
                            .into_iter()
                            .next()
                            .unwrap();
                        (name, branch.location)
                    })
                    .collect::<BTreeMap<_, _>>();

                let a_b_c =
                    ["a", "b", "c"].map(|name| branches.remove(name).unwrap());

                assert!(branches.is_empty());

                a_b_c
            };

            let mut branches = dependencies
                .clusters()
                .next()
                .unwrap()
                .branches(&syntax_tree)
                .map(|branch| branch.location);

            let first = branches.next().unwrap();

            assert_eq!(first, b);

            // The other two are mutually dependent, so there is no clear order.
            let rest = branches.collect::<BTreeSet<_>>();
            assert!(rest.contains(&a));
            assert!(rest.contains(&c));
        }
    }

    #[test]
    fn sort_clusters_by_call_graph() {
        let compiler_output = resolve_dependencies(
            "
                f: fn
                    br ->
                        # Call a function that comes is placed after this one in
                        # the source code.
                        h
                    end
                end

                g: fn
                    br ->
                        nop
                    end
                end

                h: fn
                    br ->
                        # And for some variety, call a function that is placed
                        # before.
                        g
                    end
                end
            ",
        );

        for (syntax_tree, dependencies) in compiler_output {
            let [f, g, h] = ["f", "g", "h"].map(|name| {
                syntax_tree.function_by_name(name).unwrap().location()
            });

            assert_eq!(
                dependencies_by_function(&dependencies, &syntax_tree),
                [[g], [h], [f]]
            );
        }
    }

    #[test]
    fn consider_local_functions_in_call_graph() {
        let compiler_output = resolve_dependencies(
            "
                f: fn
                    br ->
                        fn
                            br ->
                                # Call a function that is placed after this one
                                # in the source code.
                                h
                            end
                        end
                    end
                end

                g: fn
                    br ->
                        nop
                    end
                end

                h: fn
                    br ->
                        fn
                            br ->
                                # And for some variety, call a function that is
                                # placed before.
                                g
                            end
                        end
                    end
                end
            ",
        );

        for (syntax_tree, dependencies) in compiler_output {
            let [(f, f_a), (h, h_a)] = ["f", "h"].map(|name| {
                let named = syntax_tree
                    .function_by_name(name)
                    .unwrap()
                    .into_located_function();
                let local = named
                    .find_single_branch()
                    .unwrap()
                    .expressions()
                    .next()
                    .unwrap()
                    .into_local_function()
                    .map(|function| function.location)
                    .unwrap();

                (named.location, local)
            });
            let g = syntax_tree.function_by_name("g").unwrap().location();

            assert_eq!(
                dependencies_by_function(&dependencies, &syntax_tree),
                [[g], [h_a], [h], [f_a], [f]],
            );
        }
    }

    fn resolve_dependencies(
        input: &str,
    ) -> impl Iterator<Item = (SyntaxTree, Dependencies)> {
        let tokens = Tokens::tokenize(input);
        let syntax_tree = SyntaxTree::parse(tokens);

        permutate_syntax_tree(syntax_tree).map(|syntax_tree| {
            let function_calls = FunctionCalls::resolve(&syntax_tree, &NoHost);
            let dependencies =
                Dependencies::resolve(&syntax_tree, &function_calls);

            (syntax_tree, dependencies)
        })
    }

    fn permutate_syntax_tree(
        original: SyntaxTree,
    ) -> impl Iterator<Item = SyntaxTree> {
        let k = original.named_functions.len();
        let permutations =
            original.named_functions.into_values().permutations(k);

        let mut syntax_trees = Vec::new();

        for named_functions in permutations {
            permutate_rest_of_named_functions(
                SyntaxTree::default(),
                named_functions.into_iter(),
                &mut syntax_trees,
            );
        }

        syntax_trees.into_iter()
    }

    fn permutate_rest_of_named_functions(
        syntax_tree: SyntaxTree,
        mut named_functions: impl Iterator<Item = NamedFunction> + Clone,
        syntax_trees: &mut Vec<SyntaxTree>,
    ) {
        match named_functions.next() {
            Some(named_function) => {
                for function in permutate_function(named_function.inner) {
                    let named_function = NamedFunction {
                        comment: named_function.comment.clone(),
                        name: named_function.name.clone(),
                        inner: function,
                    };
                    let mut syntax_tree = syntax_tree.clone();

                    syntax_tree.named_functions.push(named_function);
                    permutate_rest_of_named_functions(
                        syntax_tree,
                        named_functions.clone(),
                        syntax_trees,
                    );
                }
            }
            None => {
                syntax_trees.push(syntax_tree);
            }
        }
    }

    fn permutate_function(
        function: Function,
    ) -> impl Iterator<Item = Function> {
        let k = function.branches.len();
        let permutations = function.branches.into_values().permutations(k);

        let mut functions = Vec::new();

        for branches in permutations {
            permutate_rest_of_function(
                Function {
                    branches: IndexMap::default(),
                },
                branches.into_iter(),
                &mut functions,
            );
        }

        functions.into_iter()
    }

    fn permutate_rest_of_function(
        function: Function,
        mut branches: impl Iterator<Item = Branch> + Clone,
        functions: &mut Vec<Function>,
    ) {
        match branches.next() {
            Some(branch) => {
                for branch in permutate_branch(branch) {
                    let mut function = function.clone();
                    function.branches.push(branch);
                    permutate_rest_of_function(
                        function,
                        branches.clone(),
                        functions,
                    );
                }
            }
            None => {
                functions.push(function);
            }
        }
    }

    fn permutate_branch(branch: Branch) -> impl Iterator<Item = Branch> {
        let mut branches = Vec::new();

        permutate_rest_of_branch(
            Branch {
                comment: branch.comment,
                parameters: branch.parameters,
                body: IndexMap::default(),
            },
            // `btree_map::IntoValues` does not implement `Clone`. This
            // workaround exchanges it for an iterator that does.
            branch.body.into_values().collect::<Vec<_>>().into_iter(),
            &mut branches,
        );

        branches.into_iter()
    }

    fn permutate_rest_of_branch(
        mut branch: Branch,
        mut body: impl Iterator<Item = Member> + Clone,
        branches: &mut Vec<Branch>,
    ) {
        loop {
            match body.next() {
                Some(Member::Expression {
                    expression: Expression::LocalFunction { function },
                    signature,
                }) => {
                    for function in permutate_function(function) {
                        let mut branch = branch.clone();
                        branch.body.push(Member::Expression {
                            expression: Expression::LocalFunction { function },
                            signature: signature.clone(),
                        });
                        permutate_rest_of_branch(
                            branch,
                            body.clone(),
                            branches,
                        );
                    }

                    break;
                }
                Some(member) => {
                    branch.body.push(member);
                }
                None => {
                    branches.push(branch);
                    break;
                }
            }
        }
    }

    fn dependencies_by_function(
        dependencies: &Dependencies,
        syntax_tree: &SyntaxTree,
    ) -> Vec<Vec<FunctionLocation>> {
        dependencies
            .clusters()
            .map(|cluster| {
                cluster
                    .functions(syntax_tree)
                    .map(|function| function.location)
                    .collect()
            })
            .collect()
    }
}
