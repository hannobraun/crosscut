use std::collections::VecDeque;

use rand::random;
use tokio::sync::mpsc::{self, error::TryRecvError};

use crate::{
    breakpoints::Breakpoints,
    compiler::compile,
    debugger::{
        model::DebugEvent,
        ui::{self, EventsRx},
    },
    display::Display,
    ffi,
    games::snake::snake,
    process::Process,
    runtime::{BuiltinEffect, Code, EvaluatorEffect, Value},
    tiles::{NUM_TILES, TILES_PER_AXIS},
    updates::{updates, Update, UpdatesTx},
};

pub struct RuntimeState {
    pub code: Code,
    pub process: Process,
    pub breakpoints: Breakpoints,
    pub memory: Memory,
    pub input: Input,
    pub tiles: [u8; NUM_TILES],
    pub display: Option<Display>,
    pub events_rx: EventsRx,
    pub updates_tx: UpdatesTx,
}

impl RuntimeState {
    pub fn new() -> Self {
        let mut script = crate::syntax::Script::default();
        snake(&mut script);

        let (code, source_map) = compile(&script);

        let entry = code.functions.get("main").cloned().unwrap();
        let process = Process::new(entry, vec![Value(TILES_PER_AXIS as i8); 2]);

        let memory = Memory::default();

        let input = Input::default();
        let (mut updates_tx, updates_rx) = updates();
        let (events_tx, events_rx) = mpsc::unbounded_channel();

        updates_tx.queue(Update::SourceCode {
            functions: script.functions,
            source_map,
        });
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
            code,
            process,
            breakpoints: Breakpoints::default(),
            memory,
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
                    if let DebugEvent::Reset = event {
                        self.memory.zero();
                    }

                    match event {
                        DebugEvent::BreakpointClear { location } => {
                            self.breakpoints.clear_durable(location);
                        }
                        DebugEvent::BreakpointSet { location } => {
                            self.breakpoints.set_durable(location);
                        }
                        DebugEvent::Continue { and_stop_at } => {
                            if let Some(EvaluatorEffect::Builtin(
                                BuiltinEffect::Breakpoint,
                            )) =
                                self.process.state().first_unhandled_effect()
                            {
                                if let Some(instruction) = and_stop_at {
                                    self.breakpoints.set_ephemeral(instruction);
                                }

                                self.process.handle_first_effect();
                            }
                        }
                        DebugEvent::Reset => {
                            self.process.reset();
                        }
                        DebugEvent::Step => {
                            if let Some(EvaluatorEffect::Builtin(
                                BuiltinEffect::Breakpoint,
                            )) =
                                self.process.state().first_unhandled_effect()
                            {
                                self.breakpoints.set_ephemeral(
                                    self.process
                                        .stack()
                                        .state()
                                        .next_instruction_overall()
                                        .unwrap(),
                                );
                                self.process.handle_first_effect();
                            }
                        }
                        DebugEvent::Stop => {
                            self.breakpoints.set_ephemeral(
                                self.process
                                    .stack()
                                    .state()
                                    .next_instruction_overall()
                                    .unwrap(),
                            );
                        }
                    }
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

        while self.process.state().can_step() {
            self.process.step(&self.code, &mut self.breakpoints);

            if let Some(EvaluatorEffect::Builtin(effect)) =
                self.process.state().first_unhandled_effect()
            {
                match effect {
                    BuiltinEffect::Breakpoint => {
                        // Nothing to do here. With an unhandled effect, the
                        // program won't continue running. The debugger is in
                        // control of what happens next.
                    }
                    BuiltinEffect::Error(_) => {
                        // Nothing needs to be done. With an unhandled
                        // effect, the program won't continue running, and
                        // the debugger will see the error and display it.
                    }
                    BuiltinEffect::Load { address } => {
                        let address: usize = (*address).into();
                        let value = self.memory.inner[address];
                        self.process.push([value]);

                        self.process.handle_first_effect();
                    }
                    BuiltinEffect::Store { address, value } => {
                        let address: usize = (*address).into();
                        self.memory.inner[address] = *value;

                        self.process.handle_first_effect();

                        self.updates_tx.queue(Update::Memory {
                            memory: self.memory.clone(),
                        });
                    }
                    BuiltinEffect::SetTile { x, y, value } => {
                        let x = *x;
                        let y = *y;
                        let value = *value;

                        self.process.handle_first_effect();

                        display.set_tile(
                            x.into(),
                            y.into(),
                            value,
                            &mut self.tiles,
                        );
                    }
                    BuiltinEffect::SubmitFrame => {
                        // This effect means that the game is done rendering.
                        // Let's break out of this loop now, so we can do our
                        // part in that and return control to the host.
                        self.process.handle_first_effect();
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
                        self.process.handle_first_effect();
                    }
                    BuiltinEffect::ReadRandom => {
                        self.process.push([Value(random())]);
                        self.process.handle_first_effect();
                    }
                }
            }
        }

        self.updates_tx
            .handle_events(&mut self.breakpoints, &mut self.process);
        self.updates_tx.queue(Update::Process(self.process.clone()));

        display.render(&self.tiles);
    }
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Memory {
    pub inner: [Value; 256],
}

impl Memory {
    pub fn zero(&mut self) {
        *self = Self::default();
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            inner: [Value(0); 256],
        }
    }
}

#[derive(Default)]
pub struct Input {
    pub buffer: VecDeque<u8>,
}
