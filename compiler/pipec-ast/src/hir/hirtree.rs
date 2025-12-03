use super::HIRNode;
use pipec_cache::{Cached, Decode, Encode};

#[derive(Decode, Encode, Hash, Debug)]
pub struct HIRTree(Vec<HIRNode>);
impl Cached for HIRTree {}

impl HIRTree {
    pub fn new(input: Vec<HIRNode>) -> Self {
        Self(input)
    }
}
