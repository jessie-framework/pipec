#![allow(dead_code)]
use pipec_arena::{ASlice, AStr, Arena};
use putbackpeekmore::PutBackPeekMore;
use std::str::Chars;

/// A span in a source later used to be read from using the function parse().
/// The idea is to not store entire Strings inside tokens, but rather these less expensive structs for more performance.
#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct Span {
    pub begin: usize,
    pub end: usize,
}

impl Span {
    pub fn parse_arena<'b>(&self, input: ASlice<AStr>, arena: &Arena) -> &'b str {
        let parsed = arena.take_str_slice(input);
        &parsed[self.begin..self.end]
    }

    pub fn parse_str<'b>(&self, input: &'b str) -> &'b str {
        &input[self.begin..self.end]
    }
    pub fn end<'a>(&mut self, input: &SpannedIterator<'a>) {
        self.end = input.index();
    }
}

/// Wraps a Chars<'a> from the Rust standard library and introduces functionality to look at the index of the iterator in memory.
#[derive(Clone, Debug)]
pub struct SpannedIterator<'a> {
    chars: PutBackPeekMore<Chars<'a>, 4>,
    index: usize,
}

impl<'a> SpannedIterator<'a> {
    pub fn new(input: &'a str) -> Self {
        let chars = PutBackPeekMore::new(input.chars());
        Self { chars, index: 0 }
    }
    // This function is for the Iterator implementation.
    fn next_char(&mut self) -> Option<char> {
        let next = self.chars.next();
        if let Some(v) = next {
            self.index += v.len_utf8();
        }
        next
    }
    pub fn new_span(&self) -> Span {
        let index = self.index();
        Span {
            begin: index,
            end: index,
        }
    }

    pub fn peek(&mut self) -> &Option<char> {
        self.chars.peek()
    }

    pub fn peek_value(&mut self, input: usize) -> &[Option<char>] {
        self.chars.peek_value(input)
    }

    pub fn index(&self) -> usize {
        self.index
    }
}

impl<'a> Iterator for SpannedIterator<'a> {
    type Item = char;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_char()
    }
}
