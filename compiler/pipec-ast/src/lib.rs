use std::path::PathBuf;

pub mod ast;
pub mod tokenizer;

pub struct RecursiveGuard(Vec<PathBuf>);
impl RecursiveGuard {
    pub(crate) fn contains(&self, input: &PathBuf) -> bool {
        self.0.contains(input)
    }
    pub fn push(&mut self, input: PathBuf) {
        self.0.push(input)
    }
    pub fn new() -> Self {
        Self(Vec::with_capacity(30))
    }
}
