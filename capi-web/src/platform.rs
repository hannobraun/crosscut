use std::time::Duration;

use async_channel::{Receiver, RecvError, Sender, TryRecvError};
use capi_core::{
    value, DataStackResult, Interpreter, PlatformFunction,
    PlatformFunctionState, RuntimeContext, RuntimeState,
};
use chrono::Local;
use futures::executor::block_on;
use gloo_timers::future::sleep;
use tracing::debug;

pub async fn run(
    script: &str,
    code: Receiver<String>,
    events: Sender<Event>,
) -> anyhow::Result<()> {
    debug!("Running script:\n{script}");

    let mut interpreter = Interpreter::new()?;

    let parent = None;
    interpreter.update(script, parent)?;

    let mut context = Context {
        events: Events { inner: events },
        sleep_duration: None,
    };

    interpreter.register_platform([
        (delay_ms as PlatformFunction<Context>, "delay_ms"),
        (print, "print"),
    ]);

    let mut new_code: Option<String> = None;

    loop {
        if let Some(code) = new_code.take() {
            let parent = None;
            if let Err(err) = interpreter.update(&code, parent) {
                context.events.status(format!("Pipeline error:\n{err:?}\n"));
            }
        }

        let sleep_duration = match interpreter.step(&mut context) {
            Ok(RuntimeState::Running) => None,
            Ok(RuntimeState::Sleeping) => context.sleep_duration.take(),
            Ok(RuntimeState::Finished) => {
                context.events.status(
                    "Program finished (will restart on change to script)\n",
                );

                match code.recv().await {
                    Ok(code) => new_code = Some(code),
                    Err(RecvError) => {
                        // The channel was closed. However this happened, it
                        // means our work here is done.
                        break;
                    }
                }

                context.events.status("Change detected. Restarting.\n");

                continue;
            }
            Err(err) => {
                context.events.status(format!("Runtime error:\n{err:?}\n"));
                break;
            }
        };

        // Always sleep, even if it's for zero duration, to give the rest of the
        // website a chance to do its thing between steps.
        let sleep_duration = sleep_duration.unwrap_or(Duration::from_millis(0));
        sleep(sleep_duration).await;

        match code.try_recv() {
            Ok(code) => {
                new_code = Some(code);
            }
            Err(TryRecvError::Empty) => {
                // No problem that we don't have a code update. Just continue.
            }
            Err(TryRecvError::Closed) => {
                // The channel was closed. However this happened, it means our
                // work here is done.
                break;
            }
        }
    }

    Ok(())
}

pub struct Context {
    pub events: Events,
    pub sleep_duration: Option<Duration>,
}

pub struct Events {
    pub inner: Sender<Event>,
}

impl Events {
    pub fn output(&self, message: String) {
        block_on(self.inner.send(Event::Output(message))).unwrap()
    }

    pub fn status(&self, message: impl Into<String>) {
        let message = format!(
            "{}: {}",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            message.into()
        );
        block_on(self.inner.send(Event::Status(message))).unwrap()
    }
}

pub enum Event {
    Output(String),
    Status(String),
}

pub fn delay_ms(
    runtime_context: RuntimeContext,
    platform_context: &mut Context,
) -> DataStackResult<PlatformFunctionState> {
    let (delay_ms, _) =
        runtime_context.data_stack.pop_specific::<value::Number>()?;

    let delay_ms = delay_ms
        .0
        .try_into()
        .expect("Negative sleep duration is invalid");
    platform_context.sleep_duration = Some(Duration::from_millis(delay_ms));

    Ok(PlatformFunctionState::Sleeping)
}

pub fn print(
    runtime_context: RuntimeContext,
    platform_context: &mut Context,
) -> DataStackResult<PlatformFunctionState> {
    let value = runtime_context.data_stack.pop_any()?;
    platform_context
        .events
        .output(format!("{}\n", value.payload));
    runtime_context.data_stack.push(value);
    Ok(PlatformFunctionState::Done)
}
