use leptos::prelude::ReadSignal;

use crate::{
    model::{PersistentState, TransientState},
    ui::components::debugger::Debugger,
};

use super::ActionsTx;

pub fn init(
    state: ReadSignal<(PersistentState, TransientState)>,
    actions: ActionsTx,
) {
    leptos::mount::mount_to_body(move || {
        leptos::view! {
            <Debugger
                state=state
                actions=actions />
        }
    });
}
