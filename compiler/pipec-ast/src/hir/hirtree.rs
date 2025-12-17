use super::HIRNode;
use pipec_cache::{Cached, Decode, Encode};

#[derive(PartialEq, Decode, Encode, Hash, Debug, Clone)]
pub struct HIRTree {
    stream: Vec<HIRNode>,
    pos: usize,
}
impl Cached for HIRTree {}

impl HIRTree {
    pub fn new(input: Vec<HIRNode>) -> Self {
        Self {
            stream: input,
            pos: 0,
        }
    }
    pub fn current_node(&mut self) -> Option<&HIRNode> {
        self.stream.get(self.pos)
    }
    pub fn next_node(&mut self) -> Option<&HIRNode> {
        self.pos += 1;
        self.stream.get(self.pos - 1)
    }
    pub fn peek(&mut self) -> Option<&HIRNode> {
        self.stream.get(self.pos)
    }
    pub fn from_vec(vec: Vec<HIRNode>) -> Self {
        Self {
            stream: vec,
            pos: 0,
        }
    }

    pub fn reset(&mut self) {
        self.pos = 0;
    }
}
