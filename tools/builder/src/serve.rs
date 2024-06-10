use std::process;

use axum::{extract::State, http::StatusCode, routing::get, Router};
use tokio::{net::TcpListener, sync::watch, task};
use tower_http::services::ServeDir;
use tracing::error;

pub async fn start(changes: watch::Receiver<()>) -> anyhow::Result<()> {
    let address = "localhost:34480";

    let router = Router::new()
        .route("/changes", get(serve_changes))
        .nest_service("/", ServeDir::new("capi/dist"))
        .with_state(changes);
    let listener = TcpListener::bind(address).await?;

    task::spawn(async {
        if let Err(err) = axum::serve(listener, router).await {
            error!("Error serving HTTP endpoints: {err}");
            process::exit(1);
        }
    });

    println!("Serving Caterpillar at http://{address}");

    Ok(())
}

async fn serve_changes(
    State(mut changes): State<watch::Receiver<()>>,
) -> StatusCode {
    changes.mark_unchanged();
    match changes.changed().await {
        Ok(()) => StatusCode::OK,
        Err(_) => {
            error!("Waiting for updates, but updates no longer available.");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
