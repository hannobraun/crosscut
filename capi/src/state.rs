use std::collections::VecDeque;

use rand::random;
use tokio::sync::mpsc::{self, error::TryRecvError};

use crate::{
    debugger::DebugEvent,
    display::Display,
    ffi,
    games::{self, snake::snake},
    process::{Process, ProgramEffect, ProgramEffectKind},
    runtime::{BuiltinEffect, EvaluatorEffectKind, Value},
    tiles::NUM_TILES,
    ui,
    updates::{updates, UpdatesTx},
};

pub struct RuntimeState {
    pub process: Process,
    pub input: Input,
    pub tiles: [u8; NUM_TILES],
    pub display: Option<Display>,
    pub events_rx: mpsc::UnboundedReceiver<DebugEvent>,
    pub updates_tx: UpdatesTx,
}

impl RuntimeState {
    pub fn new() -> Self {
        let process = games::build(snake);

        let input = Input::default();
        let (updates_tx, updates_rx) = updates(&process);
        let (events_tx, events_rx) = mpsc::unbounded_channel();

        ui::start(updates_rx, events_tx);

        // While we're still using `pixels`, the `Display` constructor needs to
        // be async. We need to do some acrobatics here to deal with that.
        leptos::spawn_local(async {
            let display = Display::new().await.unwrap();

            let mut state = ffi::STATE.lock().unwrap();
            let state = state.get_or_insert_with(Default::default);

            state.display = Some(display);
        });

        Self {
            process,
            input,
            tiles: [0; NUM_TILES],
            display: None,
            events_rx,
            updates_tx,
        }
    }

    pub fn update(&mut self) {
        let Some(display) = self.display.as_mut() else {
            // Display not initialized yet.
            return;
        };

        loop {
            match self.events_rx.try_recv() {
                Ok(event) => {
                    self.process.process_event(event);
                }
                Err(TryRecvError::Empty) => {
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    // The other end has hung up, which happens during
                    // shutdown. Shut down this task, too.
                    return;
                }
            }
        }

        while self.process.can_step() {
            self.process.step();

            if let Some(ProgramEffect {
                kind:
                    ProgramEffectKind::Evaluator(EvaluatorEffectKind::Builtin(
                        effect,
                    )),
                ..
            }) = self.process.effects.front()
            {
                match effect {
                    BuiltinEffect::Error(_) => {
                        // Nothing needs to be done. With an unhandled
                        // effect, the program won't continue running, and
                        // the debugger will see the error and display it.
                    }
                    BuiltinEffect::Load { address } => {
                        let address: usize = (*address).into();
                        let value = self.process.memory.inner[address];
                        self.process.push([value]);

                        self.process.effects.pop_front();
                    }
                    BuiltinEffect::Store { address, value } => {
                        let address: usize = (*address).into();
                        self.process.memory.inner[address] = *value;

                        self.process.effects.pop_front();
                    }
                    BuiltinEffect::SetTile { x, y, value } => {
                        let x = *x;
                        let y = *y;
                        let value = *value;

                        self.process.effects.pop_front();

                        display.set_tile(
                            x.into(),
                            y.into(),
                            value,
                            &mut self.tiles,
                        );
                    }
                    BuiltinEffect::SubmitFrame => {
                        // This effect means that the game is done rendering. Let's
                        // break out of this loop now, so we can do our part in that
                        // and return control to the host.
                        self.process.effects.pop_front();
                        break;
                    }
                    BuiltinEffect::ReadInput => {
                        let input = self
                            .input
                            .buffer
                            .pop_front()
                            .unwrap_or(0)
                            .try_into()
                            .unwrap();

                        self.process.push([Value(input)]);
                        self.process.effects.pop_front();
                    }
                    BuiltinEffect::ReadRandom => {
                        self.process.push([Value(random())]);
                        self.process.effects.pop_front();
                    }
                }
            }
        }

        self.updates_tx.send_if_relevant_change(&self.process);

        display.render(&self.tiles);
    }
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default)]
pub struct Input {
    pub buffer: VecDeque<u8>,
}
