pub mod parser;
pub mod tokenizer;

use crate::parser::{Parsed, Parser};
use crate::tokenizer::tokentree::TokenTree;
use crate::tokenizer::{Token, Tokenizer};
use sling_globals::GLOBALS;
pub fn generate_ast(input: &str) -> Result<(), std::io::Error> {
    let (_loader, loader_link) = FileLoader::new((GLOBALS).file.clone())?;
    let (mut token_loader, token_loader_link) = TokenLoader::new(loader_link);
    token_loader.tokenize(input);
    token_loader.upload(token_loader_link);
    token_loader.print_tokens();
    let mut parser = Parser::new(token_loader.tree());
    loop {
        let next = parser.parse_value();
        if next == Parsed::EOF {
            break;
        }
        println!("{:#?}", next);
    }

    Ok(())
}

use sling_cache::{Cached, Decode, Encode, Link};
use std::{fs::File, io::Read, path::PathBuf};
#[derive(Hash, Decode, Encode)]
pub struct FileLoader {
    file: PathBuf,
    filecontent: String,
}
impl Cached for FileLoader {}

impl FileLoader {
    fn new(input: PathBuf) -> Result<(Self, Link), std::io::Error> {
        let mut file = File::open(&input)?;
        let mut filecontent = String::with_capacity(1000);
        file.read_to_string(&mut filecontent)?;
        let mut out = Self {
            file: input,
            filecontent,
        };
        let link = out.get_link();
        out.try_load();
        Ok((out, link))
    }
}

#[derive(Hash, Decode, Encode)]
pub struct TokenLoader {
    workingon: Link,
    pub tokens: Option<Vec<Token>>,
}
impl Cached for TokenLoader {}

impl TokenLoader {
    pub fn new(link: Link) -> (Self, Link) {
        let mut out = Self {
            workingon: link,
            tokens: None,
        };
        let out_link = out.get_link();
        out.try_load();
        (out, out_link)
    }

    pub fn tree(self) -> TokenTree {
        TokenTree::new(self.tokens.unwrap())
    }

    pub fn tokenize(&mut self, input: &str) {
        if self.tokens.is_some() {
            return;
        }
        let mut tokenizer = Tokenizer::new(input);
        let mut tokens = Vec::with_capacity(200);
        loop {
            let next = tokenizer.consume_next_token();
            if next == Token::EOF {
                break;
            }

            tokens.push(next);
        }
        self.tokens = Some(tokens);
    }

    pub fn print_tokens(&self) {
        if let Some(ref tokens) = self.tokens {
            for token in tokens {
                println!("{:#?}", token);
            }
        }
    }
}
