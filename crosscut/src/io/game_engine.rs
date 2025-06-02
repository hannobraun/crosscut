use std::sync::Arc;

use pollster::FutureExt;
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

use crate::{
    Game,
    game_engine::{GameEngine, GameOutput, Renderer, TerminalInput},
    terminal::{self, Receiver},
};

use super::terminal::output::RawTerminalAdapter;

pub fn start_and_wait(
    game: Box<dyn Game + Send>,
    terminal_input: Receiver<TerminalInput>,
) -> anyhow::Result<()> {
    let game_engine = GameEngine::with_editor_ui(game)?;

    let mut handler = Handler {
        game_engine,
        terminal_input,
        resources: Resources::Uninitialized,
        result: Ok(()),
        color: wgpu::Color::BLACK,
    };

    let event_loop = EventLoop::new()?;
    event_loop.run_app(&mut handler)?;

    handler.result
}

struct Handler {
    game_engine: GameEngine<RawTerminalAdapter>,
    terminal_input: Receiver<TerminalInput>,
    resources: Resources,
    result: anyhow::Result<()>,
    color: wgpu::Color,
}

impl Handler {
    fn on_error(&mut self, err: anyhow::Error, event_loop: &ActiveEventLoop) {
        self.result = Err(err);
        event_loop.exit();
    }
}

impl ApplicationHandler for Handler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Resources::Uninitialized = self.resources {
            match Resources::new(event_loop) {
                Ok(resources) => {
                    self.resources = resources;
                }
                Err(err) => {
                    self.on_error(err, event_loop);
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _: WindowId,
        event: WindowEvent,
    ) {
        let Resources::Initialized { renderer, .. } = &self.resources else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        ..
                    },
                ..
            } => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Err(err) = on_frame(
                    &mut self.game_engine,
                    &self.terminal_input,
                    &mut self.color,
                ) {
                    match err {
                        OnFrameError::ChannelDisconnected(
                            terminal::ChannelDisconnected,
                        ) => {
                            // The other end has hung up. We should shut down
                            // too.
                            event_loop.exit();
                        }
                        OnFrameError::GameEngine(err) => {
                            self.on_error(err, event_loop);
                        }
                    }

                    return;
                }

                if let Err(err) = renderer.render(self.color) {
                    self.on_error(err, event_loop);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        let Resources::Initialized { window, .. } = &self.resources else {
            return;
        };

        window.request_redraw();
    }
}

fn on_frame(
    game_engine: &mut GameEngine<RawTerminalAdapter>,
    terminal_input: &Receiver<TerminalInput>,
    color: &mut wgpu::Color,
) -> Result<(), OnFrameError> {
    game_engine.on_frame()?;

    while let Some(input) = terminal_input.try_recv()? {
        game_engine.on_terminal_input(input)?;
    }

    for GameOutput::SubmitColor {
        color: [r, g, b, a],
    } in game_engine.game_output()
    {
        *color = wgpu::Color { r, g, b, a };
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
enum OnFrameError {
    #[error(transparent)]
    ChannelDisconnected(#[from] terminal::ChannelDisconnected),

    #[error(transparent)]
    GameEngine(#[from] anyhow::Error),
}

enum Resources {
    Uninitialized,
    Initialized {
        window: Arc<Window>,
        renderer: Renderer,
    },
}

impl Resources {
    fn new(event_loop: &ActiveEventLoop) -> anyhow::Result<Self> {
        let window = {
            let window = event_loop.create_window(
                Window::default_attributes().with_title("Crosscut"),
            )?;
            Arc::new(window)
        };

        let renderer = Renderer::new(&window).block_on()?;

        Ok(Self::Initialized { window, renderer })
    }
}
