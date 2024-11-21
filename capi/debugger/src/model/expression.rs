use capi_compiler::{
    code::{
        Cluster, Expression, ExpressionLocation, FunctionLocation,
        StableFunctions,
    },
    source_map::SourceMap,
};
use capi_runtime::Effect;

use super::{Breakpoints, DebugFunction};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DebugExpression {
    pub data: DebugExpressionData,
    pub kind: DebugExpressionKind,
}

impl DebugExpression {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        expression: Expression,
        location: ExpressionLocation,
        active_expression: Option<&ExpressionLocation>,
        is_in_innermost_active_function: bool,
        cluster: &Cluster,
        functions: &StableFunctions,
        source_map: &SourceMap,
        breakpoints: &Breakpoints,
        effect: Option<&Effect>,
    ) -> Self {
        let state = if Some(&location) == active_expression {
            if is_in_innermost_active_function {
                DebugExpressionState::InnermostActiveExpression
            } else {
                DebugExpressionState::ActiveCaller
            }
        } else {
            DebugExpressionState::NotActive
        };

        let has_durable_breakpoint = source_map
            .expression_to_instructions(&location)
            .iter()
            .any(|instruction| breakpoints.durable_at(instruction));

        let active_effect = effect.and_then(|effect| {
            if state.is_innermost_active_expression() {
                Some(*effect)
            } else {
                None
            }
        });

        let data = DebugExpressionData {
            expression: expression.clone(),
            location: location.clone(),
            state,
            has_durable_breakpoint,
            effect: active_effect,
        };
        let kind = DebugExpressionKind::new(
            expression,
            location,
            active_expression,
            is_in_innermost_active_function,
            cluster,
            functions,
            source_map,
            breakpoints,
            effect,
        );

        Self { kind, data }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DebugExpressionData {
    pub expression: Expression,
    pub location: ExpressionLocation,
    pub state: DebugExpressionState,
    pub has_durable_breakpoint: bool,
    pub effect: Option<Effect>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DebugExpressionState {
    InnermostActiveExpression,
    ActiveCaller,
    NotActive,
}

impl DebugExpressionState {
    /// # Indicate whether this is the innermost active expression
    ///
    /// The innermost active expression is the active expression in the
    /// innermost active function. The expression where the process is currently
    /// stopped at.
    pub fn is_innermost_active_expression(&self) -> bool {
        matches!(self, Self::InnermostActiveExpression)
    }

    /// # Indicate whether the expression is active
    ///
    /// A expression is active, either if the process is currently stopped here,
    /// or if it calls an active function (which is a function that contains an
    /// active expression).
    pub fn is_active(&self) -> bool {
        matches!(self, Self::InnermostActiveExpression | Self::ActiveCaller)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DebugExpressionKind {
    CallToFunction { name: String },
    CallToFunctionRecursive { name: String },
    CallToIntrinsic { name: String },
    Comment { text: String },
    Function { function: DebugFunction },
    UnresolvedIdentifier { name: String },
    UnresolvedLocalFunction,
    Value { as_string: String },
}

impl DebugExpressionKind {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        expression: Expression,
        location: ExpressionLocation,
        active_expression: Option<&ExpressionLocation>,
        is_in_innermost_active_function: bool,
        cluster: &Cluster,
        functions: &StableFunctions,
        source_map: &SourceMap,
        breakpoints: &Breakpoints,
        effect: Option<&Effect>,
    ) -> Self {
        match expression {
            Expression::CallToUserDefinedFunction { hash, .. } => {
                let function = functions
                    .named_by_hash(&hash)
                    .expect("Expecting function referenced by call to exist.");
                let name = function.name.clone();

                Self::CallToFunction { name }
            }
            Expression::CallToUserDefinedFunctionRecursive {
                index, ..
            } => {
                let called_function_location =
                    cluster.functions.get(&index).expect(
                        "The index of a recursive call must be valid within \
                        the calling function's cluster.",
                    );
                let FunctionLocation::NamedFunction {
                    index: called_function_index,
                } = called_function_location
                else {
                    unreachable!(
                        "Only named functions can be called recursively."
                    );
                };
                let called_function =
                    functions.named.get(called_function_index).expect(
                        "Expecting to find expression referred to from a \
                        cluster.",
                    );
                let name = called_function.name.clone();

                Self::CallToFunctionRecursive { name }
            }
            Expression::CallToIntrinsicFunction { intrinsic, .. } => {
                Self::CallToIntrinsic {
                    name: intrinsic.to_string(),
                }
            }
            Expression::Comment { text } => Self::Comment {
                text: format!("# {text}"),
            },
            Expression::LiteralNumber { value } => Self::Value {
                as_string: value.to_string(),
            },
            Expression::LocalFunction { hash } => {
                let function = functions
                    .by_hash(&hash)
                    .expect("Resolved local function must exist.");

                let function = DebugFunction::new(
                    function.fragment.clone(),
                    None,
                    FunctionLocation::AnonymousFunction { location },
                    active_expression,
                    is_in_innermost_active_function,
                    cluster,
                    functions,
                    source_map,
                    breakpoints,
                    effect,
                );

                Self::Function { function }
            }
            Expression::LocalFunctionRecursive { index } => {
                let function = {
                    let location = cluster.functions.get(&index).expect(
                        "Resolved recursive local function must exist in \
                        cluster.",
                    );
                    functions
                        .by_location(location)
                        .expect("Function referred to from cluster must exist.")
                };

                let function = DebugFunction::new(
                    function.fragment.clone(),
                    None,
                    FunctionLocation::AnonymousFunction { location },
                    active_expression,
                    is_in_innermost_active_function,
                    cluster,
                    functions,
                    source_map,
                    breakpoints,
                    effect,
                );

                Self::Function { function }
            }
            Expression::UnresolvedIdentifier { name, .. } => {
                Self::UnresolvedIdentifier { name }
            }
            Expression::UnresolvedLocalFunction => {
                Self::UnresolvedLocalFunction
            }
        }
    }
}
