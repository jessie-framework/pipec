pub mod hir;
pub mod tokenizer;

use crate::hir::{HIRGenerator, hirtree::HIRTree};
use crate::tokenizer::{Token, Tokenizer, tokentree::TokenTree};
use pipec_cache::{Cached, Decode, Encode, Link};
use pipec_globals::GLOBALS;
use std::{fs::File, io::Read, path::PathBuf};

pub fn generate_ast() -> Result<(), std::io::Error> {
    let (mut reader, reader_link) = ASTFileReader::new(GLOBALS.file.clone())?;
    reader.generate_ast();
    reader.upload_to_cache(reader_link);
    Ok(())
}

#[derive(Hash, Decode, Encode, Debug)]
pub struct FileInfo {
    file: PathBuf,
    toks: TokenTree,
}

impl Cached for FileInfo {}

#[derive(Hash, Decode, Encode, Debug)]
pub struct ASTFileReader {
    stage: FileReaderStage,
    file: FileInfo,
}

impl Cached for ASTFileReader {}

impl ASTFileReader {
    pub fn new(dir: PathBuf) -> Result<(Self, Link), std::io::Error> {
        let mut buf = String::with_capacity(2000);
        let mut file = File::open(&dir)?;
        file.read_to_string(&mut buf)?;
        let tokenizer = Tokenizer::new(&buf);
        let toks = tokenizer.tree();
        let mut first = Self {
            file: FileInfo {
                file: dir.clone(),
                toks,
            },
            stage: FileReaderStage::default(),
        };
        first.try_load();
        println!("{first:#?}");
        let link = first.get_link();
        Ok((first, link))
    }

    pub fn generate_ast(&mut self) {
        let hirgenerator = HIRGenerator::new(&mut self.file.toks);

        self.stage = FileReaderStage::HIR {
            tree: hirgenerator.tree(),
        }
    }

    pub fn upload_to_cache(&mut self, link: Link) {
        self.upload(link);
    }
}

#[derive(Default, Hash, Decode, Encode, Debug)]
pub enum FileReaderStage {
    #[default]
    First,
    HIR {
        tree: HIRTree,
    },
}
