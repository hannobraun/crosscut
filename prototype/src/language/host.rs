use std::collections::BTreeMap;

pub struct Host {
    pub functions_by_name: BTreeMap<String, usize>,
}

impl Host {
    pub fn empty() -> Self {
        Self {
            functions_by_name: BTreeMap::new(),
        }
    }
}
