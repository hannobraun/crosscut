use std::thread;

use capi_core::{Interpreter, PlatformFunction, RuntimeState};
use crossbeam_channel::{Receiver, RecvError, Sender, TryRecvError};

use crate::platform::{self, Context, PixelOp};

pub struct DesktopThread {
    pub pixel_ops: Receiver<PixelOp>,
    join_handle: thread::JoinHandle<anyhow::Result<()>>,
}

impl DesktopThread {
    pub fn run(
        code: String,
        updates: Receiver<String>,
    ) -> anyhow::Result<Self> {
        let (pixel_ops_tx, pixel_ops_rx) = crossbeam_channel::unbounded();
        let join_handle =
            thread::spawn(|| run_inner(code, updates, pixel_ops_tx));

        Ok(Self {
            pixel_ops: pixel_ops_rx,
            join_handle,
        })
    }

    pub fn join(self) -> anyhow::Result<()> {
        match self.join_handle.join() {
            Ok(result) => {
                // The result that the thread returned, which is possibly an
                // error.
                result?
            }
            Err(err) => {
                // The thread panicked! Let's make sure this bubbles up to the
                // user.
                std::panic::resume_unwind(err)
            }
        }

        Ok(())
    }
}

fn run_inner(
    code: String,
    updates: Receiver<String>,
    pixel_ops: Sender<PixelOp>,
) -> anyhow::Result<()> {
    let mut interpreter = Interpreter::new(&code)?;
    let mut context = Context {
        pixel_ops: platform::Sender { inner: pixel_ops },
    };

    interpreter.register_platform([
        (
            "clear_pixel",
            platform::clear_pixel as PlatformFunction<platform::Context>,
        ),
        ("delay_ms", platform::delay_ms),
        ("set_pixel", platform::set_pixel),
        ("print", platform::print),
    ]);

    loop {
        let runtime_state = interpreter.step(&mut context)?;

        let new_code = match runtime_state {
            RuntimeState::Running => match updates.try_recv() {
                Ok(new_code) => Some(new_code),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => break,
            },
            RuntimeState::Sleeping => {
                unreachable!(
                    "No desktop platform functions put runtime to sleep"
                )
            }
            RuntimeState::Finished => {
                eprintln!();
                eprintln!("> Program finished.");
                eprintln!("  > will restart on change to script");
                eprintln!("  > press CTRL-C to abort");
                eprintln!();

                match updates.recv() {
                    Ok(new_code) => Some(new_code),
                    Err(RecvError) => break,
                }
            }
        };

        if let Some(new_code) = new_code {
            interpreter.update(&new_code)?;
        }
    }

    Ok(())
}
