use pipec_arena::{ASlice, AStr, Arena};
use std::fs::File;
use std::path::PathBuf;

/// This struct is for loading files into the memory, ensuring every Span points to correct memory.
pub struct FileLoader {
    store: Vec<ASlice<AStr>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileId(usize);

impl FileLoader {
    pub fn open(&mut self, input: &PathBuf, arena: &mut Arena) -> std::io::Result<FileId> {
        let id = self.store.len();
        let file = File::open(input)?;
        let src = arena.slice_from_read(file)?;
        self.store.push(src);
        Ok(FileId(id))
    }

    pub fn load(&mut self, input: FileId) -> ASlice<AStr> {
        self.store[input.0]
    }
}

impl Default for FileLoader {
    fn default() -> Self {
        Self {
            store: Vec::with_capacity(100),
        }
    }
}
