use super::ASTNode;
use pipec_file_loader::FileId;

#[derive(Debug, Clone)]
pub struct ASTTree {
    stream: Vec<ASTNode>,
    pos: usize,
    pub id: FileId,
}

impl ASTTree {
    pub fn new(stream: Vec<ASTNode>, id: FileId) -> Self {
        Self { stream, pos: 0, id }
    }
    pub fn current_node(&mut self) -> Option<&ASTNode> {
        self.stream.get(self.pos)
    }
    pub fn next_node(&mut self) -> Option<ASTNode> {
        self.pos += 1;
        self.stream.get(self.pos).cloned()
    }
    pub fn peek(&mut self) -> Option<&ASTNode> {
        self.stream.get(self.pos)
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
