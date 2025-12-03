pub use bincode::{Decode, Encode};

use pipec_globals::GLOBALS;
use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

pub trait Cached: Encode + Decode<()> + Hash {
    /// Looks for the compilers cache if the struct exists.  If it does, turns the struct into whatever was in the cache.
    fn get_link(&self) -> Link {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        println!("getting link");
        Link(hasher.finish())
    }

    /// Tries to load the struct to the compilers cache.
    fn try_load(&mut self) {
        if let Some(file) = self.file() {
            match std::fs::read(&file) {
                Ok(v) => {
                    let decoded: Result<(Self, _), _> =
                        bincode::decode_from_slice(&v, bincode::config::standard());
                    match decoded {
                        Ok((v, _)) => {
                            *self = v;
                            println!("self decoded");
                        }
                        Err(_e) => {
                            println!("error {_e}");
                            let _ = std::fs::remove_file(file);
                        }
                    }
                }
                Err(_e) => {
                    println!("another error {_e},{:#?}", self.file());
                    // prob should insert some logic
                }
            }
        }
    }

    fn file(&mut self) -> Option<PathBuf> {
        let cache_dir = GLOBALS.cache.as_ref();
        cache_dir.map(|path| path.join(format!("{}.ppc", self.get_link().0)))
    }

    fn upload(&self, link: Link) {
        println!("uploaded to cache {link:#?}");
        match GLOBALS.cache.as_ref() {
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
