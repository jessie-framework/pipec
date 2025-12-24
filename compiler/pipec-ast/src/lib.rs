pub mod ast;
pub mod tokenizer;

use crate::ast::{ASTGenerator, asttree::ASTTree};
use crate::tokenizer::{Token, Tokenizer, tokentree::TokenTree};
use pipec_cache::{Cached, Decode, Encode, Link};
use std::sync::Arc;
use std::{fs::File, io::Read, path::PathBuf};

#[derive(Hash, Decode, Encode, Debug)]
pub struct FileInfo {
    file: PathBuf,
    toks: TokenTree,
}

impl Cached for FileInfo {}

#[derive(Hash, Decode, Encode, Debug)]
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

impl Cached for ASTFileReader {}

impl ASTFileReader {
    pub fn new(
        dir: &PathBuf,
        cache_dir: Arc<Option<PathBuf>>,
    ) -> Result<(Self, Link), std::io::Error> {
        let mut buf = String::with_capacity(2000);
        let mut file = File::open(dir)?;
        file.read_to_string(&mut buf)?;
        let tokenizer = Tokenizer::new(&buf);
        let toks = tokenizer.tree();
        let mut first = Self {
            file: FileInfo {
                file: dir.clone(),
                toks,
            },
        };
        first.try_load(cache_dir);
        let link = first.get_link();
        Ok((first, link))
    }

    pub fn generate_hir(
        &mut self,
        guard: &mut RecursiveGuard,
        cache_dir: Arc<Option<PathBuf>>,
    ) -> ASTTree {
        let hirgenerator = ASTGenerator::new(
            &mut self.file.toks,
            self.file.file.clone(),
            guard,
            cache_dir,
        );
        hirgenerator.tree()
    }

    pub fn upload_to_cache(&mut self, link: Link, cache_dir: Arc<Option<PathBuf>>) {
        self.upload(link, cache_dir);
    }
}
