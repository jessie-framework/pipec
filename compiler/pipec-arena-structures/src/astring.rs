use core::cmp::PartialEq;
use core::ops::Deref;
use std::fmt::Display;
use std::mem::MaybeUninit;

#[derive(Debug, Clone)]
pub struct AString<const SIZE: usize> {
    buf: [u8; SIZE],
    index: usize,
}

impl<const SIZE: usize> AString<SIZE> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        #[allow(clippy::uninit_assumed_init)]
        Self {
            buf: unsafe { MaybeUninit::uninit().assume_init() },
            index: 0,
        }
    }

    pub fn push(&mut self, input: char) -> Result<(), AStringError> {
        let mut buf = [0u8; 4];
        let char_len = input.encode_utf8(&mut buf).len();
        if self.index + char_len > SIZE {
            return Err(AStringError::BufFilled);
        }
        self.buf[self.index..self.index + char_len].copy_from_slice(&buf[..char_len]);
        self.index += char_len;
        Ok(())
    }

    pub fn push_str(&mut self, input: &str) -> Result<(), AStringError> {
        let len = input.len();
        if self.index + input.len() > SIZE {
            return Err(AStringError::BufFilled);
        }
        self.buf[self.index..self.index + len].copy_from_slice(input.as_bytes());
        self.index += len;
        Ok(())
    }

    pub fn as_str(&self) -> &str {
        self
    }
}

impl<const SIZE: usize> Display for AString<SIZE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.deref())
    }
}

impl<const SIZE: usize> PartialEq for AString<SIZE> {
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}

impl<const SIZE: usize> PartialEq<AString<SIZE>> for str {
    fn eq(&self, other: &AString<SIZE>) -> bool {
        other.deref() == self
    }
}

impl<const SIZE: usize> PartialEq<str> for AString<SIZE> {
    fn eq(&self, other: &str) -> bool {
        self.deref() == other
    }
}

impl<const SIZE: usize> Deref for AString<SIZE> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        unsafe { str::from_utf8_unchecked(&self.buf[..self.index]) }
    }
}

#[derive(Debug)]
pub enum AStringError {
    BufFilled,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_str() -> Result<(), AStringError> {
        let mut string = AString::<100>::new();
        string.push('h')?;
        string.push('e')?;
        string.push('l')?;
        string.push('l')?;
        string.push('o')?;
        string.push_str(" world!")?;
        println!("{}", &string);
        assert_eq!("hello world!", &string);
        Ok(())
    }
    #[test]
    fn different_size_same_contents() -> Result<(), AStringError> {
        let mut s1 = AString::<100>::new();
        let mut s2 = AString::<50>::new();
        s1.push_str("hello world!")?;
        s2.push_str("hello world!")?;
        assert_eq!(s1.as_str(), s2.as_str());
        Ok(())
    }
}
