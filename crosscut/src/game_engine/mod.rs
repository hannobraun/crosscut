mod editor;
mod game;
mod game_engine;

pub use self::{
    editor::input::TerminalInput,
    game::{Game, PureCrosscutGame},
    game_engine::{GameEngine, GameOutput},
};

#[cfg(test)]
#[allow(unused)] // used only intermittently, to debug tests
pub use self::editor::output::codebase_to_stdout;

#[cfg(test)]
#[allow(unused)] // user was removed; but likely to become used again soon
pub use self::editor::output::codebase_to_string;

#[cfg(test)]
mod tests;
