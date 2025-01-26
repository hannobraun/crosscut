use crosscut_compiler::host::{Host, HostFunction};

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct GameEngineHost;

impl Host for GameEngineHost {
    fn functions(&self) -> impl IntoIterator<Item = HostFunction> {
        use GameEngineFunction::*;

        [
            &Halt,
            &Load,
            &Store,
            &ReadInput,
            &ReadRandom,
            &SetPixel,
            &SubmitFrame,
        ]
        .map(|function| function.function())
    }
}

/// # An effect handled by the game engine host
///
/// ## Implementation Note
///
/// The host functions that are backed by these effects are not purely
/// functional. Long-term, they should be, but for now there's not much point to
/// it.
///
/// Without a type system, piping any values that represent I/O resources
/// through host functions is only complexity for no gain. And without a
/// _linear_ type system, there's no way to guarantee any sane semantics around
/// such functions anyway.
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    num_enum::IntoPrimitive,
    num_enum::TryFromPrimitive,
    serde::Deserialize,
    serde::Serialize,
)]
#[repr(u8)]
pub enum GameEngineFunction {
    /// # Halt the game
    ///
    /// This essentially works like the `brk` intrinsic. It was added
    /// specifically to have a breakpoint-like effect in the game engine, for
    /// use in the debugger's test suite. Maybe it will find other uses later.
    ///
    /// ## Input
    ///
    /// none
    ///
    /// ## Output
    ///
    /// none
    Halt,

    /// # Load a value from a given memory address
    ///
    /// ## Input
    ///
    /// - `u8`: The address of the value to read.
    ///
    /// ## Output
    ///
    /// - `u8`: The value at the provided address address.
    Load,

    /// # Store a value at the given memory address
    ///
    /// ## Input
    ///
    /// - `u8`: The value to store.
    /// - `u8`: The address to store the value at.
    ///
    /// ## Output
    ///
    /// none
    Store,

    /// # Read the next input event from the buffer
    ///
    /// ## Input
    ///
    /// none
    ///
    /// ## Output
    ///
    /// - `u8`: A value representing the type of input event.
    ReadInput,

    /// # Read a random value from the buffer
    ///
    /// ## Input
    ///
    /// none
    ///
    /// ## Output
    ///
    /// - `s32`: The random value.
    ReadRandom,

    /// # Set a pixel in the frame buffer
    ///
    /// ## Input
    ///
    /// - `u8`: The x-coordinate of the pixel.
    /// - `u8`: The y-coordinate of the pixel.
    /// - `u8`: The red channel value of the pixel.
    /// - `u8`: The green channel value of the pixel.
    /// - `u8`: The blue channel value of the pixel.
    /// - `u8`: The alpha channel value of the pixel.
    ///
    /// ## Output
    ///
    /// none
    SetPixel,

    /// # Submit the current frame, causing the game engine to display it
    ///
    /// This must be called regularly, or the game engine will freeze.
    ///
    /// ## Input
    ///
    /// none
    ///
    /// ## Output
    ///
    /// none
    ///
    /// ## Implementation Note
    ///
    /// The possibility of the game engine freezing is undesirable. At some
    /// future point, there will likely be an enforced timeout. This is tracked
    /// in the following issue:
    /// <https://github.com/hannobraun/crosscut/issues/42>
    SubmitFrame,
}

impl GameEngineFunction {
    fn function(&self) -> HostFunction {
        use crosscut_compiler::code::Type::*;

        let name = match self {
            Self::Halt => "halt",
            Self::Load => "load",
            Self::Store => "store",
            Self::ReadInput => "read_input",
            Self::ReadRandom => "read_random",
            Self::SetPixel => "set_pixel",
            Self::SubmitFrame => "submit_frame",
        };
        let number = (*self).into();
        let signature = match self {
            Self::Halt => ([], []).into(),
            Self::Load => ([Number], [Number]).into(),
            Self::Store => ([Number, Number], []).into(),
            Self::ReadInput => ([], [Number]).into(),
            Self::ReadRandom => ([], [Number]).into(),
            Self::SetPixel => {
                ([Number, Number, Number, Number, Number, Number], []).into()
            }
            Self::SubmitFrame => ([], []).into(),
        };

        HostFunction {
            name: name.into(),
            number,
            signature,
        }
    }
}
