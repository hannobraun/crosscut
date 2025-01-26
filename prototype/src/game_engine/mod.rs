mod game_engine;
mod terminal_editor;

pub use self::{
    game_engine::{GameInput, GameOutput},
    terminal_editor::input::TerminalInputEvent,
};
