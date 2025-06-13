mod camera;
mod editor;
mod game;
mod game_engine;
mod graphics;
mod start;

pub use self::{
    camera::{Camera, OrthographicProjection},
    editor::input::TerminalInput,
    game::{Game, Init, PureCrosscutGame, PureCrosscutGameStart},
    graphics::{Instance, Renderer},
    start::start_and_wait,
};

#[cfg(test)]
#[allow(unused)] // used only intermittently, to debug tests
pub use self::editor::output::node_to_stdout;

#[cfg(test)]
#[allow(unused)] // user was removed; but likely to become used again soon
pub use self::editor::output::node_to_string;
