use std::sync::Arc;

use anyhow::anyhow;
use winit::window::Window;

use crate::Camera;

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
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

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: Uniforms::size(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });
        queue.write_buffer(
            &uniform_buffer,
            0,
            bytemuck::cast_slice(&[Uniforms::default()]),
        );

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

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
                layout: Some(&pipeline_layout),
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
                        format: surface_config.format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        let vertex_buffer = device.create_buffer(&Vertex::buffer_descriptor());
        let instance_buffer =
            device.create_buffer(&Instance::buffer_descriptor());

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

        queue.write_buffer(&vertex_buffer, 0, bytemuck::cast_slice(&vertices));

        Ok(Self {
            surface,
            surface_config,
            device,
            queue,
            pipeline,
            bind_group,
            uniform_buffer,
            vertex_buffer,
            num_vertices,
            instance_buffer,
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
            &self.uniform_buffer,
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
            &self.instance_buffer,
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

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.draw(0..self.num_vertices, 0..num_instances);
        }

        self.queue.submit(Some(encoder.finish()));
        surface_texture.present();

        Ok(())
    }
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Uniforms {
    transform: [[f32; 4]; 4],
}

impl Uniforms {
    fn size() -> u64 {
        let Ok(size) = size_of::<Self>().try_into() else {
            unreachable!("Size of `Self` definitely fits into a `u64`.");
        };

        size
    }
}

impl Default for Uniforms {
    fn default() -> Self {
        let transform = Camera::default().to_transform();
        Self { transform }
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
                0 => Float32x3,
            ],
        }
    }
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Instance {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

impl Instance {
    const MAX_NUM: u64 = 1024;
    const ATTRIBUTES: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        1 => Float32x3,
        2 => Float32x4,
    ];

    fn size() -> u64 {
        let Ok(size) = size_of::<Self>().try_into() else {
            unreachable!("Size of `Instance` can surely fit into a `u64`");
        };

        size
    }

    pub fn buffer_descriptor() -> wgpu::BufferDescriptor<'static> {
        wgpu::BufferDescriptor {
            label: None,
            size: Self::size() * Self::MAX_NUM,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        }
    }

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: Self::size(),
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: Self::ATTRIBUTES,
        }
    }
}
