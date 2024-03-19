use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use notify::RecommendedWatcher;
use notify_debouncer_mini::{
    DebounceEventResult, DebouncedEventKind, Debouncer,
};
use tokio::{
    fs,
    sync::watch::{self, Receiver},
    task,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let serve_dir = tempfile::tempdir()?;
    let path = serve_dir.path().to_owned();

    let (_watcher, events) = watch()?;

    tokio::select! {
        result = task::spawn(build(events, path.clone())) => {
            result??;
        }
        result = serve(path) => {
            result?;
        }
    }

    Ok(())
}

fn watch() -> anyhow::Result<(Debouncer<RecommendedWatcher>, Receiver<()>)> {
    let (tx, mut rx) = watch::channel(());
    rx.mark_changed();

    let mut debouncer = notify_debouncer_mini::new_debouncer(
        Duration::from_millis(50),
        move |result: DebounceEventResult| {
            let events = result.expect("Error watching for changes");

            for event in events {
                if let DebouncedEventKind::Any = event.kind {
                    // Should only panic, if the other end panicked, causing the
                    // receiver to drop. Nothing we can do about it.
                    let _ = tx.send(());
                }
            }
        },
    )?;
    debouncer
        .watcher()
        .watch(Path::new("index.html"), notify::RecursiveMode::NonRecursive)?;

    Ok((debouncer, rx))
}

async fn build(
    mut events: Receiver<()>,
    serve_dir: PathBuf,
) -> anyhow::Result<()> {
    while let Ok(()) = events.changed().await {
        println!("Change detected. Building...");

        fs::copy("index.html", serve_dir.join("index.html")).await?;
    }

    Ok(())
}

async fn serve(serve_dir: PathBuf) -> anyhow::Result<()> {
    use warp::Filter;

    warp::serve(
        warp::get().and(
            warp::path("update")
                .map(|| "Hello, world!")
                .or(warp::fs::dir(serve_dir)),
        ),
    )
    .run(([127, 0, 0, 1], 8080))
    .await;

    Ok(())
}
