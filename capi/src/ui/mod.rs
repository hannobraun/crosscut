mod components;
mod events;
mod start;

pub use self::{
    events::{send_event, EventsTx},
    start::start,
};

pub async fn handle_updates(
    mut updates: crate::updates::UpdatesRx,
    set_process: leptos::WriteSignal<Option<crate::process::Process>>,
) {
    use leptos::SignalSet;

    loop {
        let process = match updates.changed().await {
            Ok(()) => updates.borrow_and_update().clone(),
            Err(err) => panic!("{err}"),
        };

        set_process.set(Some(process));
    }
}
