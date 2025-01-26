use std::{env, path::Path};

use anyhow::Context;
use notify::{Event, EventKind, RecursiveMode, Watcher as _};
use tokio::sync::watch;
use tracing::error;

use super::debounce::DebouncedChanges;

pub struct Watcher {
    // This field is not used, but we need to keep it around. If we drop the
    // `notify` watcher, it stops watching.
    _watcher: notify::RecommendedWatcher,
    pub changes: DebouncedChanges,
}

impl Watcher {
    pub fn new(path: &Path) -> anyhow::Result<Self> {
        let (tx, rx) = watch::channel(());

        let mut watcher = notify::recommended_watcher(move |event| {
            match event {
                Ok(Event {
                    kind: EventKind::Access(_),
                    ..
                }) => {
                    // We're not interested in read access to any files.
                    return;
                }
                Err(err) => {
                    error!("Error watching for changes: {err}");
                    return;
                }
                _ => {
                    // This is the kind of event we want to watch. Proceed.
                }
            }

            if tx.send(()).is_err() {
                // The other end has hung up. Not much we can do about that. The
                // thread this is running on will probably also end soon.
            }
        })?;
        watcher
            .watch(path, RecursiveMode::Recursive)
            .with_context(|| match path.canonicalize() {
                Ok(path) => {
                    format!("Watching `{}`", path.display())
                }
                Err(err) => {
                    let current_dir = match env::current_dir() {
                        Ok(path) => path.display().to_string(),
                        Err(err) => format!(
                            "unknown directory (failed to acquire: {})",
                            err
                        ),
                    };
                    format!(
                        "Watching `{}` in `{current_dir}` (failed to \
                        canonicalize path: {err})",
                        path.display()
                    )
                }
            })?;

        let changes = DebouncedChanges::new(rx);

        Ok(Self {
            _watcher: watcher,
            changes,
        })
    }
}
