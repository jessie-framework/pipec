use super::ASTNode;

#[derive(PartialEq, Debug, Clone)]
pub struct ASTTree {
    stream: Vec<ASTNode>,
    pos: usize,
}

impl ASTTree {
    pub fn stream(&self) -> &[ASTNode] {
        &self.stream
    }

    pub fn new(input: Vec<ASTNode>) -> Self {
        Self {
            stream: input,
            pos: 0,
        }
    }
    pub fn current_node(&mut self) -> Option<&ASTNode> {
        self.stream.get(self.pos)
    }
    pub fn next_node(&mut self) -> Option<&ASTNode> {
        self.pos += 1;
        self.stream.get(self.pos - 1)
    }
    pub fn peek(&mut self) -> Option<&ASTNode> {
        self.stream.get(self.pos)
    }
    pub fn from_vec(vec: Vec<ASTNode>) -> Self {
        Self {
            stream: vec,
            pos: 0,
        }
    }

    pub fn reset(&mut self) {
        self.pos = 0;
    }
}
