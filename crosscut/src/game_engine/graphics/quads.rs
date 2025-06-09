use crate::Camera;

pub struct Quads {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    pub uniform_buffer: wgpu::Buffer,
    pub vertex_buffer: wgpu::Buffer,
    pub num_vertices: u32,
    pub instance_buffer: wgpu::Buffer,
}

impl Quads {
    pub const NUM_VERTICES: u32 = 4;

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

        let vertex_buffer = device.create_buffer(&Vertex::buffer_descriptor());
        queue.write_buffer(&vertex_buffer, 0, bytemuck::cast_slice(&vertices));

        let num_vertices: u32 = {
            let Ok(len) = vertices.len().try_into() else {
                unreachable!(
                    "Number of vertices defined here fits into an `u32`."
                );
            };

            len
        };

        {
            assert!(num_vertices == Self::NUM_VERTICES);
        }

        let instance_buffer =
            device.create_buffer(&Instance::buffer_descriptor());

        Self {
            pipeline,
            bind_group,
            uniform_buffer,
            vertex_buffer,
            num_vertices,
            instance_buffer,
        }
    }
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Uniforms {
    pub transform: [[f32; 4]; 4],
}

impl Uniforms {
    pub fn size() -> u64 {
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
    pub const MAX_NUM: u64 = 1024;
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
                0 => Float32x3,
            ],
        }
    }
}
