use crosscut_compiler::CompilerOutput;
use crosscut_protocol::{command::CommandExt, ron_options, Versioned};
use gloo_net::http::{Request, Response};

use crate::{commands::CommandsToRuntimeTx, model::PersistentState};

pub struct CodeFetcher {
    pub timestamp: u64,
}

impl CodeFetcher {
    pub async fn new(
        commands_to_runtime_tx: &CommandsToRuntimeTx,
        state: &mut PersistentState,
    ) -> anyhow::Result<Self> {
        let code = Request::get("/code").send().await;
        let timestamp =
            on_new_code(code, commands_to_runtime_tx, state).await?;

        Ok(Self { timestamp })
    }

    pub async fn wait_for_new_code(
        &mut self,
        commands_to_runtime_tx: &CommandsToRuntimeTx,
        state: &mut PersistentState,
    ) -> anyhow::Result<()> {
        let code = Request::get(&format!("/code/{}", self.timestamp))
            .send()
            .await;

        self.timestamp =
            on_new_code(code, commands_to_runtime_tx, state).await?;

        Ok(())
    }
}

async fn on_new_code(
    code: Result<Response, gloo_net::Error>,
    commands_to_runtime_tx: &CommandsToRuntimeTx,
    state: &mut PersistentState,
) -> anyhow::Result<u64> {
    let code = code?.text().await?;
    let code: Versioned<CompilerOutput> = ron_options().from_str(&code)?;

    let command = state.on_new_code(code.inner);
    commands_to_runtime_tx.send(command.serialize()).expect(
        "Command receiver lives in static variable, should never drop.",
    );

    Ok(code.timestamp)
}
