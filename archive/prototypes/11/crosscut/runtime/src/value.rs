use core::fmt;

#[derive(
    Copy,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    serde::Deserialize,
    serde::Serialize,
    udigest::Digestable,
)]
pub struct Value(pub [u8; 4]);

impl Value {
    pub fn to_i8(&self) -> Result<i8, IntegerOverflow> {
        let [v, 0, 0, 0] = self.0 else {
            return Err(IntegerOverflow);
        };
        Ok(i8::from_le_bytes([v]))
    }

    pub fn to_i32(&self) -> i32 {
        i32::from_le_bytes(self.0)
    }

    pub fn to_u8(&self) -> Result<u8, IntegerOverflow> {
        let [v, 0, 0, 0] = self.0 else {
            return Err(IntegerOverflow);
        };
        Ok(u8::from_le_bytes([v]))
    }

    pub fn to_u32(&self) -> u32 {
        u32::from_le_bytes(self.0)
    }

    /// # Convert value to `usize`
    ///
    /// ## Panics
    ///
    /// Panics, if this is running on a platform where `u32` can not be
    /// losslessly converted to `usize`. This should never be an issue on 32-bit
    /// and 64-bit platforms.
    pub fn to_usize(&self) -> usize {
        assert!(
            size_of::<usize>() >= size_of::<u32>(),
            "Expecting lossless conversion of `u32` to `usize` to be possible \
            on all supported platforms.",
        );

        self.to_u32().try_into().expect(
            "Just checked, that `u32` can alway be converted to `usize`",
        )
    }
}

impl From<i8> for Value {
    fn from(value: i8) -> Self {
        let [v] = value.to_le_bytes();
        Value([v, 0, 0, 0])
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Self(value.to_le_bytes())
    }
}

impl From<u8> for Value {
    fn from(value: u8) -> Self {
        let value: u32 = value.into();
        value.into()
    }
}

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Self(value.to_le_bytes())
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x")?;
        for b in self.0.into_iter().rev() {
            write!(f, "{b:02x}")?;
        }

        Ok(())
    }
}

#[derive(
    Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize,
)]
pub struct IntegerOverflow;
