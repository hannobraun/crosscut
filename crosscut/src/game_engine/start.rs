use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey, PhysicalKey},
    window::{Window, WindowId},
};

use crate::{
    game_engine::{TerminalInput, game_engine::GameEngine},
    terminal::{self, RawTerminalAdapter, Receiver},
};

use super::Init;

pub fn start_and_wait(
    init: Box<dyn Init + Send>,
    terminal_input: Receiver<TerminalInput>,
) -> anyhow::Result<()> {
    let mut handler = Handler {
        terminal_input,
        resources: Resources::Uninitialized { init: Some(init) },
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
            WindowEvent::Resized(new_size) => {
                let new_size = [new_size.width, new_size.height];
                game_engine.on_window_resized(new_size);
            }
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
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                game_engine.on_key(key);
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
        /// # Implementatioan of [`Init`] that initializes the game
        ///
        /// This field is always going to be `Some`, except right before this
        /// variant is replaced by `Self::Initialized`. The only reason we have
        /// an `Option` here, is to deal with the fact that we only ever have a
        /// mutable reference to this field, and can't easily move out of it.
        init: Option<Box<dyn Init>>,
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
        if let Resources::Uninitialized { init } = self {
            let Some(init) = init.take() else {
                unreachable!(
                    "`game` should always be `Some`, unless the following code \
                    panics, before we replace `self` below. That would be a \
                    bug."
                );
            };

            let window = {
                let window = event_loop.create_window(
                    Window::default_attributes()
                        .with_title("Game made with Crosscut"),
                )?;
                Arc::new(window)
            };

            let game_engine = GameEngine::with_editor_ui(init, &window)?;

            *self = Self::Initialized {
                window,
                game_engine,
            };
        }

        Ok(())
    }
}
