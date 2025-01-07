use crate::code::Signature;

/// # A host into which a Crosscut application is embedded
///
/// ## Implementation Note
///
/// There's really no good reason that this should be a trait and not just a
/// struct.
pub trait Host {
    /// # Iterate over all of the host's functions
    ///
    /// Implementations must guarantee that each function has a unique number
    /// (see [`HostFunction::number`]) and name (see [`HostFunction::name`]).
    fn functions(&self) -> impl IntoIterator<Item = HostFunction>;

    /// # Access a host function by its number
    ///
    /// Returns `None`, if the provided number does not identify a host
    /// function.
    fn function_by_number(&self, number: u8) -> Option<HostFunction> {
        self.functions()
            .into_iter()
            .find(|function| function.number == number)
    }

    /// # Access a host function by its name
    ///
    /// Returns `None`, if the provided name does not identify a host function.
    fn function_by_name(&self, name: &str) -> Option<HostFunction> {
        self.functions()
            .into_iter()
            .find(|function| function.name == name)
    }
}

/// # A function that is provided by the host
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct HostFunction {
    /// # The name that identifies the function in input code
    pub name: String,

    /// # The number that identifies the function in the host effect
    pub number: u8,

    /// # The type signature of the function
    pub signature: Signature,
}

/// # A [`Host`] implementation that can be used where no host is required
///
/// Does not provide any functions. This is used by compiler unit tests, but
/// might also come in handy elsewhere.
pub struct NoHost;

impl Host for NoHost {
    fn functions(&self) -> impl IntoIterator<Item = HostFunction> {
        None
    }
}
