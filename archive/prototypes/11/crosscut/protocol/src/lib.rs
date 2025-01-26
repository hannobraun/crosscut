pub mod command;
pub mod host_state;
pub mod updates;

/// The size of the updates buffer
///
/// This is a ridiculous 1 MiB large. It should be possible to make this much
/// smaller, but for now, we're using a very space-inefficient serialization
/// format.
pub const UPDATES_BUFFER_SIZE: usize = 1024 * 1024;

/// The size of the commands buffer
///
/// This is a ridiculous 1 MiB large. It should be possible to make this much
/// smaller, but for now, we're using a very space-inefficient serialization
/// format.
pub const COMMANDS_BUFFER_SIZE: usize = 1024 * 1024;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Versioned<T> {
    pub timestamp: u64,
    pub inner: T,
}

pub fn ron_options() -> ron::Options {
    ron::Options::default()
        // The default recursion limit (as of this writing) is `128`. After some
        // recent changes, this started resulting in a
        // `ron::Error::ExceededRecursionLimit`.
        //
        // I don't think there's a deep reason for that. What we're sending is
        // just _very_ unoptimized, and another layer of abstraction that I
        // added recently pushed us over the edge.
        //
        // Let's not worry about it too much for now. Once the traffic between
        // the various components in a development setup becomes relevant, we'll
        // need to replace RON with something more space-efficient anyway. And
        // worry more about what we're sending in the first place.
        .with_recursion_limit(256)
}
