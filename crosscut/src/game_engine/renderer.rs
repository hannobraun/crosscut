use std::sync::Arc;

use anyhow::anyhow;
use winit::window::Window;

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
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

        let vertex_buffer = device.create_buffer(&Vertex::buffer_descriptor());
        let instance_buffer =
            device.create_buffer(&Instance::buffer_descriptor());

        let shader =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("shader.wgsl").into(),
                ),
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: None,
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vert_main"),
                    compilation_options:
                        wgpu::PipelineCompilationOptions::default(),
                    buffers: &[Vertex::layout(), Instance::layout()],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("frag_main"),
                    compilation_options:
                        wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        Ok(Self {
            surface,
            device,
            queue,
            pipeline,
            vertex_buffer,
            instance_buffer,
        })
    }

    pub fn render(&self, bg_color: wgpu::Color) -> anyhow::Result<()> {
        let vertices = [[0.5, -0.5], [0.5, 0.5], [-0.5, -0.5], [-0.5, 0.5]]
            .map(|[x, y]| {
                let position = [x, y, 0.];
                Vertex { position }
            });
        let num_vertices: u32 = {
            let Ok(len) = vertices.len().try_into() else {
                unreachable!(
                    "Number of vertices defined here fits into an `u32`."
                );
            };

            len
        };

        {
            let num_vertices: u64 = num_vertices.into();
            assert!(num_vertices <= Vertex::MAX_NUM);
        }

        self.queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(&vertices),
        );

        let instance = Instance {
            position: [0., 0., 0.],
        };
        let num_instances = 1;

        {
            let num_instances: u64 = num_instances.into();
            assert!(num_instances <= Instance::MAX_NUM);
        }

        self.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&[instance]),
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

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.draw(0..num_vertices, 0..num_instances);
        }

        self.queue.submit(Some(encoder.finish()));
        surface_texture.present();

        Ok(())
    }
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Vertex {
    position: [f32; 3],
}

impl Vertex {
    const MAX_NUM: u64 = 4;

    fn size() -> u64 {
        let Ok(size) = size_of::<Self>().try_into() else {
            unreachable!("Size of `Vertex` can surely fit into a `u64`");
        };

        size
    }

    fn buffer_descriptor() -> wgpu::BufferDescriptor<'static> {
        wgpu::BufferDescriptor {
            label: None,
            size: Self::size() * Self::MAX_NUM,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        }
    }

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: Self::size(),
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![
                0 => Float32x3
            ],
        }
    }
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Instance {
    position: [f32; 3],
}

impl Instance {
    const MAX_NUM: u64 = 1;

    fn size() -> u64 {
        let Ok(size) = size_of::<Self>().try_into() else {
            unreachable!("Size of `Instance` can surely fit into a `u64`");
        };

        size
    }

    fn buffer_descriptor() -> wgpu::BufferDescriptor<'static> {
        wgpu::BufferDescriptor {
            label: None,
            size: Self::size() * Self::MAX_NUM,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        }
    }

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: Self::size(),
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![
                1 => Float32x3
            ],
        }
    }
}
