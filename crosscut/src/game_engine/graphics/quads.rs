use crate::Camera;

pub struct Quads {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
}

impl Quads {
    const NUM_VERTICES: u32 = 4;

    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> Self {
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

        let vertices = [[0.5, -0.5], [0.5, 0.5], [-0.5, -0.5], [-0.5, 0.5]]
            .map(|[x, y]| {
                let position = [x, y, 0.];
                Vertex { position }
            });

        {
            let Ok(num_vertices) = vertices.len().try_into() else {
                unreachable!(
                    "Number of vertices defined here fits into an `u32`."
                );
            };

            let num_vertices: u32 = num_vertices;
            assert!(num_vertices == Self::NUM_VERTICES);
        }

        let vertex_buffer = device.create_buffer(&Vertex::buffer_descriptor());
        queue.write_buffer(&vertex_buffer, 0, bytemuck::cast_slice(&vertices));

        let instance_buffer =
            device.create_buffer(&Instance::buffer_descriptor());

        Self {
            pipeline,
            bind_group,
            uniform_buffer,
            vertex_buffer,
            instance_buffer,
        }
    }

    pub fn draw(
        &self,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        bg_color: wgpu::Color,
        positions: impl IntoIterator<Item = Instance>,
        camera: &Camera,
    ) {
        queue.write_buffer(
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

        queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&instances),
        );

        {
            let mut render_pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view,
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
            render_pass.draw(0..Quads::NUM_VERTICES, 0..num_instances);
        }
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
pub struct Instance {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

impl Instance {
    const MAX_NUM: u64 = 1024;
    const ATTRIBUTES: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x4,
    ];

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
            attributes: Self::ATTRIBUTES,
        }
    }
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Vertex {
    position: [f32; 3],
}

impl Vertex {
    fn size() -> u64 {
        let Ok(size) = size_of::<Self>().try_into() else {
            unreachable!("Size of `Vertex` can surely fit into a `u64`");
        };

        size
    }

    fn buffer_descriptor() -> wgpu::BufferDescriptor<'static> {
        let num_vertices: u64 = Quads::NUM_VERTICES.into();

        wgpu::BufferDescriptor {
            label: None,
            size: Self::size() * num_vertices,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        }
    }

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: Self::size(),
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![
                2 => Float32x3,
            ],
        }
    }
}
