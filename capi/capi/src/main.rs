mod capi;
mod display;
mod server;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("tower_http::trace=info")
        .init();

    let (program, functions) = capi::Program::new();

    let (events_tx, events_rx) = tokio::sync::mpsc::unbounded_channel();
    let (updates_tx, updates_rx) =
        tokio::sync::watch::channel(Default::default());

    std::thread::spawn(|| {
        use futures::StreamExt;
        let mut updates = tokio_stream::wrappers::WatchStream::new(updates_rx);
        while let Some(functions) = futures::executor::block_on(updates.next())
        {
            dbg!(functions);
        }
    });

    server::start(functions.clone(), events_tx);
    display::run(program, functions, events_rx, updates_tx)
}
