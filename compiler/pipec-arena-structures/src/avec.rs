use core::mem::MaybeUninit;
use core::ops::Deref;
use core::slice;
use std::fmt::{Debug, Display};

#[derive(Debug, Hash)]
pub struct AVec<T, const SIZE: usize> {
    buf: [T; SIZE],
    index: usize,
}

impl<T, const SIZE: usize> AVec<T, SIZE> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        #[allow(clippy::uninit_assumed_init)]
        Self {
            buf: unsafe { MaybeUninit::uninit().assume_init() },
            index: 0,
        }
    }

    pub fn take(&self, input: usize) -> Option<&T> {
        if input > self.index {
            return None;
        }
        Some(&self.buf[input])
    }

    pub fn push(&mut self, input: T) -> Result<(), AVecError> {
        if self.index == SIZE {
            return Err(AVecError::BufFilled);
        }

        self.buf[self.index] = input;
        self.index += 1;
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.index
    }

    pub fn is_empty(&self) -> bool {
        self.index == 0
    }

    pub fn iter<'this>(&'this self) -> AVecIter<'this, T, SIZE> {
        AVecIter {
            index: 0,
            iter: self,
        }
    }
}

pub struct AVecIter<'this, T, const SIZE: usize> {
    index: usize,
    iter: &'this AVec<T, SIZE>,
}

impl<'this, T, const SIZE: usize> Iterator for AVecIter<'this, T, SIZE> {
    type Item = &'this T;
    fn next(&mut self) -> Option<Self::Item> {
        let out = self.iter.get(self.index);
        self.index += 1;
        out
    }
}

impl<T: PartialEq, const SIZE: usize> PartialEq<&[T]> for AVec<T, SIZE> {
    fn eq(&self, other: &&[T]) -> bool {
        self.deref() == *other
    }
}
impl<T: PartialEq, const SIZE: usize> PartialEq<AVec<T, SIZE>> for &[T] {
    fn eq(&self, other: &AVec<T, SIZE>) -> bool {
        *self == other.deref()
    }
}

impl<T, const SIZE: usize> Deref for AVec<T, SIZE> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.buf.as_ptr(), self.index) }
    }
}

impl<T: Debug, const SIZE: usize> Display for AVec<T, SIZE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.deref().iter()).finish()
    }
}

#[derive(Debug)]
pub enum AVecError {
    BufFilled,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() -> Result<(), AVecError> {
        #[allow(dead_code)]
        #[derive(Debug, PartialEq)]
        struct Color(u8, u8, u8);
        let mut vec: AVec<Color, 5> = AVec::new();
        vec.push(Color(255, 0, 0))?;
        vec.push(Color(255, 0, 0))?;
        vec.push(Color(255, 0, 0))?;
        assert_eq!(
            &vec[..],
            &[Color(255, 0, 0), Color(255, 0, 0), Color(255, 0, 0)]
        );
        println!("{}", vec);
        Ok(())
    }
}
