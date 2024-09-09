use capi_process::Instructions;
use capi_protocol::{
    command::{CommandToRuntime, SerializedCommandToRuntime},
    updates::{Code, SerializedUpdate, UpdateFromRuntime},
    Versioned,
};
use gloo_net::http::{Request, Response};
use leptos::SignalSet;
use tokio::{
    select,
    sync::{
        mpsc::{self, UnboundedSender},
        watch,
    },
};

use crate::{
    debugger::{Debugger, RemoteProcess},
    ui::{self, Action},
};

pub struct DebuggerState {
    pub code_rx: watch::Receiver<Instructions>,
    pub updates_from_runtime_tx: mpsc::UnboundedSender<SerializedUpdate>,
    pub commands_to_runtime_rx:
        mpsc::UnboundedReceiver<SerializedCommandToRuntime>,
}

impl DebuggerState {
    pub fn new() -> Self {
        let (code_tx, code_rx) = watch::channel(Instructions::default());
        let (updates_from_runtime_tx, mut updates_from_runtime_rx) =
            mpsc::unbounded_channel();
        let (commands_to_runtime_tx, commands_to_runtime_rx) =
            mpsc::unbounded_channel();
        let (actions_tx, mut actions_rx) = mpsc::unbounded_channel();

        let mut debugger = Debugger::default();
        let mut remote_process = RemoteProcess::default();

        let (debugger_read, debugger_write) =
            leptos::create_signal(debugger.clone());

        leptos::spawn_local(async move {
            let code = Request::get("/code").send().await;
            let mut timestamp =
                on_new_code(code, &code_tx, &mut remote_process).await;

            loop {
                let response =
                    Request::get(&format!("/code/{timestamp}")).send();

                select! {
                    code = response => {
                        timestamp =
                            on_new_code(
                                code,
                                &code_tx,
                                &mut remote_process,
                            )
                            .await;
                    }
                    update = updates_from_runtime_rx.recv() => {
                        let Some(update) = update else {
                            // This means the other end has hung up. Nothing we
                            // can do, except end this task too.
                            break;
                        };

                        on_update_from_runtime(
                            update,
                            &mut remote_process,
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
                            &mut debugger,
                            &commands_to_runtime_tx,
                        );
                    }
                }

                debugger.update(
                    remote_process.code.clone(),
                    remote_process.memory.clone(),
                    remote_process.process.as_ref(),
                );
                debugger_write.set(debugger.clone());
            }
        });

        ui::init(debugger_read, actions_tx);

        Self {
            updates_from_runtime_tx,
            code_rx,
            commands_to_runtime_rx,
        }
    }
}

impl Default for DebuggerState {
    fn default() -> Self {
        Self::new()
    }
}

async fn on_new_code(
    code: Result<Response, gloo_net::Error>,
    code_tx: &watch::Sender<Instructions>,
    remote_process: &mut RemoteProcess,
) -> u64 {
    let code = code.unwrap().text().await.unwrap();
    let code: Versioned<Code> = ron::from_str(&code).unwrap();

    code_tx
        .send(code.inner.instructions.clone())
        .expect("Code receiver should never drop.");

    remote_process.on_code_update(code.inner);

    code.timestamp
}

fn on_update_from_runtime(update: Vec<u8>, remote_process: &mut RemoteProcess) {
    let update = UpdateFromRuntime::deserialize(update);
    remote_process.on_update_from_runtime(update);
}

fn on_ui_action(
    action: Action,
    debugger: &mut Debugger,
    commands_to_runtime_tx: &UnboundedSender<SerializedCommandToRuntime>,
) {
    let command = match action {
        Action::BreakpointClear { instruction } => {
            debugger.breakpoints.clear_durable(&instruction);
            CommandToRuntime::BreakpointClear { instruction }
        }
        Action::BreakpointSet { instruction } => {
            debugger.breakpoints.set_durable(instruction);
            CommandToRuntime::BreakpointSet { instruction }
        }
        Action::Continue => CommandToRuntime::Continue,
        Action::Reset => CommandToRuntime::Reset,
        Action::Step => CommandToRuntime::Step,
        Action::Stop => CommandToRuntime::Stop,
    };

    commands_to_runtime_tx.send(command.serialize()).unwrap();
}
