#[derive(Debug)]
pub struct Host {}

impl Host {
    pub fn new() -> Self {
        Self {}
    }

    pub fn function_id_by_name(&self, _name: &str) -> Option<u32> {
        // This is a placeholder while host functions are still being
        // implemented.
        None
    }

    pub fn function_name_by_id(&self, _id: &u32) -> &str {
        // This is a placeholder while host functions are still being
        // implemented.
        ""
    }
}
