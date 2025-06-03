use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

use crate::{
    Game,
    game_engine::{TerminalInput, game_engine::GameEngine},
    terminal::{self, RawTerminalAdapter, Receiver},
};

pub fn start_and_wait(
    game: Box<dyn Game + Send>,
    terminal_input: Receiver<TerminalInput>,
) -> anyhow::Result<()> {
    let mut handler = Handler {
        terminal_input,
        resources: Resources::Uninitialized { game: Some(game) },
        result: Ok(()),
    };

    let event_loop = EventLoop::new()?;
    event_loop.run_app(&mut handler)?;

    handler.result
}

struct Handler {
    terminal_input: Receiver<TerminalInput>,
    resources: Resources,
    result: anyhow::Result<()>,
}

impl Handler {
    fn on_error(&mut self, err: anyhow::Error, event_loop: &ActiveEventLoop) {
        self.result = Err(err);
        event_loop.exit();
    }
}

impl ApplicationHandler for Handler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(err) = self.resources.init_if_necessary(event_loop) {
            self.on_error(err, event_loop);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _: WindowId,
        event: WindowEvent,
    ) {
        let Resources::Initialized { game_engine, .. } = &mut self.resources
        else {
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
                if let Err(err) = on_frame(game_engine, &self.terminal_input) {
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
) -> Result<(), OnFrameError> {
    game_engine.on_frame()?;

    while let Some(input) = terminal_input.try_recv()? {
        game_engine.on_terminal_input(input)?;
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

#[allow(clippy::large_enum_variant)]
enum Resources {
    Uninitialized {
        game: Option<Box<dyn Game>>,
    },
    Initialized {
        window: Arc<Window>,
        game_engine: GameEngine<RawTerminalAdapter>,
    },
}

impl Resources {
    fn init_if_necessary(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> anyhow::Result<()> {
        if let Resources::Uninitialized { game } = self {
            let Some(game) = game.take() else {
                unreachable!(
                    "`game` should always be `Some`, unless the following code \
                    panics, before we replace `self` below. That would be a \
                    bug."
                );
            };

            let window = {
                let window = event_loop.create_window(
                    Window::default_attributes().with_title("Crosscut"),
                )?;
                Arc::new(window)
            };

            let game_engine = GameEngine::with_editor_ui(game, &window)?;

            *self = Self::Initialized {
                window,
                game_engine,
            };
        }

        Ok(())
    }
}
