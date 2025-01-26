use std::collections::BTreeSet;

use super::syntax::{Expression, MemberLocation, SyntaxTree};

/// # Tracks tail expressions
///
/// A tail expression is the last expression in a function. Any function that
/// has at least one expression, has a tail expression.
///
/// Knowing if a given expression is a tail, is relevant if that expression is a
/// call to a user-defined function. Those tail calls get eliminated (which is
/// called "tail call elimination", or "tail call optimization"), to reduce
/// stack usage and prevent unlimited stack growth in the presence of recursive
/// tail calls.
#[derive(Debug)]
pub struct TailExpressions {
    inner: BTreeSet<MemberLocation>,
}

impl TailExpressions {
    /// # Find all tail expressions
    pub fn find(syntax_tree: &SyntaxTree) -> Self {
        let mut tail_expressions = BTreeSet::new();

        for function in syntax_tree.all_functions() {
            for branch in function.branches() {
                if let Some(expression) = branch.expressions().last() {
                    if let Expression::Identifier { .. } = expression.fragment {
                        tail_expressions.insert(expression.location);
                    }
                }
            }
        }

        Self {
            inner: tail_expressions,
        }
    }

    /// # Determine if the expression at the provided location is a tail
    pub fn is_tail_expression(&self, location: &MemberLocation) -> bool {
        self.inner.contains(location)
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::code::{syntax::SyntaxTree, tokens::Tokens};

    use super::TailExpressions;

    #[test]
    fn find_tail_calls() {
        // The last expression in a function should be marked as being a tail.
        // Others should not be.

        let (syntax_tree, tail_expressions) = find_tail_expressions(
            r"
                f: fn
                    br ->
                        not_tail
                        tail
                    end
                end
            ",
        );

        let (not_tail, tail) = syntax_tree
            .function_by_name("f")
            .unwrap()
            .into_located_function()
            .find_single_branch()
            .unwrap()
            .expressions()
            .map(|expression| expression.location)
            .collect_tuple()
            .unwrap();

        assert!(!tail_expressions.is_tail_expression(&not_tail));
        assert!(tail_expressions.is_tail_expression(&tail));
    }

    #[test]
    fn ignore_comments() {
        // A comment being located after a tail call should not confuse the
        // analysis.

        let (syntax_tree, tail_expressions) = find_tail_expressions(
            r"
                f: fn
                    br ->
                        not_tail
                        tail
                        # This is a comment.
                    end
                end
            ",
        );

        let tail = syntax_tree
            .function_by_name("f")
            .unwrap()
            .into_located_function()
            .find_single_branch()
            .unwrap()
            .expressions()
            .map(|expression| expression.location)
            .nth(1)
            .unwrap();

        assert!(tail_expressions.is_tail_expression(&tail));
    }

    pub fn find_tail_expressions(input: &str) -> (SyntaxTree, TailExpressions) {
        let tokens = Tokens::tokenize(input);
        let syntax_tree = SyntaxTree::parse(tokens);
        let tail_expressions = TailExpressions::find(&syntax_tree);

        (syntax_tree, tail_expressions)
    }
}
