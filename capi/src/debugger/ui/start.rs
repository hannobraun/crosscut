use leptos::create_signal;

use crate::{debugger::ui::components::debugger::Debugger, updates::UpdatesRx};

use super::{handle_updates, EventsTx};

pub fn start(updates_rx: UpdatesRx, events_tx: EventsTx) {
    let (debugger, set_debugger) = create_signal(None);

    leptos::mount_to_body(move || {
        leptos::view! {
            <Debugger process=debugger events=events_tx />
        }
    });

    leptos::spawn_local(handle_updates(updates_rx, set_debugger));
}
