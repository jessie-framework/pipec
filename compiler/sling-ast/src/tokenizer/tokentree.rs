use crate::Token;

pub struct TokenTree {
    stream: Vec<Token>,
    pos: usize,
}

impl TokenTree {
    pub fn current_token(&mut self) -> Option<&Token> {
        self.stream.get(self.pos)
    }
    pub fn next_token(&mut self) -> Option<&Token> {
        self.pos += 1;
        self.current_token()
    }
    pub fn peek(&mut self) -> Option<&Token> {
        self.stream.get(self.pos + 1)
    }
}
