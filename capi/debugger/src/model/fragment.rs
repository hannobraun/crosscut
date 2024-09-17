use capi_compiler::{
    fragments::{Fragment, FragmentId, FragmentKind, Fragments, Payload},
    host::Host,
    source_map::SourceMap,
};
use capi_game_engine::host::GameEngineHost;
use capi_process::{Breakpoints, Effect};

use super::DebugFunction;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DebugFragment {
    pub data: DebugFragmentData,
    pub kind: DebugFragmentKind,
}

impl DebugFragment {
    pub fn new(
        fragment: Fragment,
        active_fragment: Option<FragmentId>,
        is_in_innermost_active_function: bool,
        fragments: &Fragments,
        source_map: &SourceMap,
        breakpoints: &Breakpoints,
        effects: &[Effect],
    ) -> Option<Self> {
        let state = if Some(fragment.id()) == active_fragment {
            if is_in_innermost_active_function {
                DebugFragmentState::InnermostActiveFragment
            } else {
                DebugFragmentState::ActiveCaller
            }
        } else {
            DebugFragmentState::NotActive
        };

        let has_durable_breakpoint = source_map
            .fragment_to_instructions(&fragment.id())
            .iter()
            .any(|instruction| breakpoints.durable_at(instruction));

        let effect = effects.first().and_then(|effect| {
            // This displays the effect on _any_ active fragment in _any_ active
            // function, not just the fragment that actually triggered the
            // effect.
            //
            // This makes sense, as the effect is relevant, regardless of which
            // active function you're looking at. The developer might step
            // through some functions, hit the effect, but might not want to
            // actually step into the function where the effect comes from.
            if state.is_active() {
                Some(*effect)
            } else {
                None
            }
        });

        let data = DebugFragmentData {
            id: fragment.id(),
            fragment: fragment.clone(),
            state,
            has_durable_breakpoint,
            effect,
        };
        let kind = DebugFragmentKind::new(
            fragment,
            active_fragment,
            is_in_innermost_active_function,
            fragments,
            source_map,
            breakpoints,
            effects,
        )?;

        Some(Self { kind, data })
    }

    pub fn id(&self) -> FragmentId {
        self.data.fragment.id()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DebugFragmentData {
    /// # The fragment ID
    pub id: FragmentId,

    /// # The fragment that the `DebugFragment` was built from
    pub fragment: Fragment,

    /// # The static of the fragment
    pub state: DebugFragmentState,

    pub has_durable_breakpoint: bool,
    pub effect: Option<Effect>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DebugFragmentState {
    InnermostActiveFragment,
    ActiveCaller,
    NotActive,
}

impl DebugFragmentState {
    /// # Indicate whether this is the innermost active fragment
    ///
    /// The innermost active fragment is the active fragment in the innermost
    /// active function. The fragment where the process is currently stopped at.
    pub fn is_innermost_active_fragment(&self) -> bool {
        matches!(self, Self::InnermostActiveFragment)
    }

    /// # Indicate whether the fragment is active
    ///
    /// A fragment is active, either if the process is currently stopped here,
    /// or if it calls an active function (which is a function that contains an
    /// active fragment).
    pub fn is_active(&self) -> bool {
        matches!(self, Self::InnermostActiveFragment | Self::ActiveCaller)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DebugFragmentKind {
    CallToFunction { name: String },
    CallToHostFunction { name: String },
    CallToIntrinsic { name: String },
    Comment { text: String },
    Function { function: DebugFunction },
    ResolvedBinding { name: String },
    UnresolvedIdentifier { name: String },
    Value { as_string: String },
}

impl DebugFragmentKind {
    pub fn new(
        fragment: Fragment,
        active_fragment: Option<FragmentId>,
        is_in_innermost_active_function: bool,
        fragments: &Fragments,
        source_map: &SourceMap,
        breakpoints: &Breakpoints,
        effects: &[Effect],
    ) -> Option<Self> {
        let FragmentKind::Payload { payload, .. } = fragment.kind else {
            return None;
        };

        let kind = match payload {
            Payload::CallToFunction { name, .. } => {
                Self::CallToFunction { name }
            }
            Payload::CallToHostFunction { effect_number } => {
                let name = GameEngineHost::effect_number_to_function_name(
                    effect_number,
                )
                .expect("Expected effect number in code to be valid.")
                .to_string();

                Self::CallToHostFunction { name }
            }
            Payload::CallToIntrinsic { intrinsic, .. } => {
                Self::CallToIntrinsic {
                    name: intrinsic.to_string(),
                }
            }
            Payload::Comment { text } => Self::Comment {
                text: format!("# {text}"),
            },
            Payload::Function { function } => {
                let function = DebugFunction::new(
                    function,
                    active_fragment,
                    is_in_innermost_active_function,
                    fragments,
                    source_map,
                    breakpoints,
                    effects,
                );

                Self::Function { function }
            }
            Payload::ResolvedBinding { name } => Self::ResolvedBinding { name },
            Payload::UnresolvedIdentifier { name } => {
                Self::UnresolvedIdentifier { name }
            }
            Payload::Value(value) => Self::Value {
                as_string: value.to_string(),
            },
        };

        Some(kind)
    }
}
