use std::collections::BTreeMap;

pub struct Host {
    pub functions_by_id: BTreeMap<usize, String>,
    pub functions_by_name: BTreeMap<String, usize>,
}

impl Host {
    pub fn empty() -> Self {
        Self {
            functions_by_id: BTreeMap::new(),
            functions_by_name: BTreeMap::new(),
        }
    }

    pub fn from_functions(
        names: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        let mut host = Host::empty();

        for (id, name) in names.into_iter().enumerate() {
            let name = name.into();

            host.functions_by_id.insert(id, name.clone());
            host.functions_by_name.insert(name, id);
        }

        host
    }
}
