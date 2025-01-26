/// Linear memory that games can access
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Memory {
    #[serde(with = "serde_big_array::BigArray")]
    pub inner: [u8; 256],
}

impl Default for Memory {
    fn default() -> Self {
        Self { inner: [0; 256] }
    }
}
