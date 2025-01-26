use std::panic;

use crosscut_game_engine::{command::Command, game_engine::GameEngine};
use crosscut_protocol::{
    command::{CommandExt, SerializedCommandToRuntime},
    updates::Updates,
};

use crate::ffi_out::on_panic;

pub struct Host {
    pub game_engine: GameEngine,
    pub commands: Vec<SerializedCommandToRuntime>,
    pub updates: Updates,
}

impl Host {
    pub fn new() -> Self {
        panic::set_hook(Box::new(|panic_info| {
            on_panic(&panic_info.to_string());
        }));

        Self {
            game_engine: GameEngine::new(),
            commands: Vec::new(),
            updates: Updates::default(),
        }
    }

    pub fn update(&mut self, current_time_ms: f64, pixels: &mut [u8]) {
        for command in self.commands.drain(..) {
            let command = Command::deserialize(command);
            self.game_engine.on_command(command);
        }

        self.game_engine
            .run_until_end_of_frame(current_time_ms / 1000.0, pixels);

        self.updates.queue_updates(
            &self.game_engine.runtime,
            self.game_engine.memory(),
        );
    }
}

impl Default for Host {
    fn default() -> Self {
        Self::new()
    }
}
