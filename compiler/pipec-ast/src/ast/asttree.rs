use super::ASTNode;
use pipec_arena::{ASpan, Arena};
use pipec_arena_structures::AVec;

#[derive(Debug)]
pub struct ASTTree {
    stream: ASpan<AVec<ASTNode, 1000>>,
    pos: usize,
}

impl ASTTree {
    pub fn new(input: ASpan<AVec<ASTNode, 1000>>) -> Self {
        Self {
            stream: input,
            pos: 0,
        }
    }
    pub fn current_node(&mut self, arena: &mut Arena) -> Option<&ASTNode> {
        let handle = arena.take(self.stream.clone());
        handle.get(self.pos)
    }
    pub fn next_node(&mut self, arena: &mut Arena) -> Option<&ASTNode> {
        let handle = arena.take(self.stream.clone());
        self.pos += 1;
        handle.get(self.pos - 1)
    }
    pub fn peek(&mut self, arena: &mut Arena) -> Option<&ASTNode> {
        let handle = arena.take(self.stream.clone());
        handle.get(self.pos)
    }
    // pub fn from_vec(vec: Vec<ASTNode>) -> Self {
    //     Self {
    //         stream: vec,
    //         pos: 0,
    //     }
    // }

    pub fn reset(&mut self) {
        self.pos = 0;
    }
}
