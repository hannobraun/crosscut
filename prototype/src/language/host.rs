use std::collections::BTreeMap;

use super::code::HostFunction;

pub struct Host {
    pub functions: BTreeMap<String, HostFunction>,
}
