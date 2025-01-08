pub struct Host {}

impl Host {
    #[cfg(test)]
    pub fn without_functions() -> Self {
        Self {}
    }

    pub fn from_function_names(
        _: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {}
    }
}
