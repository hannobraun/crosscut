#[cfg(test)]
pub struct Host {}

#[cfg(test)]
impl Host {
    pub fn without_functions() -> Self {
        Self {}
    }
}
