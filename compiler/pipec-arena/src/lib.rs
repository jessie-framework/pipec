#![allow(dead_code)]
use core::mem::MaybeUninit;
use core::ptr;
use core::ptr::copy_nonoverlapping;
use core::slice;
use std::{
    io::{BufReader, Read},
    marker::PhantomData,
    mem::{align_of, size_of},
    ops::Range,
};

/// An arena allocator made to be used by the compiler.
pub struct Arena {
    data: Box<[MaybeUninit<u8>]>,
    capacity: usize,
    bump: usize,
}

/// An "owned pointer" for a slice in the arena.
#[derive(Debug)]
pub struct ASlice<T> {
    _marker: PhantomData<T>,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

/// Compiler can't "know" the size of ASlice<[u8]>, so this is for api convenience.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ABytes;
/// Compiler can't "know" the size of ASlice<str>, so this is for api convenience.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AStr;

impl<T> Clone for ASlice<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ASlice<T> {}
impl<T> ASlice<T> {
    pub fn slice(&self, index: Range<usize>) -> Self {
        let start = self.start + index.start;
        let end = self.end + index.end;
        Self {
            _marker: PhantomData,
            start,
            end,
        }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Creates a new ASlice<T> from raw parts.
    ///
    /// # Safety
    /// The returned slice may point to illegal memory.
    pub unsafe fn from_raw_parts(start: usize, end: usize) -> Self {
        Self {
            _marker: PhantomData,
            start,
            end,
        }
    }
}

/// An "owned pointer" the arena returns after you do an allocation with it.
/// Lifetimes are a mess to deal with so returning a struct like this instead of say &'a mut T is easier
#[derive(Debug)]
pub struct ASpan<T> {
    _marker: PhantomData<T>,
    pub(crate) val: usize,
}

impl<T> std::hash::Hash for ASpan<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.val);
    }
}

impl<T> PartialEq for ASpan<T> {
    fn eq(&self, other: &Self) -> bool {
        self.val == other.val
    }
}

impl<T> Eq for ASpan<T> {}

impl<T> Clone for ASpan<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ASpan<T> {}

impl<T> ASpan<T> {
    pub(crate) fn new(input: usize) -> Self {
        Self {
            val: input,
            _marker: std::marker::PhantomData,
        }
    }
}

/// Size for the Arena allocator.
#[derive(Clone, Copy)]
pub enum Size {
    Kibs(usize),
    Megs(usize),
    Gigs(usize),
}

impl Size {
    fn as_bytes(&self) -> usize {
        match self {
            Size::Kibs(v) => 1024 * v,
            Size::Megs(v) => 1024 * 1024 * v,
            Size::Gigs(v) => 1024 * 1024 * 1024 * v,
        }
    }
}

impl Arena {
    /// Returns a new arena.
    pub fn new(capacity: Size) -> Self {
        Self {
            data: Box::<[u8]>::new_uninit_slice(capacity.as_bytes()),
            capacity: capacity.as_bytes(),
            bump: 0,
        }
    }

    /// Allocates a string slice in the arena.
    pub fn alloc_str<'b>(&mut self, input: &str) -> &'b str {
        unsafe {
            let bytes = input.as_bytes();
            let bytes_ptr = bytes.as_ptr();
            let data_ptr = self.data.as_mut_ptr().add(self.bump) as *mut u8;
            copy_nonoverlapping(bytes_ptr, data_ptr, bytes.len());
            self.bump += bytes.len();
            str::from_utf8_unchecked(slice::from_raw_parts(data_ptr, bytes.len()))
        }
    }

    /// Allocates an empty memory region given a size.
    /// # Safety
    /// The returned data will be empty. Yeah
    pub unsafe fn alloc_empty(&mut self, size: usize) -> ASlice<ABytes> {
        unsafe {
            let bump = self.bump;
            self.bump += size;
            ASlice::from_raw_parts(bump, self.bump)
        }
    }

    /// Takes an ASpan<T> and turns it into a &mut T.
    pub fn take<'b, T>(&self, input: ASpan<T>) -> &'b mut T {
        unsafe {
            let ptr = self.data.as_ptr().add(input.val) as *mut T;
            &mut *ptr
        }
    }

    /// Takes an ASlice<&[u8]> and turns it into a &mut [u8].
    pub fn take_slice<'b>(&self, input: ASlice<ABytes>) -> &'b mut [u8] {
        unsafe {
            let ptr = self.data.as_ptr().add(input.start) as *mut u8;

            let slice = slice::from_raw_parts_mut(ptr, input.len());
            slice as &mut [u8]
        }
    }

    /// Takes an ASlice<String> and turns it into a &mut str.
    pub fn take_str_slice<'b>(&self, input: ASlice<AStr>) -> &'b str {
        unsafe {
            let ptr = self.data.as_ptr().add(input.start) as *mut u8;

            let slice = slice::from_raw_parts_mut(ptr, input.len());
            str::from_utf8_unchecked_mut(slice)
        }
    }

    /// Takes an implementator of the Read trait as input and allocates it in the arena.
    pub fn slice_from_read<O>(&mut self, input: impl Read) -> Result<ASlice<O>, std::io::Error> {
        let start = self.bump;
        let mut end = self.bump;
        let reader = BufReader::new(input);
        for byte in reader.bytes() {
            let byte = byte?;
            self.alloc_byte(byte);
            end += 1;
        }
        Ok(unsafe { ASlice::from_raw_parts(start, end) })
    }

    /// Allocates T in the arena and returns an ASpan<T>
    #[inline]
    pub fn alloc<T>(&mut self, input: T) -> ASpan<T> {
        unsafe {
            let ptr = self
                .data
                .as_mut_ptr()
                .add(self.bump)
                .add(self.padding::<T>()) as *mut T;
            ptr::write(ptr, input);
            let out = ASpan::new(self.bump + self.padding::<T>());
            self.bump += size_of::<T>() + self.padding::<T>();
            out
        }
    }

    /// Allocates a byte in the arena and ignores the result.
    #[inline]
    pub(crate) fn alloc_byte(&mut self, input: u8) {
        unsafe {
            let ptr = self.data.as_mut_ptr().add(self.bump) as *mut u8;
            ptr::write(ptr, input);
            self.bump += 1;
        }
    }

    #[inline]
    fn padding<T>(&mut self) -> usize {
        (align_of::<T>() - (self.bump % align_of::<T>())) % align_of::<T>()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.bump
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_alloc() {
        #[derive(Debug)]
        struct RandomStruct(usize, u8);
        #[derive(Debug)]
        struct BigFat(usize, u8, usize, u16, u32, char, char, char, char);
        #[derive(Debug)]
        struct FakeBoxed<'a>(Option<&'a Self>);
        let mut arena = Arena::new(Size::Kibs(2));
        let alloced_struct = arena.alloc(RandomStruct(100, 10));
        let another2 = arena.alloc(31321);
        let bigfat = arena.alloc(BigFat(32, 23, 254, 64, 32, 'a', 'b', 'c', 'd'));
        let smol = arena.alloc(2u8);
        let list = arena.alloc([32, 32, 321, 5]);
        let fake = arena.alloc(FakeBoxed(Some(&FakeBoxed(Some(&FakeBoxed(None))))));
        println!("{:#?}", arena.take(alloced_struct));
        println!("{:#?}", arena.take(another2));
        println!("{:#?}", arena.take(bigfat));
        println!("{:#?}", arena.take(smol));
        println!("{:#?}", arena.take(list));
        println!("{:#?}", arena.take(fake));
    }
}
