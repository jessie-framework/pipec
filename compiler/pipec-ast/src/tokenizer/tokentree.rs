use crate::Token;

pub struct TokenTree {
    stream: Vec<Token>,
    pos: usize,
}

impl TokenTree {
    pub fn new(stream: Vec<Token>) -> Self {
        Self { stream, pos: 0 }
    }
    pub fn current_token(&mut self) -> Option<&Token> {
        self.stream.get(self.pos)
    }
    pub fn next_token(&mut self) -> Option<&Token> {
        self.pos += 1;
        self.stream.get(self.pos - 1)
    }
    pub fn peek(&mut self) -> Option<&Token> {
        self.stream.get(self.pos)
    }
}
