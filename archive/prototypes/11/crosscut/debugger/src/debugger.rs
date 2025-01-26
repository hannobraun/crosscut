use crosscut_protocol::{
    command::CommandExt,
    updates::{SerializedUpdate, UpdateFromHost},
};
use leptos::reactive::prelude::Set;
use tokio::{select, sync::mpsc};

use crate::{
    code::CodeFetcher,
    commands::{CommandsToRuntimeRx, CommandsToRuntimeTx},
    model::{PersistentState, TransientState, UserAction},
    ui,
};

pub struct Debugger {
    pub updates_from_runtime_tx: mpsc::UnboundedSender<SerializedUpdate>,
    pub commands_to_runtime_rx: CommandsToRuntimeRx,
}

impl Debugger {
    pub fn new() -> Self {
        let (updates_from_runtime_tx, mut updates_from_runtime_rx) =
            mpsc::unbounded_channel();
        let (commands_to_runtime_tx, commands_to_runtime_rx) =
            mpsc::unbounded_channel();
        let (actions_tx, mut actions_rx) = mpsc::unbounded_channel();

        let mut persistent = PersistentState::default();
        let mut transient = persistent.generate_transient_state();

        let (state_read, state_write) =
            leptos::prelude::signal((persistent.clone(), transient.clone()));

        leptos::task::Executor::init_wasm_bindgen().unwrap();

        leptos::task::spawn_local(async move {
            let mut code =
                CodeFetcher::new(&commands_to_runtime_tx, &mut persistent)
                    .await
                    .unwrap();

            loop {
                select! {
                    result =
                        code.wait_for_new_code(
                            &commands_to_runtime_tx,
                            &mut persistent,
                        )
                    => {
                        result.unwrap();

                        // Nothing else to do, except do the update that happens
                        // below this `select!`.
                    }
                    update = updates_from_runtime_rx.recv() => {
                        let Some(update) = update else {
                            // This means the other end has hung up. Nothing we
                            // can do, except end this task too.
                            break;
                        };

                        on_update_from_runtime(
                            update,
                            &mut persistent,
                        );
                    }
                    action = actions_rx.recv() => {
                        let Some(action) = action else {
                            // This means the other end has hung up. Nothing we
                            // can do, except end this task too.
                            break;
                        };

                        on_ui_action(
                            action,
                            &mut persistent,
                            &transient,
                            &commands_to_runtime_tx,
                        );
                    }
                }

                transient = persistent.generate_transient_state();
                state_write.set((persistent.clone(), transient.clone()));
            }
        });

        ui::init(state_read, actions_tx);

        Self {
            updates_from_runtime_tx,
            commands_to_runtime_rx,
        }
    }
}

impl Default for Debugger {
    fn default() -> Self {
        Self::new()
    }
}

fn on_update_from_runtime(update: Vec<u8>, state: &mut PersistentState) {
    let update = UpdateFromHost::deserialize(update);
    state.on_update_from_host(update);
}

fn on_ui_action(
    action: UserAction,
    persistent: &mut PersistentState,
    transient: &TransientState,
    commands_to_runtime_tx: &CommandsToRuntimeTx,
) {
    let commands = persistent.on_user_action(action, transient).expect(
        "Failed to handle UI action. This is most likely a bug in the \
        Crosscut debugger:",
    );

    for command in commands {
        commands_to_runtime_tx.send(command.serialize()).unwrap();
    }
}
