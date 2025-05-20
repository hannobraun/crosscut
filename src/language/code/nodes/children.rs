use super::NodeHash;

pub struct Children<'r> {
    pub hashes: Vec<&'r NodeHash>,
}
