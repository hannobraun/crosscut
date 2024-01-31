use std::{path::PathBuf, thread};

use capi_core::{Interpreter, RuntimeState};
use crossbeam_channel::{Receiver, RecvError, Sender, TryRecvError};

use crate::{
    loader::Loader,
    platform::{DesktopPlatform, PixelOp, PlatformContext},
};

pub struct DesktopThread {
    pub pixel_ops: Receiver<PixelOp>,
    lifeline: Sender<()>,
    join_handle: JoinHandle,
}

impl DesktopThread {
    pub fn run(entry_script_path: PathBuf) -> anyhow::Result<Self> {
        struct RunProgram;

        impl RunTarget for RunProgram {
            fn finish(
                &self,
                _: &mut Interpreter<DesktopPlatform>,
                _: &mut PlatformContext,
            ) -> anyhow::Result<()> {
                eprintln!();
                eprintln!("> Program finished.");
                eprintln!("  > will restart on change to script");
                eprintln!("  > press CTRL-C to abort");
                eprintln!();

                Ok(())
            }
        }

        Self::new(entry_script_path, RunProgram)
    }

    pub fn test(entry_script_path: PathBuf) -> anyhow::Result<Self> {
        struct RunTests;

        impl RunTarget for RunTests {
            fn finish(
                &self,
                interpreter: &mut Interpreter<DesktopPlatform>,
                platform_context: &mut PlatformContext,
            ) -> anyhow::Result<()> {
                interpreter.run_tests(platform_context)?;

                eprintln!();
                eprintln!("> Test run finished.");
                eprintln!("  > will re-run on change to script");
                eprintln!("  > press CTRL-C to abort");
                eprintln!();

                Ok(())
            }
        }

        Self::new(entry_script_path, RunTests)
    }

    fn new(
        entry_script_path: PathBuf,
        run_target: impl RunTarget,
    ) -> anyhow::Result<Self> {
        let (pixel_ops_tx, pixel_ops_rx) = crossbeam_channel::unbounded();
        let (lifeline_tx, lifeline_rx) = crossbeam_channel::bounded(0);

        let join_handle = thread::spawn(|| {
            Self::run_inner(
                entry_script_path,
                lifeline_rx,
                pixel_ops_tx,
                run_target,
            )
        });

        Ok(Self {
            pixel_ops: pixel_ops_rx,
            lifeline: lifeline_tx,
            join_handle,
        })
    }

    fn run_inner(
        entry_script_path: PathBuf,
        lifeline: Receiver<()>,
        pixel_ops: Sender<PixelOp>,
        run_target: impl RunTarget,
    ) -> anyhow::Result<()> {
        let mut loader = Loader::new(&entry_script_path)?;

        let parent = None;
        loader.load(&entry_script_path, parent)?;

        let mut interpreter = Interpreter::new()?;
        let mut platform_context =
            PlatformContext::new().with_pixel_ops_sender(pixel_ops);

        loop {
            if let Err(TryRecvError::Disconnected) = lifeline.try_recv() {
                // If the other end of the lifeline got dropped, that means
                // we're supposed to stop.
                break;
            }

            match loader.updates().try_recv() {
                Ok(update) => {
                    // This shouldn't block for too long, as we know at this
                    // point that there has been a change, via the legacy update
                    // mechanism.
                    let scripts = loader.wait_for_updated_scripts()?;

                    let (_path, parent, new_code) = update?;
                    interpreter.update(&new_code, parent, scripts)?;
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => break,
            }

            let runtime_state = interpreter.step(&mut platform_context)?;

            match runtime_state {
                RuntimeState::Running => {}
                RuntimeState::Sleeping => {
                    // None of the desktop platform functions put the runtime to
                    // sleep.
                    unreachable!()
                }
                RuntimeState::Finished => {
                    run_target
                        .finish(&mut interpreter, &mut platform_context)?;

                    match loader.updates().recv() {
                        Ok(update) => {
                            // This shouldn't block for too long, as we know at
                            // this point that there has been a change, via the
                            // legacy update mechanism.
                            let scripts = loader.wait_for_updated_scripts()?;

                            let (_path, parent, new_code) = update?;
                            interpreter.update(&new_code, parent, scripts)?;
                        }
                        Err(RecvError) => break,
                    }
                }
            };
        }

        Ok(())
    }

    pub fn join(self) -> anyhow::Result<()> {
        Self::join_inner(self.join_handle)
    }

    pub fn quit(self) -> anyhow::Result<()> {
        // This will signal the thread that it should stop.
        drop(self.lifeline);

        Self::join_inner(self.join_handle)
    }

    fn join_inner(join_handle: JoinHandle) -> anyhow::Result<()> {
        match join_handle.join() {
            Ok(result) => {
                // The result that the thread returned, which is possibly an
                // error.
                result
            }
            Err(err) => {
                // The thread panicked! Let's make sure this bubbles up to the
                // caller.
                std::panic::resume_unwind(err)
            }
        }
    }
}

type JoinHandle = thread::JoinHandle<anyhow::Result<()>>;

trait RunTarget: Send + 'static {
    fn finish(
        &self,
        interpreter: &mut Interpreter<DesktopPlatform>,
        platform_context: &mut PlatformContext,
    ) -> anyhow::Result<()>;
}
