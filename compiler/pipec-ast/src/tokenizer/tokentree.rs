use crate::tokenizer::Token;
use crate::tokenizer::Tokenizer;
use putbackpeekmore::PutBackPeekMore;

#[derive(Debug)]
pub struct TokenTree<'toks> {
    pub(crate) stream: PutBackPeekMore<Tokenizer<'toks>, 4>,
}

impl<'toks> TokenTree<'toks> {
    pub fn next_token(&mut self) -> Option<Token> {
        self.stream.next()
    }
    pub fn peek(&mut self) -> &Option<Token> {
        self.stream.peek()
    }
}
