use crosscut_game_engine::command::Command;

pub trait CommandExt {
    fn deserialize(bytes: SerializedCommandToRuntime) -> Self;
    fn serialize(&self) -> SerializedCommandToRuntime;
}

impl CommandExt for Command {
    fn deserialize(bytes: SerializedCommandToRuntime) -> Self {
        let string = std::str::from_utf8(&bytes).unwrap();
        ron::from_str(string).unwrap()
    }

    fn serialize(&self) -> SerializedCommandToRuntime {
        ron::to_string(self).unwrap().into_bytes()
    }
}

pub type SerializedCommandToRuntime = Vec<u8>;
