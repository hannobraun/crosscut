use std::sync::Arc;

use anyhow::anyhow;
use winit::window::Window;

use crate::{Camera, Instance, game_engine::graphics::quads::Quads};

use super::background::Background;

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    background: Background,
    quads: Quads,
}

impl Renderer {
    pub async fn new(window: &Arc<Window>) -> anyhow::Result<Self> {
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

        let required_limits = wgpu::Limits::downlevel_webgl2_defaults()
            .using_resolution(adapter.limits());

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits,
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let size = window.inner_size();
        let surface_config = surface
            .get_default_config(&adapter, size.width, size.height)
            .ok_or_else(|| {
                anyhow!("Could not acquire default surface configuration.")
            })?;
        surface.configure(&device, &surface_config);

        let background = Background::new();
        let quads = Quads::new(&device, &queue, &surface_config);

        Ok(Self {
            surface,
            surface_config,
            device,
            queue,
            background,
            quads,
        })
    }

    pub fn handle_resize(&mut self, new_size: [u32; 2]) {
        let [width, height] = new_size;

        self.surface_config.width = width;
        self.surface_config.height = height;

        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn render(
        &mut self,
        bg_color: wgpu::Color,
        positions: impl IntoIterator<Item = Instance>,
        camera: &Camera,
    ) -> anyhow::Result<()> {
        let surface_texture = self.surface.get_current_texture()?;
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: None },
        );

        self.background.draw(&view, &mut encoder, bg_color);
        self.quads
            .draw(&self.queue, &view, &mut encoder, positions, camera);

        self.queue.submit(Some(encoder.finish()));
        surface_texture.present();

        Ok(())
    }
}
