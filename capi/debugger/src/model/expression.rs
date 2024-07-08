use capi_compiler::{
    repr::syntax::{self, ExpressionKind},
    source_map::SourceMap,
};
use capi_process::{EvaluatorEffect, Location, Process};

#[derive(Clone, Eq, PartialEq)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub location: Option<Location>,
    pub has_durable_breakpoint: bool,
    pub is_comment: bool,
    pub is_on_call_stack: bool,
    pub effect: Option<EvaluatorEffect>,
}

impl Expression {
    pub fn new(
        syntax_location: syntax::Location,
        kind: ExpressionKind,
        source_map: &SourceMap,
        process: &Process,
    ) -> Self {
        let location = source_map.syntax_to_runtime(&syntax_location);

        let has_durable_breakpoint = if let Some(location) = &location {
            process.breakpoints().durable_at(location)
        } else {
            false
        };

        let is_comment = matches!(kind, ExpressionKind::Comment { .. });

        let effect =
            process.state().first_unhandled_effect().and_then(|effect| {
                let effect_location = source_map.runtime_to_syntax(
                    &process.state().most_recent_step().unwrap(),
                );

                if effect_location == syntax_location {
                    Some(effect.clone())
                } else {
                    None
                }
            });

        let is_on_call_stack = if let Some(location) = &location {
            process.stack().is_next_instruction_in_any_frame(location)
        } else {
            false
        };

        Self {
            kind,
            location,
            has_durable_breakpoint,
            is_comment,
            is_on_call_stack,
            effect,
        }
    }
}
