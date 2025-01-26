use tokio::process::Command;

use crate::build::build_once;

pub async fn run() -> anyhow::Result<()> {
    let optimize = true;
    let files = build_once(optimize).await?;

    if let Some(files) = files {
        Command::new("cargo")
            .arg("run")
            .args(["--package", "crosscut-cli"])
            .arg("--")
            .arg("export")
            .args(["--path", "export"])
            .env("FILES", files.path().display().to_string())
            .kill_on_drop(true)
            .status()
            .await?;
    }

    Ok(())
}
