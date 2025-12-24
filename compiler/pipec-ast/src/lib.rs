pub mod ast;
pub mod tokenizer;

use crate::ast::{ASTGenerator, asttree::ASTTree};
use crate::tokenizer::{Token, Tokenizer, tokentree::TokenTree};
use std::{fs::File, io::Read, path::PathBuf};

#[derive(Debug)]
pub struct FileInfo {
    file: PathBuf,
    toks: TokenTree,
}

#[derive(Debug)]
pub struct ASTFileReader {
    file: FileInfo,
}

/// Prevents the compiler from accidentally parsing a recursive dependency.
pub struct RecursiveGuard(Vec<PathBuf>);

impl RecursiveGuard {
    pub fn contains(&self, input: &PathBuf) -> bool {
        self.0.contains(input)
    }

    pub fn push(&mut self, input: PathBuf) {
        self.0.push(input);
    }
}

impl Default for RecursiveGuard {
    fn default() -> Self {
        Self(Vec::with_capacity(100))
    }
}

impl ASTFileReader {
    pub fn new(dir: &PathBuf) -> Result<Self, std::io::Error> {
        let mut buf = String::with_capacity(2000);
        let mut file = File::open(dir)?;
        file.read_to_string(&mut buf)?;
        let tokenizer = Tokenizer::new(&buf);
        let toks = tokenizer.tree();
        let first = Self {
            file: FileInfo {
                file: dir.clone(),
                toks,
            },
        };
        Ok(first)
    }

    pub fn generate_hir(&mut self, guard: &mut RecursiveGuard) -> ASTTree {
        let hirgenerator = ASTGenerator::new(&mut self.file.toks, self.file.file.clone(), guard);
        hirgenerator.tree()
    }
}
