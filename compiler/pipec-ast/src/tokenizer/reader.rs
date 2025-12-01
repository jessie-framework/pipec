use pipec_cache::{Decode, Encode};

#[derive(Decode, Encode, Hash)]
#[allow(dead_code)]
pub struct Reader;

#[allow(dead_code)]
impl Reader {
    pub fn new() -> Self {
        Self
    }
}
