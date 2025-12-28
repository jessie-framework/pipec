#![allow(dead_code)]
use core::mem::MaybeUninit;
use core::ptr;
use core::ptr::copy_nonoverlapping;
use core::slice;
use std::mem::{align_of, size_of};

/// An arena allocator made to be used by the compiler.
pub struct Arena {
    data: Box<[MaybeUninit<u8>]>,
    capacity: usize,
    bump: usize,
}

/// An "owned pointer" the arena returns after you do an allocation with it.
/// Lifetimes are a mess to deal with so returning a struct like this instead of say &'a mut T is easier
pub struct ASpan<T> {
    _marker: std::marker::PhantomData<T>,
    pub(crate) val: usize,
}

impl<T> ASpan<T> {
    pub(crate) fn new(input: usize) -> Self {
        Self {
            val: input,
            _marker: std::marker::PhantomData,
        }
    }
}

/// Size for the Arena allocator.
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

    /// Takes an ASpan<T> and turns it into a &mut T.
    pub fn take<'b, T>(&mut self, input: ASpan<T>) -> &'b mut T {
        unsafe {
            let ptr = self.data.as_mut_ptr().add(input.val) as *mut T;
            &mut *ptr
        }
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

    #[inline]
    fn padding<T>(&mut self) -> usize {
        (align_of::<T>() - (self.bump % align_of::<T>())) % align_of::<T>()
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
