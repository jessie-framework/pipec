use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

/// This struct is for loading files into the memory, ensuring every Span points to correct memory.
pub struct FileLoader {
    store: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileId(usize);

impl FileLoader {
    pub fn open(&mut self, input: &PathBuf) -> std::io::Result<FileId> {
        let id = self.store.len();
        let mut src = String::with_capacity(10000);
        let mut file = File::open(input)?;
        file.read_to_string(&mut src)?;
        Ok(FileId(id))
    }

    pub fn load(&mut self, input: FileId) -> String {
        self.store[input.0].clone()
    }
}

impl Default for FileLoader {
    fn default() -> Self {
        Self {
            store: Vec::with_capacity(100),
        }
    }
}
