use std::sync::Arc;

use anyhow::anyhow;
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
    game_engine::{GameEngine, GameOutput, OnRender, TerminalInput},
    threads::{self, Receiver, Sender},
};

use super::terminal::output::RawTerminalAdapter;

pub fn start_and_wait(
    game: Box<dyn Game + Send>,
    terminal_input: Receiver<TerminalInput>,
    _: Sender<OnRender>,
    _: Receiver<GameOutput>,
) -> anyhow::Result<()> {
    let game_engine = GameEngine::with_editor_ui(game)?;

    let mut handler = Handler {
        game_engine,
        terminal_input,
        resources: None,
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
    resources: Option<Resources>,
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
        match Resources::new(event_loop) {
            Ok(resources) => {
                self.resources = Some(resources);
            }
            Err(err) => {
                self.on_error(err, event_loop);
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _: WindowId,
        event: WindowEvent,
    ) {
        let Some(resources) = self.resources.as_ref() else {
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
                            threads::ChannelDisconnected,
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

                if let Err(err) = resources.renderer.render(self.color) {
                    self.on_error(err, event_loop);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        let Some(resources) = self.resources.as_ref() else {
            return;
        };

        resources.window.request_redraw();
    }
}

fn on_frame(
    game_engine: &mut GameEngine<RawTerminalAdapter>,
    terminal_input: &Receiver<TerminalInput>,
    color: &mut wgpu::Color,
) -> Result<(), OnFrameError> {
    // If a new frame is being rendered on the other thread, then the game
    // engine can get ready to provide the next one.
    game_engine.on_frame()?;

    while let Some(input) = terminal_input.try_recv()? {
        game_engine.on_editor_input(input)?;
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
    ChannelDisconnected(#[from] threads::ChannelDisconnected),

    #[error(transparent)]
    GameEngine(#[from] anyhow::Error),
}

struct Resources {
    window: Arc<Window>,
    renderer: Renderer,
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

        Ok(Self { window, renderer })
    }
}

struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl Renderer {
    async fn new(window: &Arc<Window>) -> anyhow::Result<Self> {
        let instance =
            wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let size = window.inner_size();
        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .ok_or_else(|| {
                anyhow!("Could not acquire default surface configuration.")
            })?;
        surface.configure(&device, &config);

        Ok(Self {
            surface,
            device,
            queue,
        })
    }

    fn render(&self, bg_color: wgpu::Color) -> anyhow::Result<()> {
        let surface_texture = self.surface.get_current_texture()?;
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: None },
        );
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(bg_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        self.queue.submit(Some(encoder.finish()));
        surface_texture.present();

        Ok(())
    }
}
