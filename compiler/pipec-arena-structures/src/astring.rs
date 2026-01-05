use pipec_arena::{ABytes, ASlice, Arena};

#[derive(Debug, Clone)]
pub struct AString {
    buf: ASlice<ABytes>,
    index: usize,
    pub capacity: usize,
}

impl AString {
    #[allow(clippy::new_without_default)]
    pub fn with_capacity(capacity: usize, arena: &mut Arena) -> Self {
        let buf = unsafe { arena.alloc_empty(capacity) };
        Self {
            buf,
            index: 0,
            capacity,
        }
    }

    pub fn push(&mut self, input: char, arena: &mut Arena) -> Result<(), AStringError> {
        let mut buf = [0u8; 4];
        let char_len = input.encode_utf8(&mut buf).len();
        if self.index + char_len > self.capacity {
            return Err(AStringError::BufFilled);
        }
        let abuf = arena.take_slice(self.buf);
        abuf[self.index..self.index + char_len].copy_from_slice(&buf[..char_len]);
        self.index += char_len;
        Ok(())
    }

    pub fn push_str(&mut self, input: &str, arena: &mut Arena) -> Result<(), AStringError> {
        let len = input.len();
        if self.index + input.len() > self.capacity {
            return Err(AStringError::BufFilled);
        }
        arena.take_slice(self.buf)[self.index..self.index + len].copy_from_slice(input.as_bytes());
        self.index += len;
        Ok(())
    }

    pub fn as_str(&self, arena: &mut Arena) -> &str {
        unsafe { str::from_utf8_unchecked(&arena.take_slice(self.buf)[..self.index]) }
    }
}

#[derive(Debug)]
pub enum AStringError {
    BufFilled,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pipec_arena::Size;
    #[test]
    fn test_str() -> Result<(), AStringError> {
        let mut arena = Arena::new(Size::Megs(2));
        let mut string = AString::with_capacity(100, &mut arena);
        string.push('h', &mut arena)?;
        string.push('e', &mut arena)?;
        string.push('l', &mut arena)?;
        string.push('l', &mut arena)?;
        string.push('o', &mut arena)?;
        string.push_str(" world!", &mut arena)?;
        let as_str = string.as_str(&mut arena);
        assert_eq!("hello world!", as_str);
        Ok(())
    }
    #[test]
    fn different_size_same_contents() -> Result<(), AStringError> {
        let mut arena = Arena::new(Size::Megs(2));
        let mut s1 = AString::with_capacity(100, &mut arena);
        let mut s2 = AString::with_capacity(50, &mut arena);
        s1.push_str("hello world!", &mut arena)?;
        s2.push_str("hello world!", &mut arena)?;
        assert_eq!(s1.as_str(&mut arena), s2.as_str(&mut arena));
        Ok(())
    }
}
