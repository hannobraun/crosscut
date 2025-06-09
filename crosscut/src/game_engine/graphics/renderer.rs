use std::sync::Arc;

use anyhow::anyhow;
use winit::window::Window;

use crate::{
    Camera, Instance,
    game_engine::graphics::quads::{Quads, Uniforms},
};

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
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

        let quads = Quads::new(&device, &queue, &surface_config);

        Ok(Self {
            surface,
            surface_config,
            device,
            queue,
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
        &self,
        bg_color: wgpu::Color,
        positions: impl IntoIterator<Item = Instance>,
        camera: &Camera,
    ) -> anyhow::Result<()> {
        self.queue.write_buffer(
            &self.quads.uniform_buffer,
            0,
            bytemuck::cast_slice(&[Uniforms {
                transform: camera.to_transform(),
            }]),
        );

        let instances = positions.into_iter().collect::<Vec<_>>();
        let num_instances: u32 = {
            let Ok(len) = instances.len().try_into() else {
                panic!(
                    "A number of instances that doesn't fit into a `u32` is \
                    not supported."
                );
            };

            len
        };

        {
            let num_instances: u64 = num_instances.into();
            assert!(num_instances <= Instance::MAX_NUM);
        }

        self.queue.write_buffer(
            &self.quads.instance_buffer,
            0,
            bytemuck::cast_slice(&instances),
        );

        let surface_texture = self.surface.get_current_texture()?;
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: None },
        );

        {
            let mut render_pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(bg_color),
                                store: wgpu::StoreOp::Store,
                            },
                        },
                    )],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

            render_pass.set_pipeline(&self.quads.pipeline);
            render_pass.set_bind_group(0, &self.quads.bind_group, &[]);
            render_pass
                .set_vertex_buffer(0, self.quads.vertex_buffer.slice(..));
            render_pass
                .set_vertex_buffer(1, self.quads.instance_buffer.slice(..));
            render_pass.draw(0..Quads::NUM_VERTICES, 0..num_instances);
        }

        self.queue.submit(Some(encoder.finish()));
        surface_texture.present();

        Ok(())
    }
}
