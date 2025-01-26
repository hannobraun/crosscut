use std::{net::SocketAddr, path::PathBuf};

use anyhow::Context;
use crosscut_watch::Watcher;
use tokio::{sync::mpsc, task};
use tracing::error;

use crate::build_game::{self, build_and_watch_game};

use super::server::{self, CodeTx};

pub enum Event {
    ChangeDetected,
    BuildFinished,
    ServerReady,
}

type EventsTx = mpsc::Sender<Event>;
pub type EventsRx = mpsc::Receiver<Event>;

pub async fn start(
    games_path: PathBuf,
    address: SocketAddr,
) -> anyhow::Result<EventsRx> {
    let (events_tx, events_rx) = mpsc::channel(1);

    task::spawn(async move {
        if let Err(err) = start_inner(games_path, address, events_tx).await {
            error!("Error while running server: {err:?}");

            // This tasks sender has already been dropped, which will cause the
            // shutdown to propagate to other tasks.
        }
    });

    Ok(events_rx)
}

async fn start_inner(
    games_path: PathBuf,
    address: SocketAddr,
    events: EventsTx,
) -> anyhow::Result<()> {
    let watcher =
        Watcher::new(&games_path).context("Creating watcher for game")?;
    let mut build_events =
        build_and_watch_game(games_path.join("snake"), watcher.changes);

    let mut server_task = ServerTask::Uninitialized { address };

    while let Some(event) = build_events.recv().await {
        match event {
            build_game::Event::ChangeDetected => {
                events.send(Event::ChangeDetected).await?;
            }
            build_game::Event::BuildFinished(code) => {
                events.send(Event::BuildFinished).await?;

                match server_task {
                    ServerTask::Uninitialized { address } => {
                        let (ready_rx, code_tx) = server::start(address, code);

                        ready_rx.await?;
                        events.send(Event::ServerReady).await?;

                        server_task = ServerTask::Initialized { code_tx };
                    }
                    ServerTask::Initialized { code_tx } => {
                        if code_tx.send(code).is_err() {
                            return Ok(());
                        }
                        server_task = ServerTask::Initialized { code_tx };
                    }
                }
            }
        }
    }

    Ok(())
}

enum ServerTask {
    Uninitialized { address: SocketAddr },
    Initialized { code_tx: CodeTx },
}
