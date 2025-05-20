use crate::{language::code::NodeHash, util::form::Form};

pub struct NodeByHash;

impl Form for NodeByHash {
    type Form<T: 'static> = NodeHash;
}
