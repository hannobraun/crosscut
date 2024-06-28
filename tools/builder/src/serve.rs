use tokio::process::{Child, Command};

use crate::build::UpdatesRx;

pub async fn start(mut updates: UpdatesRx) -> anyhow::Result<()> {
    let address = "localhost:34480";

    let mut server: Option<Child> = None;

    updates.mark_unchanged(); // make sure we enter the loop body immediately
    while let Ok(()) = updates.changed().await {
        let Some(serve_dir) = &*updates.borrow() else {
            continue;
        };

        if let Some(mut server) = server.take() {
            server.kill().await?;
        }

        server = Some(
            Command::new("cargo")
                .arg("run")
                .args(["--package", "capi-server"])
                .arg("--")
                .args(["--address", address])
                .args(["--serve-dir", &serve_dir.display().to_string()])
                .spawn()?,
        );

        println!();
        println!("✅ Build is ready:");
        println!();
        println!("\t🚀 http://{address}/");
        println!();
        println!("------------------------------------------------");
        println!();
    }

    Ok(())
}
