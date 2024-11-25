use std::{
    path::{Path, PathBuf},
    process,
};

use anyhow::Context;
use capi_watch::DebouncedChanges;
use tempfile::{tempdir, TempDir};
use tokio::{fs, process::Command, sync::mpsc, task};
use tracing::error;
use walkdir::WalkDir;
use wasm_bindgen_cli_support::Bindgen;

pub fn start(changes: DebouncedChanges) -> UpdatesRx {
    let (tx, rx) = mpsc::channel(1);
    task::spawn(async {
        if let Err(err) = watch_and_build(changes, tx).await {
            error!("Build error: {err:?}");
            process::exit(1);
        }
    });
    rx
}

async fn watch_and_build(
    mut changes: DebouncedChanges,
    updates: UpdatesTx,
) -> anyhow::Result<()> {
    println!();
    println!("⏳ Starting initial build of Caterpillar...");
    println!();

    // We're not really doing anything with this variable, but it needs to
    // exist. It keeps the `TempDir` instances from being dropped before we're
    // done with it. Dropping it prematurely would delete the temporary
    // directory we serve files out of.
    let mut _output_dir = build_once().await?;

    if let Some(output_dir) = &_output_dir {
        let output_path = output_dir.path().to_path_buf();
        if updates.send(output_path).await.is_err() {
            // If the send failed, the other end has hung up. That means either
            // we're currently shutting down, or something went wrong over there
            // and we _should_ be shutting down.
            return Ok(());
        }
    }

    while changes.wait_for_change().await {
        println!();
        println!("🔄 Change detected.");
        println!("⏳ Rebuilding Caterpillar...");
        println!();

        _output_dir = build_once().await?;

        if let Some(output_dir) = &_output_dir {
            let output_path = output_dir.path().to_path_buf();
            if updates.send(output_path).await.is_err() {
                // If the send failed, the other end has hung up. That means
                // either we're currently shutting down, or something went wrong
                // over there and we _should_ be shutting down.
                break;
            }
        }
    }

    Ok(())
}

async fn build_once() -> anyhow::Result<Option<TempDir>> {
    let packages = [("capi-host", Some("cdylib")), ("capi-debugger", None)];

    for (package, crate_type) in packages {
        let mut command = Command::new("cargo");

        command
            .arg("rustc")
            .args(["--package", package])
            .args(["--target", "wasm32-unknown-unknown"]);

        if let Some(crate_type) = crate_type {
            command.args(["--crate-type", crate_type]);
        }

        let exit_status = command.status().await?;
        if !exit_status.success() {
            // The build failed, and since the rest of this function is
            // dependent on its success, we're done here.
            //
            // But that doesn't mean that the builder overall should be done.
            // Next time we detect a change, we should try again.
            return Ok(None);
        }
    }

    let target = "target/wasm32-unknown-unknown/debug";
    let new_output_dir = tempdir()?;
    copy(target, new_output_dir.path(), "capi_host.wasm").await?;

    let wasm_module = format!("{target}/capi-debugger.wasm");

    let mut bindgen = Bindgen::new();
    bindgen
        .input_path(wasm_module)
        .web(true)?
        .generate(&new_output_dir)?;

    let www_dir = "capi/debugger/www";
    for entry in WalkDir::new(www_dir) {
        let entry = entry?;

        if entry.file_type().is_dir() {
            continue;
        }

        let relative_path = entry.path().strip_prefix(www_dir)?;
        copy(www_dir, new_output_dir.path(), relative_path).await?;
    }

    Ok(Some(new_output_dir))
}

async fn copy(
    source_dir: impl AsRef<Path>,
    target_dir: impl AsRef<Path>,
    file: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let file = file.as_ref();
    let source_dir = source_dir.as_ref();
    let target_dir = target_dir.as_ref();

    fs::copy(source_dir.join(file), target_dir.join(file))
        .await
        .with_context(|| {
            format!(
                "Copying file `{}` from `{}` to `{}`",
                file.display(),
                source_dir.display(),
                target_dir.display()
            )
        })?;

    Ok(())
}

pub type UpdatesRx = mpsc::Receiver<Update>;
pub type UpdatesTx = mpsc::Sender<Update>;

pub type Update = PathBuf;
