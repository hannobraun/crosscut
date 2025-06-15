use glam::Vec2;

pub struct Text {
    font_system: glyphon::FontSystem,
    swash_cache: glyphon::SwashCache,
    atlas: glyphon::TextAtlas,
    viewport: glyphon::Viewport,
    text_renderer: glyphon::TextRenderer,
}

impl Text {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        let font_system = glyphon::FontSystem::new();

        let cache = glyphon::Cache::new(device);
        let swash_cache = glyphon::SwashCache::new();

        let mut atlas =
            glyphon::TextAtlas::new(device, queue, &cache, texture_format);

        let viewport = glyphon::Viewport::new(device, &cache);

        let text_renderer = glyphon::TextRenderer::new(
            &mut atlas,
            device,
            wgpu::MultisampleState::default(),
            None,
        );

        Self {
            font_system,
            swash_cache,
            atlas,
            viewport,
            text_renderer,
        }
    }

    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        surface_config: &wgpu::SurfaceConfiguration,
        texts: impl IntoIterator<Item = (String, Vec2)>,
    ) -> anyhow::Result<()> {
        self.viewport.update(
            queue,
            glyphon::Resolution {
                width: surface_config.width,
                height: surface_config.height,
            },
        );

        let mut buffers = Vec::new();

        for (text, position) in texts {
            let mut buffer = glyphon::Buffer::new(
                &mut self.font_system,
                glyphon::Metrics {
                    font_size: 16.0,
                    line_height: 16.0,
                },
            );
            buffer.set_text(
                &mut self.font_system,
                &text,
                &glyphon::Attrs::new(),
                glyphon::Shaping::Basic,
            );

            buffers.push((buffer, position));
        }

        let mut text_areas = Vec::new();

        for (buffer, position) in &buffers {
            let [Ok(right), Ok(bottom)] =
                [surface_config.width, surface_config.height]
                    .map(|dimension| dimension.try_into())
            else {
                unreachable!(
                    "It should not be possible to have window dimensions that \
                    don't fit into an `i32`."
                );
            };

            text_areas.push(glyphon::TextArea {
                buffer,
                left: position.x,
                top: position.y,
                scale: 1.,
                bounds: glyphon::TextBounds {
                    left: 0,
                    top: 0,
                    right,
                    bottom,
                },
                default_color: glyphon::Color::rgba(255, 255, 255, 255),
                custom_glyphs: &[],
            });
        }

        self.text_renderer.prepare(
            device,
            queue,
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            text_areas,
            &mut self.swash_cache,
        )?;

        {
            let mut render_pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        },
                    )],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

            self.text_renderer.render(
                &self.atlas,
                &self.viewport,
                &mut render_pass,
            )?;
        }

        Ok(())
    }
}
