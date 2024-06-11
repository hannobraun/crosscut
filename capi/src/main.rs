use ffi::STATE;

mod breakpoints;
mod code;
mod compiler;
mod debugger;
mod display;
mod effects;
mod ffi;
mod games;
mod program;
mod runner;
mod runtime;
mod source_map;
mod state;
mod syntax;
mod ui;
mod updates;

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Error)
        .expect("Failed to initialize logging to console");

    leptos::spawn_local(main_async());
}

async fn main_async() {
    let (_, _, updates_rx, events_tx, runner) = {
        let mut state = STATE.inner.lock().unwrap();
        let state = state.get_or_insert_with(Default::default);

        let program = state.game.program.clone();
        let updates_tx = state.updates.tx.clone();
        let updates_rx = state.updates.rx.clone();
        let events_tx = state.runner.events_tx.clone();
        let runner = state.runner.handle.take().unwrap();

        (program, updates_tx, updates_rx, events_tx, runner)
    };

    let set_program = ui::start(events_tx);
    leptos::spawn_local(handle_updates(updates_rx, set_program));

    crate::display::run(runner).await.unwrap();

    log::info!("Caterpillar initialized.");
}

async fn handle_updates(
    mut updates: crate::updates::UpdatesRx,
    set_program: leptos::WriteSignal<Option<crate::program::Program>>,
) {
    use leptos::SignalSet;

    loop {
        let program = match updates.changed().await {
            Ok(()) => updates.borrow_and_update().clone(),
            Err(err) => panic!("{err}"),
        };

        set_program.set(Some(program));
    }
}
