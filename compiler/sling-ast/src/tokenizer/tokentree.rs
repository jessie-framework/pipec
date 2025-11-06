use crate::Token;

pub struct TokenTree {
    stream: Vec<Token>,
    pos: usize,
}

impl TokenTree {
    pub fn current(&mut self) -> &Token {
        &self.stream[self.pos]
    }

    pub fn next(&mut self) -> &Token {
        self.pos += 1;
        self.current()
    }
}
