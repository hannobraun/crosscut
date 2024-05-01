use std::time::{Duration, Instant};

use capi_runtime::{Program, ProgramState, Value};
use pixels::{Pixels, SurfaceTexture};
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use crate::{
    server::EventsRx,
    tiles::{PIXELS_PER_TILE_AXIS, TILES_OFFSET_IN_MEMORY, TILES_PER_AXIS},
    updates::UpdatesTx,
};

pub fn run(
    program: Program,
    events: EventsRx,
    updates: UpdatesTx,
) -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;

    let mut state = State {
        program,
        events,
        updates,
        mem: [0; MEM_SIZE],
        window: None,
        pixels: None,
    };

    event_loop.run_app(&mut state)?;

    Ok(())
}

const PIXELS_PER_AXIS: usize = TILES_PER_AXIS * PIXELS_PER_TILE_AXIS;
const MEM_SIZE: usize =
    TILES_OFFSET_IN_MEMORY + TILES_PER_AXIS * TILES_PER_AXIS;

struct State {
    program: Program,
    events: EventsRx,
    updates: UpdatesTx,
    mem: [u8; MEM_SIZE],
    window: Option<Window>,
    pixels: Option<Pixels>,
}

impl ApplicationHandler for State {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = self.window.get_or_insert_with(|| {
            event_loop
                .create_window(
                    Window::default_attributes().with_title("Caterpillar"),
                )
                .unwrap()
        });

        self.pixels.get_or_insert_with(|| {
            let size_u32: u32 = PIXELS_PER_AXIS
                .try_into()
                .expect("Expected `SIZE` to fit into `u32`");

            let surface_texture =
                SurfaceTexture::new(size_u32, size_u32, window);
            Pixels::new(size_u32, size_u32, surface_texture).unwrap()
        });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(pixels) = &self.pixels else { return };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                ..
            } => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Err(err) = pixels.render() {
                    eprintln!("Render error: {err}");
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        let Some(window) = &self.window else { return };
        let Some(pixels) = &mut self.pixels else {
            return;
        };

        let start_of_execution = Instant::now();
        let timeout = Duration::from_millis(5);

        loop {
            while let Ok(event) = self.events.try_recv() {
                // This doesn't work so well. This receive loop was moved here,
                // so we can have some control over the program from the
                // debugger, while it is stuck in an endless loop.
                //
                // And this works somewhat. We can send events. But unless those
                // events result in the program to stop running, we won't see
                // any indication of them being received in the debugger, as the
                // program isn't sent when it's running.

                self.program.apply_debug_event(event, &mut self.mem);
                self.updates.send(&self.program);
            }

            // This block needs to be located here, as receiving events from the
            // client can lead to a reset, which then must result in the
            // arguments being available, or the program can't work correctly.
            if let ProgramState::Finished = self.program.state {
                self.program.reset(&mut self.mem);
                self.program
                    .push([Value(TILES_PER_AXIS.try_into().unwrap()); 2]);
            }

            match self.program.step(&mut self.mem) {
                ProgramState::Running => {}
                ProgramState::Paused { .. } => {
                    break;
                }
                ProgramState::Finished => {
                    assert_eq!(
                        self.program.evaluator.data_stack.num_values(),
                        0
                    );
                    break;
                }
                ProgramState::Error { .. } => {
                    break;
                }
            }

            if start_of_execution.elapsed() > timeout {
                self.program.halt();
            }
        }

        self.updates.send(&self.program);

        for tile_y in 0..TILES_PER_AXIS {
            for tile_x in 0..TILES_PER_AXIS {
                let i =
                    TILES_OFFSET_IN_MEMORY + tile_y * TILES_PER_AXIS + tile_x;
                let tile = self.mem[i];

                let color = if tile == 0 {
                    [0, 0, 0, 0]
                } else {
                    [255, 255, 255, 255]
                };

                for offset_y in 0..PIXELS_PER_TILE_AXIS {
                    for offset_x in 0..PIXELS_PER_TILE_AXIS {
                        let num_channels = 4;

                        let frame_x = (tile_x * PIXELS_PER_TILE_AXIS
                            + offset_x)
                            * num_channels;
                        let frame_y = (tile_y * PIXELS_PER_TILE_AXIS
                            + offset_y)
                            * num_channels;

                        let i = frame_y * PIXELS_PER_AXIS + frame_x;
                        pixels.frame_mut()[i..i + num_channels]
                            .copy_from_slice(&color);
                    }
                }
            }
        }

        window.request_redraw();
    }
}
