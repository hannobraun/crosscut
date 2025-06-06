// It makes sense to prevent this in public APIs, but it also warns me about the
// names of private modules that I only re-export from. That provides no value
// and is pretty annoying.
#![allow(clippy::module_inception)]

mod game_engine;
mod language;
mod terminal;
mod util;

pub use async_trait::async_trait;
pub use wgpu;
pub use winit;

pub use self::{
    game_engine::{
        Camera, Game, GameStart, OrthographicProjection, PureCrosscutGame,
        PureCrosscutGameStart, Renderer,
    },
    language::language::Language,
};

pub fn start_and_wait(game: Box<dyn GameStart + Send>) -> anyhow::Result<()> {
    let terminal = terminal::start()?;

    // This call is going to block until the user requests a shutdown via the
    // game I/O, or any of the other threads shut down.
    game_engine::start_and_wait(game, terminal.input)?;

    // At this point, the shutdown should be in progress. This call shouldn't
    // block for long, if at all. The purpose of still joining the thread is
    // just to get any error that it might have produced.
    terminal.handle.join()?;

    Ok(())
}
