use std::sync::Arc;

use crosscut::{Color, Game, Language, Renderer, Window, async_trait};

#[derive(Default)]
pub struct Snake {
    renderer: Option<Renderer>,
}

#[async_trait]
impl Game for Snake {
    async fn on_start(
        &mut self,
        _: &mut Language,
        window: &Arc<Window>,
    ) -> anyhow::Result<()> {
        self.renderer = Some(Renderer::new(window).await?);
        Ok(())
    }

    fn on_code_update(&mut self, _: &mut Language) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_frame(&mut self, _: &mut Language) -> anyhow::Result<()> {
        let Some(renderer) = &self.renderer else {
            return Ok(());
        };

        renderer.render(Color {
            r: 1.,
            g: 1.,
            b: 1.,
            a: 1.,
        })?;

        Ok(())
    }
}
