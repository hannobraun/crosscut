use anyhow::anyhow;
use crosscut_compiler::{
    code::{
        syntax::{Binding, Branch, Located, MemberLocation, Parameter},
        DependencyCluster, FunctionCalls, Functions, Type, Types,
    },
    source_map::SourceMap,
};
use crosscut_runtime::Effect;

use super::{Breakpoints, DebugMember};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DebugBranch {
    pub parameters: Vec<DebugParameter>,
    pub body: Vec<DebugMember>,
    pub is_active: bool,
}

impl DebugBranch {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        branch: Located<Branch>,
        active_expression: Option<&MemberLocation>,
        is_in_innermost_active_function: bool,
        cluster: &DependencyCluster,
        functions: &Functions,
        function_calls: &FunctionCalls,
        types: &Types,
        source_map: &SourceMap,
        breakpoints: &Breakpoints,
        effect: Option<&Effect>,
    ) -> Self {
        let body = branch
            .as_ref()
            .body()
            .map(|member| {
                DebugMember::new(
                    member.fragment.clone(),
                    member.location,
                    active_expression,
                    is_in_innermost_active_function,
                    cluster,
                    functions,
                    function_calls,
                    types,
                    source_map,
                    breakpoints,
                    effect,
                )
            })
            .collect::<Vec<_>>();
        let parameters = branch
            .as_ref()
            .parameters()
            .map(|parameter| match parameter.fragment {
                Parameter::Binding {
                    binding: Binding { name },
                    // We're ignoring this, because it's only the type that the
                    // developer specified explicitly. But what we're interested
                    // in is the type that was inferred by the compiler. (Which
                    // could be identical to this one, or this one could be
                    // empty, while the compiler figured out something useful.)
                    type_: _,
                } => {
                    let type_ =
                        types.type_of_parameter(&parameter.location).cloned();

                    DebugParameter {
                        name: name.clone(),
                        type_,
                    }
                }
                Parameter::Literal { value } => DebugParameter {
                    name: format!("{value:?}"),
                    type_: None,
                },
            })
            .collect();

        let is_active = body
            .iter()
            .any(|expression| expression.data.state.is_active());

        Self {
            parameters,
            body,
            is_active,
        }
    }

    pub fn active_expression(&self) -> anyhow::Result<&DebugMember> {
        self.body
            .iter()
            .find(|expression| expression.data.state.is_active())
            .ok_or_else(|| {
                anyhow!(
                    "Expected active expression in branch, but could not find \
                    any. Branch:\n\
                    {self:#?}"
                )
            })
    }

    pub fn expression_after(
        &self,
        expression: &MemberLocation,
    ) -> anyhow::Result<Option<&DebugMember>> {
        if !self.body.iter().any(|f| f.data.location == *expression) {
            return Err(anyhow!(
                "Expected expression to be in branch, but could not find it. \
                Expression:\n\
                {expression:#?}\n\
                Branch:\n\
                {self:#?}"
            ));
        }

        let mut expressions = self
            .body
            .iter()
            .skip_while(|f| f.data.location != *expression);

        // This is the expression we've been passed as an argument. Need to
        // ignore it, to advance the iterator to the one we're actually looking
        // for.
        assert_eq!(
            expressions
                .next()
                .as_ref()
                .map(|expression| &expression.data.location),
            Some(expression)
        );

        Ok(expressions.next())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DebugParameter {
    pub name: String,
    pub type_: Option<Type>,
}
