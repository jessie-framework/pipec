use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

/// This struct is for loading files into the memory, ensuring every Span points to correct memory.
pub struct FileLoader {
    store: String,
}

/// A wrapper for string sources to ensure API correctness.
pub struct FileData<'a> {
    pub data: &'a str,
}

impl FileLoader {
    fn load<'a>(&'a mut self, input: &PathBuf) -> std::io::Result<FileData<'a>> {
        let mut file = File::open(input)?;
        let bytes_read = file.read_to_string(&mut self.store)?;
        Ok(FileData {
            data: &self.store[self.store.len() - bytes_read..],
        })
    }
}

impl Default for FileLoader {
    fn default() -> Self {
        Self {
            store: String::with_capacity(100000),
        }
    }
}
