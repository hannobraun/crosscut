use std::{
    io,
    path::{Path, PathBuf},
    str,
    time::SystemTime,
};

use capi_compiler::Compiler;
use capi_game_engine::host::GameEngineHost;
use capi_protocol::Versioned;
use capi_watch::DebouncedChanges;
use snafu::{whatever, Snafu, Whatever};
use tokio::{fs, sync::mpsc, task};

pub use capi_compiler::CompilerOutput;

pub async fn build_game_once(
    games_path: impl AsRef<Path>,
    game: &str,
) -> Result<CompilerOutput, BuildGameOnceError> {
    let mut compiler = Compiler::default();
    let output =
        build_game_once_with_compiler(games_path, game, &mut compiler).await?;
    Ok(output)
}

pub fn build_and_watch_game(
    games_path: PathBuf,
    game: impl Into<String>,
    changes: DebouncedChanges,
) -> EventsRx {
    let game = game.into();

    let (events_tx, events_rx) = mpsc::channel(1);

    task::spawn(async move {
        if let Err(err) =
            build_and_watch_game_inner(&games_path, game, changes, events_tx)
                .await
        {
            tracing::error!("Error building and watching game: {err}");

            // This task's channel sender has been dropped, which will cause the
            // shutdown to propagate through the rest of the system.
        }
    });

    events_rx
}

pub type EventsRx = mpsc::Receiver<Event>;

pub enum Event {
    ChangeDetected,
    BuildFinished(Versioned<CompilerOutput>),
}

async fn build_and_watch_game_inner(
    games_path: impl AsRef<Path>,
    game: String,
    mut changes: DebouncedChanges,
    events: mpsc::Sender<Event>,
) -> Result<(), Whatever> {
    let games_path = games_path.as_ref();

    let mut compiler = Compiler::default();
    let mut timestamp = Timestamp(0);

    let mut ignored_error = None;

    loop {
        if events.send(Event::ChangeDetected).await.is_err() {
            // Receiver dropped. We must be in the process of shutting down.
            return Ok(());
        }

        let code = match build_game_once_with_compiler(
            games_path,
            &game,
            &mut compiler,
        )
        .await
        {
            Ok(code) => code,
            Err(err) => match err.source.kind() {
                io::ErrorKind::NotFound => {
                    // Depending on the editor, this can happen while the file
                    // is being saved.
                    if let Some(old_err) = ignored_error {
                        whatever!(
                            "{err}\n\
                            \n\
                            Previously ignored an error, because a false \
                            positive was suspected: {old_err}"
                        );
                    } else {
                        ignored_error = Some(err);
                        continue;
                    }
                }
                _ => {
                    whatever!("{err}");
                }
            },
        };

        ignored_error = None;

        timestamp.update();

        let code = Versioned {
            timestamp: timestamp.0,
            inner: code,
        };
        if events.send(Event::BuildFinished(code)).await.is_err() {
            // Receiver dropped. We must be in the process of shutting down.
            return Ok(());
        }

        if changes.wait_for_change().await {
            continue;
        } else {
            break;
        }
    }

    Ok(())
}

async fn build_game_once_with_compiler(
    games_path: impl AsRef<Path>,
    game: &str,
    compiler: &mut Compiler,
) -> Result<CompilerOutput, BuildGameOnceError> {
    let path = games_path.as_ref().join(format!("{game}/{game}.capi"));
    let source = fs::read_to_string(&path)
        .await
        .map_err(|source| BuildGameOnceError { source, path })?;
    let output =
        compiler.compile::<GameEngineHost>(&source, &GameEngineHost::default());

    Ok(output)
}

#[derive(Debug, Snafu)]
#[snafu(display("Error while building `{}`: {source}", path.display()))]
pub struct BuildGameOnceError {
    pub source: io::Error,
    pub path: PathBuf,
}

struct Timestamp(u64);

impl Timestamp {
    fn update(&mut self) {
        let timestamp = SystemTime::UNIX_EPOCH
            .elapsed()
            .expect(
                "Current system time should never be later than Unix epoch.",
            )
            .as_nanos()
            .try_into()
            .expect(
                "`u64` should be able to represent nanosecond timestamps \
                until the year 2554.",
            );

        let timestamp = if timestamp > self.0 {
            timestamp
        } else {
            // Due to various factors, the new timestamp isn't necessarily
            // larger than the previous one. We need it to be though, otherwise
            // we can't use it to distinguish new builds from previous ones.
            self.0 + 1
        };

        self.0 = timestamp;
    }
}
