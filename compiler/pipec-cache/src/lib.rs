pub use bincode::{Decode, Encode};

use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::Arc;

pub trait Cached: Encode + Decode<()> + Hash {
    /// Looks for the compilers cache if the struct exists.  If it does, turns the struct into whatever was in the cache.
    fn get_link(&self) -> Link {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        Link(hasher.finish())
    }

    /// Tries to load the struct to the compilers cache.
    fn try_load(&mut self, cache_dir: Arc<Option<PathBuf>>) {
        if let Some(file) = self.file(cache_dir) {
            match std::fs::read(&file) {
                Ok(v) => {
                    let decoded: Result<(Self, _), _> =
                        bincode::decode_from_slice(&v, bincode::config::standard());
                    match decoded {
                        Ok((v, _)) => {
                            *self = v;
                        }
                        Err(_e) => {
                            let _ = std::fs::remove_file(file);
                        }
                    }
                }
                Err(_e) => {
                    // prob should insert some logic
                }
            }
        }
    }

    fn file(&mut self, cache_dir: Arc<Option<PathBuf>>) -> Option<PathBuf> {
        cache_dir
            .as_ref()
            .clone()
            .map(|path| path.join(format!("{}.ppc", self.get_link().0)))
    }

    fn upload(&self, link: Link, cache_dir: Arc<Option<PathBuf>>) {
        match cache_dir.as_ref() {
            None => {}
            Some(dir) => {
                if !dir.exists() {
                    std::fs::create_dir_all(dir).unwrap();
                }
                let file_dir = dir.join(format!("{}.ppc", link.0));
                if !file_dir.exists() {
                    let mut file = File::create(file_dir).unwrap();
                    let _ = bincode::encode_into_std_write(
                        self,
                        &mut file,
                        bincode::config::standard(),
                    );
                }
            }
        }
    }
}
#[derive(Decode, Encode, Hash, Copy, Clone, Default, Debug)]
pub struct Link(u64);
