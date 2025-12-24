#![allow(dead_code)]
use std::str::Chars;

/// A span in a source later used to be read from using the function parse().
/// The idea is to not store entire Strings inside tokens, but rather these less expensive structs for more performance.
#[derive(Default, Clone, Copy)]
pub struct Span {
    begin: usize,
    end: usize,
}

impl Span {
    pub fn parse<'b>(&mut self, input: &'b str) -> &'b str {
        &input[self.begin..self.end]
    }
}

/// Wraps a Chars<'a> from the Rust standard library and introduces functionality to look at the index of the iterator in memory.
#[derive(Clone, Debug)]
pub struct SpannedIterator<'a> {
    chars: Chars<'a>,
    index: usize,
}

impl<'a> SpannedIterator<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars(),
            index: 0,
        }
    }
    // This function is for the Iterator implementation.
    fn next_char(&mut self) -> Option<char> {
        let next = self.chars.next();
        if let Some(v) = next {
            self.index += v.len_utf8();
        }
        next
    }

    fn index(&self) -> usize {
        self.index
    }
}

impl<'a> Iterator for SpannedIterator<'a> {
    type Item = char;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_char()
    }
}
